use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use actix_web::rt::task;
use async_trait::async_trait;
use serde_json::{json, Value};
use shared::models::{
    forms::{CustomInputJob, GenerateTestsJob, SubmitJob},
    language::Language,
    python::PythonSignature,
    runner::{CustomInputResponse, Diagnostic, DiagnosticType, RunnerError, RunnerResponse},
    test::Test,
};
use wasm_memory::{
    ContainerVariant, ContainerVariantType, FunctionType, FunctionValue, WasmFunctionCall,
    WasmMemory,
};
use wasmtime::{Caller, InstancePre, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, Trap};
use wasmtime_wasi::{
    p1::{add_to_linker_sync, WasiP1Ctx},
    p2::pipe::{MemoryInputPipe, MemoryOutputPipe},
    DirPerms, FilePerms, I32Exit, WasiCtxBuilder,
};

use super::{
    clamp_fuel,
    cplusplus::{self, PrefixLocks},
    epoch_ticks_until, finish_test, run_command, timeout_error, Runner, TestResults, WasmRuntime,
    MAX_FUEL, MAX_MEMORY,
};

const HARNESS: &str = include_str!("python_harness.py");
const RESULT_MARK: &str = "\u{1e}acm-result ";
/// Fuel for starting the interpreter and decoding the job. No submitted code
/// runs before the harness asks for the metered budget.
const STARTUP_FUEL: u64 = 100_000_000_000;
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_RESULT_BYTES: usize = 64 * 1024 * 1024;
const MAX_STDERR_MESSAGE: usize = 2000;
/// A test may use this many times its budget before it is stopped. Python
/// budgets are already wide, so the margin is narrower than for compiled code.
const TEST_PADDING: i64 = 4;
const CUSTOM_PADDING: i64 = 20;
/// WASI `errno::notsup`.
const ERRNO_NOTSUP: i32 = 58;

struct State {
    wasi: WasiP1Ctx,
    limits: StoreLimits,
    budget: u64,
    metered: bool,
}

/// CPython compiled to WebAssembly, loaded once and instantiated per test.
struct Interpreter {
    home: PathBuf,
    instance: OnceLock<Result<InstancePre<State>, String>>,
}

impl Interpreter {
    fn instance(&self, runtime: &WasmRuntime) -> Result<InstancePre<State>, RunnerError> {
        self.instance
            .get_or_init(|| {
                self.load(runtime).map_err(|error| {
                    log::error!("cannot load Python from {}: {error:#}", self.home.display());
                    "Python is not available in this runner.".to_string()
                })
            })
            .clone()
            .map_err(|message| RunnerError::InternalServerError { message })
    }

    fn load(&self, runtime: &WasmRuntime) -> anyhow::Result<InstancePre<State>> {
        let bytes = std::fs::read(self.home.join("python.wasm"))?;
        let module = Module::from_binary(&runtime.engine, &bytes)?;
        let mut linker = Linker::new(&runtime.engine);
        add_to_linker_sync(&mut linker, |state: &mut State| &mut state.wasi)?;
        // The harness yields once, immediately before it loads the submission.
        // Later calls change nothing, so a submission cannot refill its fuel.
        linker.allow_shadowing(true);
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "sched_yield",
            |mut caller: Caller<'_, State>| -> wasmtime::Result<i32> {
                if !caller.data().metered {
                    let budget = caller.data().budget;
                    caller.set_fuel(budget)?;
                    caller.data_mut().metered = true;
                }
                Ok(0)
            },
        )?;
        // A sleep would block this thread in the host, where neither fuel nor
        // the deadline can interrupt it. Sleeping fails in the guest instead.
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "poll_oneoff",
            |_: Caller<'_, State>, _: i32, _: i32, _: i32, _: i32| ERRNO_NOTSUP,
        )?;
        Ok(linker.instantiate_pre(&module)?)
    }
}

#[derive(Clone)]
pub struct Python {
    runtime: Arc<WasmRuntime>,
    interpreter: Arc<Interpreter>,
    locks: PrefixLocks,
}

