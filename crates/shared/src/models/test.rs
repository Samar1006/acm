use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use wasm_memory::{ContainerVariant, FunctionValue, WasmFunctionCall};

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct Test {
    #[serde(default)]
    pub id: i64,
    pub index: i64,
    pub max_fuel: Option<i64>,
    pub input: WasmFunctionCall,
    pub expected_output: FunctionValue,
}

impl FromRow<'_, SqliteRow> for Test {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let input: String = row.try_get("input")?;
        let expected_output: String = row.try_get("expected_output")?;

        Ok(Test {
            id: row.try_get("id")?,
            index: row.try_get("test_number")?,
            max_fuel: row.try_get("max_runtime")?,
            input: serde_json::from_str(&input).map_err(|e| sqlx::Error::ColumnDecode {
                index: "input".into(),
                source: Box::new(e),
            })?,
            expected_output: serde_json::from_str(&expected_output).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "expected_output".into(),
                    source: Box::new(e),
                }
            })?,
        })
    }
}

impl Test {
    pub fn adjust_runtime(&mut self, runtime_multiplier: Option<f64>) {
        self.max_fuel = self
            .max_fuel
            .map(|fuel| (fuel as f64 * runtime_multiplier.unwrap_or(1.5)) as i64);
    }

    /// Whether `output` answers this test. A whole program's output is judged
    /// like a contest answer; a function's return value must equal the expected one.
    pub fn accepts(&self, output: &FunctionValue) -> bool {
        match (self.input.stdin(), &self.expected_output, output) {
            (
                Some(_),
                FunctionValue::String(ContainerVariant::Single(expected)),
                FunctionValue::String(ContainerVariant::Single(actual)),
            ) => super::stdio::outputs_match(expected, actual),
            _ => output == &self.expected_output,
        }
    }

    pub fn make_result(self, output: FunctionValue, fuel: u64) -> TestResult {
        TestResult {
            id: self.id,
            index: self.index,
            success: self.accepts(&output),
            input: self.input,
            expected_output: self.expected_output,
            output: Some(output),
            error: None,
            max_fuel: self.max_fuel,
            fuel: fuel as i64,
            hidden: false,
        }
    }

    pub fn make_result_error(self, error: String, fuel: u64) -> TestResult {
        TestResult {
            id: self.id,
            index: self.index,
            success: false,
            input: self.input,
            expected_output: self.expected_output,
            output: None,
            error: Some(error),
            max_fuel: self.max_fuel,
            fuel: fuel as i64,
            hidden: false,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct TestResult {
    #[serde(default)]
    pub id: i64,
    pub index: i64,
    pub success: bool,
    pub input: WasmFunctionCall,
    pub expected_output: FunctionValue,
    pub output: Option<FunctionValue>,
    pub fuel: i64,
    pub error: Option<String>,
    pub max_fuel: Option<i64>,

    #[serde(default = "default_hidden")]
    pub hidden: bool,
}

fn default_hidden() -> bool {
    false
}

impl Serialize for TestResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.hidden {
            let mut s = serializer.serialize_struct("TestResult", 4)?;
            s.serialize_field("id", &self.id)?;
            s.serialize_field("index", &self.index)?;
            s.serialize_field("success", &self.success)?;
            s.serialize_field("error", &self.error)?;
            s.end()
        } else {
            let mut s = serializer.serialize_struct("TestResult", 9)?;
            s.serialize_field("id", &self.id)?;
            s.serialize_field("index", &self.index)?;
            s.serialize_field("success", &self.success)?;
            s.serialize_field("input", &self.input)?;
            s.serialize_field("expected_output", &self.expected_output)?;
            s.serialize_field("output", &self.output)?;
            s.serialize_field("fuel", &self.fuel)?;
            s.serialize_field("error", &self.error)?;
            s.serialize_field("max_fuel", &self.max_fuel)?;
            s.end()
        }
    }
}

impl TestResult {
    pub fn adjust_runtime(&mut self, runtime_multiplier: Option<f64>) {
        self.max_fuel = self
            .max_fuel
            .map(|fuel| (fuel as f64 * runtime_multiplier.unwrap_or(1.5)) as i64);
    }
}

impl Eq for TestResult {}

impl PartialOrd for TestResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TestResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl FromRow<'_, SqliteRow> for TestResult {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let input: String = row.try_get("input")?;
        let expected_output: String = row.try_get("expected_output")?;
        let output: String = row.try_get("output")?;

        Ok(TestResult {
            id: row.try_get("id")?,
            index: row.try_get("test_number")?,
            success: row.try_get("success")?,
            input: serde_json::from_str(&input).map_err(|e| sqlx::Error::ColumnDecode {
                index: "input".into(),
                source: Box::new(e),
            })?,
            expected_output: serde_json::from_str(&expected_output).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "expected_output".into(),
                    source: Box::new(e),
                }
            })?,
            output: serde_json::from_str(&output).map_err(|e| sqlx::Error::ColumnDecode {
                index: "output".into(),
                source: Box::new(e),
            })?,
            fuel: row.try_get("runtime")?,
            error: row.try_get("error")?,
            max_fuel: row.try_get("max_runtime")?,
            hidden: row.try_get("hidden")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(input: WasmFunctionCall, expected: FunctionValue) -> Test {
        Test {
            id: 0,
            index: 0,
            max_fuel: None,
            input,
            expected_output: expected,
        }
    }

    fn text(value: &str) -> FunctionValue {
        FunctionValue::String(ContainerVariant::Single(value.into()))
    }

    #[test]
    fn program_output_is_judged_by_tokens() {
        let program = test(
            WasmFunctionCall::program("3\nA3\nE3\nD3\n"),
            text("1.6666666666666667\n"),
        );
        assert!(program.accepts(&text("1.666667")));
        assert!(!program.accepts(&text("1.67")));
        assert!(program.clone().make_result(text("1.6666667\n"), 5).success);
    }

    #[test]
    fn function_results_still_need_equal_values() {
        let function = test(
            WasmFunctionCall::new(
                "greet",
                vec![],
                wasm_memory::FunctionType::String(wasm_memory::ContainerVariantType::Single),
            ),
            text("a b"),
        );
        assert!(function.accepts(&text("a b")));
        assert!(!function.accepts(&text("a  b")));
    }
}