impl Python {
    /// Compiling the interpreter takes several seconds, so it starts in the
    /// background. Requests that arrive earlier wait for it.
    pub fn new(runtime: Arc<WasmRuntime>, home: &Path) -> Self {
        let python = Self {
            runtime,
            interpreter: Arc::new(Interpreter {
                home: home.to_path_buf(),
                instance: OnceLock::new(),
            }),
            locks: PrefixLocks::default(),
        };
        let warm = python.clone();
        let spawned = std::thread::Builder::new()
            .name("python-loader".into())
            .spawn(move || {
                let _ = warm.interpreter.instance(&warm.runtime);
            });
        if let Err(error) = spawned {
            log::warn!("cannot preload Python: {error}");
        }
        python
    }

    async fn run(
        &self,
        implementation: Arc<str>,
        input: WasmFunctionCall,
        fuel: Option<i64>,
        deadline: tokio::time::Instant,
        timeout_message: &str,
    ) -> Result<(FunctionValue, String, u64), RunnerError> {
        if tokio::time::Instant::now() >= deadline {
            return Err(timeout_error(timeout_message));
        }
        let python = self.clone();
        let blocking_timeout_message = timeout_message.to_string();
        let handle = task::spawn_blocking(move || {
            python.run_blocking(
                &implementation,
                input,
                fuel,
                deadline,
                &blocking_timeout_message,
            )
        });
        tokio::pin!(handle);
        tokio::select! {
            biased;
            result = &mut handle => result.map_err(|e| {
                log::error!("caught error: {e}");
                RunnerError::InternalServerError {
                    message: "Failed to create thread".to_string(),
                }
            })?,
            _ = tokio::time::sleep_until(deadline) => {
                let _ = handle.await;
                Err(timeout_error(timeout_message))
            }
        }
    }

    fn run_blocking(
        &self,
        implementation: &str,
        input: WasmFunctionCall,
        fuel: Option<i64>,
        deadline: tokio::time::Instant,
        timeout_message: &str,
    ) -> Result<(FunctionValue, String, u64), RunnerError> {
        let internal = |context: &str, error: &dyn std::fmt::Display| {
            log::error!("{context}: {error}");
            RunnerError::InternalServerError {
                message: format!("Failed to start Python: {context}"),
            }
        };
        let instance_pre = self.interpreter.instance(&self.runtime)?;
        if tokio::time::Instant::now() >= deadline {
            return Err(timeout_error(timeout_message));
        }

        let signature = PythonSignature::from_calls([&input])?;
        let job = json!({
            "source": implementation,
            "name": signature.name(),
            "args": input.arguments.iter().map(encode).collect::<Vec<_>>(),
            "types": signature.arguments(),
            "returns": signature.result(),
        });
        let stdout = MemoryOutputPipe::new(MAX_OUTPUT_BYTES);
        let stderr = MemoryOutputPipe::new(MAX_RESULT_BYTES);
        let mut wasi = WasiCtxBuilder::new();
        wasi.args(&[
            "python", "-I", "-S", "-B", "-u", "-W", "ignore", "-c", HARNESS,
        ])
        .stdin(MemoryInputPipe::new(job.to_string()))
        .stdout(stdout.clone())
        .stderr(stderr.clone())
        .allow_blocking_current_thread(true)
        // The guest sees the standard library and nothing else, read-only.
        .preopened_dir(
            self.interpreter.home.join("lib"),
            "/lib",
            DirPerms::READ,
            FilePerms::READ,
        )
        .map_err(|e| internal("standard library", &e))?;

        let budget = clamp_fuel(fuel);
        let mut store = Store::new(
            &self.runtime.engine,
            State {
                wasi: wasi.build_p1(),
                limits: StoreLimitsBuilder::new()
                    .memory_size(MAX_MEMORY)
                    .instances(1)
                    .build(),
                budget,
                metered: false,
            },
        );
        store.set_epoch_deadline(epoch_ticks_until(
            deadline,
            tokio::time::Instant::now(),
            self.runtime.epoch_period,
        ));
        store.epoch_deadline_trap();
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(STARTUP_FUEL)
            .map_err(|e| internal("fuel", &e))?;

        let outcome = instance_pre
            .instantiate(&mut store)
            .and_then(|instance| instance.get_typed_func::<(), ()>(&mut store, "_start"))
            .and_then(|start| start.call(&mut store, ()));
        let remaining = store.get_fuel().map_err(|e| internal("fuel", &e))?;
        let metered = store.data().metered;
        drop(store);

        // A clean interpreter exit is reported as an error carrying its status.
        let trap = outcome
            .err()
            .filter(|error| error.downcast_ref::<I32Exit>().is_none());
        if trap
            .as_ref()
            .is_some_and(|error| error.downcast_ref::<Trap>() == Some(&Trap::Interrupt))
        {
            return Err(timeout_error(timeout_message));
        }
        let stderr = stderr.contents();
        if !metered {
            let detail = match &trap {
                Some(error) => error.root_cause().to_string(),
                None => tail(&stderr),
            };
            return Err(internal("the interpreter stopped early", &detail));
        }

        let output = String::from_utf8_lossy(&stdout.contents()).into_owned();
        let fuel = budget.saturating_sub(remaining);
        match record(&stderr) {
            Some(Record::Value(value)) => decode(input.return_type, value)
                .map(|value| (value, output, fuel))
                .map_err(|message| RunnerError::RuntimeError { message }),
            Some(Record::Error(message)) => Err(RunnerError::RuntimeError { message }),
            Some(Record::Syntax(diagnostic)) => Err(RunnerError::CompilationError {
                diagnostics: vec![diagnostic],
            }),
            None => Err(RunnerError::RuntimeError {
                message: match trap {
                    Some(error) => error.root_cause().to_string(),
                    None => format!("Python exited without a result.\n{}", tail(&stderr))
                        .trim_end()
                        .to_string(),
                },
            }),
        }
    }

    async fn run_test(
        &self,
        implementation: &Arc<str>,
        test: Test,
        padding: i64,
        deadline: tokio::time::Instant,
        timeout_message: &str,
    ) -> Result<(shared::models::test::TestResult, String), RunnerError> {
        let max_runtime = test
            .max_fuel
            .map(|fuel| fuel.saturating_mul(padding).clamp(0, MAX_FUEL));
        let outcome = self
            .run(
                implementation.clone(),
                test.input.clone(),
                max_runtime,
                deadline,
                timeout_message,
            )
            .await;
        finish_test(test, max_runtime, outcome)
    }
}

#[async_trait]
impl Runner for Python {
    async fn run_tests(
        &self,
        form: SubmitJob,
        deadline: tokio::time::Instant,
        timeout_message: &str,
    ) -> Result<RunnerResponse, RunnerError> {
        PythonSignature::from_calls(form.tests.iter().map(|test| &test.input))?;
        let implementation: Arc<str> = form.implementation.into();
        let mut results = TestResults::new();
        let run_all = form.run_all;
        for mut test in form.tests {
            test.adjust_runtime(form.runtime_multiplier);
            test.max_fuel = Language::Python.fuel_limit(test.max_fuel);
            let (result, _) = self
                .run_test(
                    &implementation,
                    test,
                    TEST_PADDING,
                    deadline,
                    timeout_message,
                )
                .await?;
            if !results.insert(result) && !run_all {
                break;
            }
        }
        Ok(results.into())
    }

    async fn generate_tests(
        &self,
        _form: GenerateTestsJob,
        _deadline: tokio::time::Instant,
        _timeout_message: &str,
    ) -> Result<Vec<Test>, RunnerError> {
        Err(RunnerError::UnsupportedSignature {
            message: "Test generation uses C++ references, not Python.".into(),
        })
    }

    async fn run_custom_input(
        &self,
        form: CustomInputJob,
        deadline: tokio::time::Instant,
        timeout_message: &str,
    ) -> Result<CustomInputResponse, RunnerError> {
        PythonSignature::from_calls([&form.input])?;
        let prefix = PathBuf::from(format!(
            "/tmp/acm/python/custom/{}/{}",
            form.user_id, form.problem_id
        ));
        let _guard = self
            .locks
            .acquire(prefix.clone(), deadline, timeout_message)
            .await?;
        let reference = cplusplus::compile_problem(
            &prefix.join("reference"),
            &cplusplus::process_file(&form.reference),
            deadline,
            timeout_message,
        )
        .await?;
        let (expected_output, _, fuel) = run_command(
            self.runtime.clone(),
            &reference,
            form.input.clone(),
            None,
            Language::Cpp,
            deadline,
            timeout_message,
        )
        .await?;
        let mut test = Test {
            id: 0,
            index: 0,
            max_fuel: Some(fuel as i64),
            input: form.input,
            expected_output,
        };
        test.adjust_runtime(form.runtime_multiplier);
        test.max_fuel = Language::Python.fuel_limit(test.max_fuel);
        let (result, output) = self
            .run_test(
                &form.implementation.into(),
                test,
                CUSTOM_PADDING,
                deadline,
                timeout_message,
            )
            .await?;
        Ok(CustomInputResponse { result, output })
    }
}

enum Record {
    Value(Value),
    Error(String),
    Syntax(Diagnostic),
}

/// Finds the harness result. A submission that writes its own record only
/// replaces its own answer, which is still compared with the expected output.
fn record(stderr: &[u8]) -> Option<Record> {
    let text = String::from_utf8_lossy(stderr);
    let line = text
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix(RESULT_MARK))?;
    let mut record: Value = serde_json::from_str(line).ok()?;
    if let Some(value) = record.get_mut("value") {
        return Some(Record::Value(value.take()));
    }
    if let Some(message) = record["error"].as_str() {
        return Some(Record::Error(message.into()));
    }
    if let Some(name) = record["missing"].as_str() {
        return Some(Record::Syntax(Diagnostic {
            line: 0,
            col: 0,
            diagnostic_type: DiagnosticType::Error,
            message: format!("Expected a function named \"{name}\", but it was not found."),
        }));
    }
    let syntax = record.get("syntax")?;
    Some(Record::Syntax(Diagnostic {
        line: syntax["line"].as_u64().unwrap_or(0) as usize,
        col: syntax["col"].as_u64().unwrap_or(0) as usize,
        diagnostic_type: DiagnosticType::Error,
        message: syntax["message"].as_str()?.into(),
    }))
}

fn tail(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let start = text
        .char_indices()
        .rev()
        .nth(MAX_STDERR_MESSAGE - 1)
        .map_or(0, |(index, _)| index);
    text[start..].trim().to_string()
}

fn encode(value: &FunctionValue) -> Value {
    match value {
        FunctionValue::String(v) => encode_container(v, |s| Value::from(s.as_str())),
        FunctionValue::Int(v) => encode_container(v, |i| Value::from(*i)),
        FunctionValue::Long(v) => encode_container(v, |i| Value::from(*i)),
        // Floats travel as text so that NaN and the infinities survive JSON.
        FunctionValue::Float(v) => encode_container(v, |f| Value::from(format!("{f:e}"))),
        FunctionValue::Double(v) => encode_container(v, |f| Value::from(format!("{f:e}"))),
        FunctionValue::Char(v) => encode_container(v, |c| Value::from(c.to_string())),
        FunctionValue::Bool(v) => encode_container(v, |b| Value::from(*b)),
    }
}

fn encode_container<T: WasmMemory>(
    value: &ContainerVariant<T>,
    item: impl Fn(&T) -> Value,
) -> Value {
    let list = |items: &Vec<T>| Value::Array(items.iter().map(&item).collect());
    match value {
        ContainerVariant::Single(single) => item(single),
        ContainerVariant::List(items) => list(items),
        ContainerVariant::Grid(rows) | ContainerVariant::Graph(rows) => {
            Value::Array(rows.iter().map(list).collect())
        }
    }
}

fn decode(ty: FunctionType, value: Value) -> Result<FunctionValue, String> {
    Ok(match ty {
        FunctionType::String(v) => FunctionValue::String(decode_container(v, value, |item| {
            item.as_str().map(str::to_owned).ok_or_else(malformed)
        })?),
        FunctionType::Int(v) => FunctionValue::Int(decode_container(v, value, |item| {
            item.as_i64()
                .and_then(|i| i32::try_from(i).ok())
                .ok_or_else(|| format!("The returned integer {item} does not fit in 32 bits."))
        })?),
        FunctionType::Long(v) => FunctionValue::Long(decode_container(v, value, |item| {
            item.as_i64()
                .ok_or_else(|| format!("The returned integer {item} does not fit in 64 bits."))
        })?),
        FunctionType::Float(v) => FunctionValue::Float(decode_container(v, value, |item| {
            decode_float(item).map(|f| f as f32)
        })?),
        FunctionType::Double(v) => FunctionValue::Double(decode_container(v, value, decode_float)?),
        FunctionType::Char(v) => FunctionValue::Char(decode_container(v, value, |item| {
            let mut chars = item.as_str().ok_or_else(malformed)?.chars();
            match (chars.next(), chars.next()) {
                // Characters are single bytes in the test format.
                (Some(c), None) if (c as u32) < 256 => Ok(c),
                (Some(c), None) => Err(format!(
                    "The returned character {c:?} is outside the supported one-byte range."
                )),
                _ => Err(malformed()),
            }
        })?),
        FunctionType::Bool(v) => FunctionValue::Bool(decode_container(v, value, |item| {
            item.as_bool().ok_or_else(malformed)
        })?),
    })
}

fn malformed() -> String {
    "The returned value could not be decoded.".into()
}

fn decode_float(item: &Value) -> Result<f64, String> {
    item.as_str()
        .and_then(|text| text.parse().ok())
        .ok_or_else(malformed)
}

fn decode_container<T: WasmMemory>(
    variant: ContainerVariantType,
    value: Value,
    item: impl Fn(&Value) -> Result<T, String>,
) -> Result<ContainerVariant<T>, String> {
    let list = |value: &Value| -> Result<Vec<T>, String> {
        value
            .as_array()
            .ok_or_else(malformed)?
            .iter()
            .map(&item)
            .collect()
    };
    let grid = |value: &Value| -> Result<Vec<Vec<T>>, String> {
        value
            .as_array()
            .ok_or_else(malformed)?
            .iter()
            .map(list)
            .collect()
    };
    Ok(match variant {
        ContainerVariantType::Single => ContainerVariant::Single(item(&value)?),
        ContainerVariantType::List => ContainerVariant::List(list(&value)?),
        ContainerVariantType::Grid => ContainerVariant::Grid(grid(&value)?),
        ContainerVariantType::Graph => ContainerVariant::Graph(grid(&value)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn int(value: i32) -> FunctionValue {
        FunctionValue::Int(ContainerVariant::Single(value))
    }

    #[test]
    fn values_cross_the_sandbox_boundary_as_json() {
        let grid = FunctionValue::Double(ContainerVariant::Grid(vec![
            vec![1.5, f64::NEG_INFINITY],
            vec![],
        ]));
        let encoded = encode(&grid);
        assert_eq!(encoded, json!([["1.5e0", "-inf"], []]));
        let decoded = decode(FunctionType::Double(ContainerVariantType::Grid), encoded).unwrap();
        // Float containers never compare equal as test values, so compare the text.
        assert_eq!(format!("{decoded:?}"), format!("{grid:?}"));
        let nan = decode(
            FunctionType::Float(ContainerVariantType::Single),
            json!("nan"),
        );
        assert!(matches!(nan, Ok(FunctionValue::Float(ContainerVariant::Single(f))) if f.is_nan()));
        for (value, ty) in [
            (
                FunctionValue::String(ContainerVariant::List(vec!["a\"\n".into(), "é".into()])),
                FunctionType::String(ContainerVariantType::List),
            ),
            (
                FunctionValue::Char(ContainerVariant::Single('x')),
                FunctionType::Char(ContainerVariantType::Single),
            ),
            (
                FunctionValue::Bool(ContainerVariant::List(vec![true, false])),
                FunctionType::Bool(ContainerVariantType::List),
            ),
            (
                FunctionValue::Long(ContainerVariant::Graph(vec![vec![i64::MIN, i64::MAX]])),
                FunctionType::Long(ContainerVariantType::Graph),
            ),
        ] {
            assert_eq!(decode(ty, encode(&value)).unwrap(), value);
        }

        let int = FunctionType::Int(ContainerVariantType::Single);
        assert!(decode(int, json!(1u64 << 40))
            .unwrap_err()
            .contains("32 bits"));
        assert!(decode(int, json!(1.5)).is_err());
        assert!(decode(int, json!([1])).is_err());
        let long = FunctionType::Long(ContainerVariantType::Single);
        assert!(decode(long, json!(u64::MAX))
            .unwrap_err()
            .contains("64 bits"));
        let character = FunctionType::Char(ContainerVariantType::Single);
        assert!(decode(character, json!("ab")).is_err());
        assert!(decode(character, json!("\u{1f600}")).is_err());
        assert!(decode(FunctionType::Int(ContainerVariantType::List), json!(1)).is_err());
    }

    #[test]
    fn only_the_last_complete_record_is_used() {
        let stderr =
            format!("noise\n{RESULT_MARK}{{\"value\": 1}}\n{RESULT_MARK}{{\"error\": \"boom\"}}\n");
        assert!(matches!(
            record(stderr.as_bytes()),
            Some(Record::Error(message)) if message == "boom"
        ));
        assert!(record(format!("{RESULT_MARK}{{\"value\": 1").as_bytes()).is_none());
        assert!(record(b"Fatal Python error").is_none());
        let syntax = format!(
            "{RESULT_MARK}{{\"syntax\": {{\"line\": 2, \"col\": 5, \"message\": \"bad\"}}}}\n"
        );
        assert!(matches!(
            record(syntax.as_bytes()),
            Some(Record::Syntax(Diagnostic {
                line: 2,
                col: 5,
                ..
            }))
        ));
        let missing = format!("{RESULT_MARK}{{\"missing\": \"add\"}}\n");
        assert!(matches!(
            record(missing.as_bytes()),
            Some(Record::Syntax(Diagnostic { line: 0, message, .. })) if message.contains("\"add\"")
        ));
        assert_eq!(tail("é".repeat(5000).as_bytes()).chars().count(), 2000);
    }

    #[tokio::test]
    async fn a_missing_interpreter_is_an_internal_error() {
        let Some((runtime, _)) = runtime() else {
            return;
        };
        let python = Python::new(runtime, Path::new("/nonexistent/acm-python"));
        let call = WasmFunctionCall::new(
            "add",
            vec![int(1)],
            FunctionType::Int(ContainerVariantType::Single),
        );
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        assert!(matches!(
            python.run("".into(), call, None, deadline, "timeout").await,
            Err(RunnerError::InternalServerError { message }) if message.contains("not available")
        ));
    }

    /// Set `ACM_PYTHON_HOME` to a directory with `python.wasm` and `lib` to run
    /// the tests below. Without it they pass without checking anything; the
    /// runner container tests cover the same behavior.
    fn runtime() -> Option<(Arc<WasmRuntime>, PathBuf)> {
        let home = PathBuf::from(std::env::var_os("ACM_PYTHON_HOME")?);
        let directory = std::env::temp_dir().join("acm-python-test");
        std::fs::create_dir_all(&directory).unwrap();
        let config = directory.join("wasmtime-cache.toml");
        std::fs::write(
            &config,
            format!(
                "[cache]\ndirectory = {:?}\n",
                directory.join("cache").to_str().unwrap()
            ),
        )
        .unwrap();
        Some((WasmRuntime::new(&config).unwrap(), home))
    }

    async fn run(
        python: &Python,
        source: &str,
        call: &WasmFunctionCall,
        fuel: Option<i64>,
    ) -> Result<(FunctionValue, String, u64), RunnerError> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(900);
        python
            .run(source.into(), call.clone(), fuel, deadline, "timeout")
            .await
    }

    fn message(result: Result<(FunctionValue, String, u64), RunnerError>) -> String {
        match result {
            Err(RunnerError::RuntimeError { message }) => message,
            _ => panic!("expected a runtime error"),
        }
    }

    #[tokio::test]
    async fn real_interpreter_runs_meters_and_confines_submissions() {
        let Some((runtime, home)) = runtime() else {
            return;
        };
        let python = Python::new(runtime, &home);
        let add = WasmFunctionCall::new(
            "add",
            vec![int(2), int(40)],
            FunctionType::Int(ContainerVariantType::Single),
        );
        let budget = Some(2_000_000_000);

        let (value, output, fuel) = run(
            &python,
            "import sys\ndef add(a, b):\n    print('sum', a + b)\n    print('err', file=sys.stderr)\n    return a + b\n",
            &add,
            budget,
        )
        .await
        .unwrap();
        assert_eq!(value, int(42));
        assert_eq!(output, "sum 42\nerr\n");
        // Interpreter startup and the first compile are not charged.
        assert!(fuel > 0 && fuel < 5_000_000, "unexpected fuel {fuel}");
        // Standard library imports load precompiled bytecode.
        let (_, _, fuel) = run(
            &python,
            "import collections, heapq, json, re\ndef add(a, b):\n    return a + b\n",
            &add,
            budget,
        )
        .await
        .unwrap();
        assert!(fuel < 200_000_000, "imports were not precompiled: {fuel}");

        // Containers, strings and floats arrive and return as native values.
        let stats = WasmFunctionCall::new(
            "stats",
            vec![
                FunctionValue::Double(ContainerVariant::List(vec![1.5, 2.5, f64::INFINITY])),
                FunctionValue::String(ContainerVariant::Grid(vec![vec!["é".into(), "b".into()]])),
                FunctionValue::Char(ContainerVariant::Single('x')),
                FunctionValue::Bool(ContainerVariant::Single(true)),
            ],
            FunctionType::String(ContainerVariantType::List),
        );
        let (value, _, _) = run(
            &python,
            "def stats(values, grid, letter, flag):\n    assert flag is True and values[2] == float('inf')\n    return [str(sum(values[:2])), grid[0][0] + letter]\n",
            &stats,
            budget,
        )
        .await
        .unwrap();
        assert_eq!(
            value,
            FunctionValue::String(ContainerVariant::List(vec!["4.0".into(), "éx".into()]))
        );
        let halves = WasmFunctionCall::new(
            "halves",
            vec![FunctionValue::Int(ContainerVariant::List(vec![1, 2]))],
            FunctionType::Double(ContainerVariantType::Grid),
        );
        let (value, _, _) = run(
            &python,
            "def halves(values):\n    return [[v / 2 for v in values], (1, float('-inf'))]\n",
            &halves,
            budget,
        )
        .await
        .unwrap();
        assert_eq!(
            format!("{value:?}"),
            "Double(Grid([[0.5, 1.0], [1.0, -inf]]))"
        );

        // Problems in the submission are reported against its own source.
        let error = run(&python, "def add(a, b):\n  return (a +\n", &add, budget).await;
        assert!(matches!(
            error,
            Err(RunnerError::CompilationError { diagnostics })
                if diagnostics[0].line == 2 && diagnostics[0].message.contains("never closed")
        ));
        let error = run(&python, "def other(a, b):\n  return a\n", &add, budget).await;
        assert!(matches!(
            error,
            Err(RunnerError::CompilationError { diagnostics })
                if diagnostics[0].message.contains("\"add\"")
        ));
        let text = message(run(&python, "def add(a, b):\n    return [a][b]\n", &add, budget).await);
        assert!(text.contains("File \"solution.py\", line 2, in add"));
        assert!(text.ends_with("IndexError: list index out of range"));
        let text = message(run(&python, "def add(a, b):\n    return 'x'\n", &add, budget).await);
        assert_eq!(text, "TypeError: add() must return int, but returned str");
        let text = message(
            run(
                &python,
                "def add(a, b):\n    return 2 ** a ** 6\n",
                &add,
                budget,
            )
            .await,
        );
        assert!(text.contains("32 bits"));
        let text = message(
            run(
                &python,
                "import sys\ndef add(a, b):\n    sys.exit(0)\n",
                &add,
                budget,
            )
            .await,
        );
        assert!(text.ends_with("SystemExit: 0"));
        let text = message(
            run(
                &python,
                "import os\ndef add(a, b):\n    os._exit(0)\n",
                &add,
                budget,
            )
            .await,
        );
        assert!(text.starts_with("Python exited without a result."));
        let text = message(run(&python, "import numpy\n", &add, budget).await);
        assert!(text.contains("ModuleNotFoundError"));

        // Fuel is enforced from the first submitted statement and cannot be refilled.
        for source in [
            "def add(a, b):\n    while True:\n        pass\n",
            "while True:\n    pass\n",
            "import os\ndef add(a, b):\n    while True:\n        os.sched_yield()\n",
        ] {
            let text = message(run(&python, source, &add, Some(300_000_000)).await);
            assert!(text.contains("fuel"), "unexpected error: {text}");
        }
        let (_, _, fuel) = run(
            &python,
            "import os\ndef add(a, b):\n    total = 0\n    for i in range(200000):\n        total += i\n        os.sched_yield()\n    return 42\n",
            &add,
            Some(5_000_000_000),
        )
        .await
        .unwrap();
        assert!(fuel > 100_000_000, "yielding hid fuel use: {fuel}");

        // The wall-clock deadline stops code that still has fuel.
        let deadline = tokio::time::Instant::now() + Duration::from_millis(1500);
        let stopped = python
            .run(
                "def add(a, b):\n    while True:\n        pass\n".into(),
                add.clone(),
                None,
                deadline,
                "timeout",
            )
            .await;
        assert!(matches!(stopped, Err(RunnerError::TimeoutError { .. })));

        // Memory is capped, and the guest sees only a read-only standard library.
        let text = message(
            run(
                &python,
                "def add(a, b):\n    return len(bytearray(1 << 30))\n",
                &add,
                budget,
            )
            .await,
        );
        assert!(text.ends_with("MemoryError"), "unexpected error: {text}");
        for (statement, expected) in [
            ("open('/lib/python3.14/acm-test', 'w')", "Error"),
            ("os.remove('/lib/python3.14/os.py')", "Error"),
            ("os.mkdir('/lib/acm-test')", "Error"),
            ("open('/etc/passwd')", "FileNotFoundError"),
            ("open('/proc/self/environ')", "FileNotFoundError"),
            ("open('/lib/../etc/passwd')", "Error"),
            ("os.listdir('/tmp')", "FileNotFoundError"),
            ("os.listdir('/')", "FileNotFoundError"),
            ("__import__('socket').socket()", "Error"),
            ("__import__('subprocess').run(['ls'])", "Error"),
            ("__import__('time').sleep(600)", "OSError"),
        ] {
            let source = format!("import os\ndef add(a, b):\n    {statement}\n    return 0\n");
            let text = message(run(&python, &source, &add, budget).await);
            assert!(
                text.lines().last().unwrap().contains(expected),
                "{statement}: {text}"
            );
        }
        assert!(!home.join("lib/python3.14/acm-test").exists());
        assert!(!home.join("lib/acm-test").exists());
        let (value, output, _) = run(
            &python,
            "import os, sys\ndef add(a, b):\n    print(sorted(os.environ), sys.argv, os.listdir('/lib'))\n    return len(open('/lib/python3.14/os.py').read(10))\n",
            &add,
            budget,
        )
        .await
        .unwrap();
        assert_eq!(value, int(10));
        assert_eq!(output, "[] ['-c'] ['python3.14']\n");

        // A submission stops at its first failed test.
        let tests = [42, 0, 42, 0]
            .into_iter()
            .enumerate()
            .map(|(index, expected)| Test {
                id: index as i64,
                index: index as i64,
                max_fuel: Some(4),
                input: add.clone(),
                expected_output: int(expected),
            })
            .collect::<Vec<_>>();
        let job = |tests: Vec<Test>| SubmitJob {
            language: Language::Python,
            problem_id: 1,
            user_id: 1,
            implementation: "def add(a, b):\n    return a + b\n".into(),
            runtime_multiplier: None,
            tests,
            run_all: false,
            deadline_seconds: None,
        };
        let deadline = tokio::time::Instant::now() + Duration::from_secs(900);
        let response = python
            .run_tests(job(tests.clone()), deadline, "timeout")
            .await
            .unwrap();
        assert!(!response.passed);
        let ran: Vec<_> = response
            .tests
            .iter()
            .map(|t| (t.index, t.success))
            .collect();
        assert_eq!(ran, [(1, false), (0, true)]);
        let response = python
            .run_tests(
                job(vec![tests[0].clone(), tests[2].clone()]),
                deadline,
                "timeout",
            )
            .await
            .unwrap();
        assert!(response.passed && response.tests.len() == 2);
        assert_eq!(
            response.runtime,
            response.tests.iter().map(|t| t.fuel).sum::<i64>()
        );

        // The first run after a failure is unaffected.
        let (value, _, _) = run(&python, "def add(a, b):\n    return a + b\n", &add, budget)
            .await
            .unwrap();
        assert_eq!(value, int(42));
    }
}
