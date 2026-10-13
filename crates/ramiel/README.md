# Ramiel

[Chico ACM](../../README.md) / **Runner**

[Setup](#running) · [Platforms](#docker-platform-support) · [Rust submissions](#rust-submissions) · [Python submissions](#python-submissions) · [Isolation](#compiler-filesystem-isolation) · [Limits](#limits)

Ramiel is the runner service for Chico ACM. It compiles C++ and Rust submissions to WebAssembly and executes them with Wasmtime. Python submissions run in a CPython interpreter that is itself WebAssembly. It also exposes `GET /healthz` for process health checks.

> [!IMPORTANT]
> The API uses Ramiel for submissions, custom input, and generated tests. Ramiel's HTTP endpoints are intended for the API, not for public exposure.

---

## Running

### Local Compose

Use the [complete local stack](../../README.md#docker-compose) for normal development. Run Compose commands from the repository root after you supply the required private environment values:

```sh
docker compose up --build
```

Ramiel has no published host port in this stack. In another terminal, check its health inside the container:

```sh
docker compose exec -T ramiel curl --fail http://127.0.0.1:8082/healthz
```

> [!NOTE]
> A successful health check proves that the process responds. Use the [compiler tests](#rust-submissions) to check compilation and execution.

### Host-native development

A host-native process requires all of these components:

- Native Linux amd64 or arm64 with Landlock ABI 3 or later.
- The static compiler helper at `/usr/local/libexec/acm-compiler-sandbox`.
- WASI SDK 27 at `/opt/wasi-sdk/bin/clang++`.
- Rust 1.92.0 with the `wasm32-wasip1` target at `/opt/submission-rust`.
- A Wasmtime cache configuration. The default path is `./wasmtime-cache.toml`.
- Optional, for Python: `python.wasm` and its `lib` directory at `/opt/acm-python`. See [Python submissions](#python-submissions). Without them, C++ and Rust still work and Python requests return an error.

Do not export the API's private environment into this process. From the repository root, run:

```sh
SQLX_OFFLINE=true cargo run --locked -p ramiel -- --hostname 127.0.0.1 --port 8082
```

Use `--hostname`, `--port`, `--wasmtime-cache-config`, and `--python-home` for the bind address, port, cache configuration, and Python directory. Their environment forms are `HOSTNAME`, `PORT`, `WASMTIME_CACHE_CONFIG`, and `ACM_PYTHON_HOME`. Set the bind address explicitly rather than depend on the shell's `HOSTNAME` value.

### Production

Use the [production operator guide](../../deploy/README.md), not the local Compose configuration. Production Compose keeps Ramiel on a private network and pins its platform to native amd64. The same Dockerfile supplies WASI SDK 27 for local and production builds. CPU emulation is unsupported.

## Docker platform support

Use `docker compose up --build` from the repository root with the required private `.env` values. See [local setup](../../docs/local-demo.md). The image includes checksum-verified WASI SDK 27, Rust 1.92.0 submission tools, and a checksum-verified CPython 3.14.7 WebAssembly build for amd64 and arm64. No host compiler or interpreter installation is required.

| Platform | Verification |
| --- | --- |
| Apple Silicon, Docker Engine 29.4.1, LinuxKit 6.12.76 | Native ARM64 C++/Rust execution, resource limits, file isolation, recovery, and five practice fixtures passed. |
| Native Linux amd64 | Real compiler/isolation tests run in the `runner-isolation` CI matrix. |
| Native Linux arm64 | Real compiler/isolation tests run in the `runner-isolation` CI matrix. |
| Docker Desktop on Windows or Intel Mac | Not yet tested directly. Use Linux containers on a matching CPU architecture. |

All platforms require Landlock ABI 3 or later under Docker's security policy. A kernel version alone does not prove support. The helper must deny access to an existing file before Ramiel can start. An unsupported kernel produces an error, not unrestricted compilation.

These instructions use source builds, not a prebuilt registry release.

## Rust submissions

The API accepts `language: "cpp"`, `language: "rust"`, or `language: "python"`. Old requests, saved drafts, and database rows default to C++.
Ramiel adds `/run/rust` and `/custom-input/rust`. Reference solutions and `/generate-tests/c++` remain C++.

Rust supports the same typed arguments and results as C++: scalars, `String`, `Vec<T>`, and `Vec<Vec<T>>` (lists and grids). All tests must use one function signature.
The editor provides a template from the problem's tests. For Coin Change:

```rust
fn coin_change(coins: Vec<i32>, amount: i32) -> i32 {
    // Return the answer.
    0
}
```

The standard library is available. The [WASI target](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html) has platform limits.
The guest has no filesystem access, network access, or application environment variables.
Cargo dependencies and build scripts are not supported.

The container includes Rust 1.92.0 and the `wasm32-wasip1` target at `/opt/submission-rust`.
Ramiel invokes `rustc` directly with fixed flags and a cleared environment.
A generated wrapper exports `acm_entry`. Scalars pass as Wasm parameters; strings and vectors use a compact encoding in linear memory, not C++ layouts.
Compiler diagnostics use the existing error display.

Rust uses the existing deadlines and memory limits. Its fuel budget is four times the adjusted C++ reference budget, with a 100,000-unit minimum.
The existing global fuel cap still applies. This is an initial allowance, not a cross-language performance comparison.
C++ fuel budgets remain unchanged.

After the default local Compose runner is healthy, run from the repository root:

```sh
python3 scripts/test-rust-runner.py docker://acm-local-ramiel-1
```

For a custom project name, use the container name from `docker compose ps`. For a supported host-native process, use `http://127.0.0.1:8082` instead of the `docker://` address.

The script checks real C++ and Rust compilation, Wasmtime execution, resource limits, file-access denial, and recovery. It does not replace a logged-in browser test. See [recovery and backup tests](../../docs/testing.md#isolated-failure-recovery) for the separate operational checks.

## Python submissions

Ramiel adds `/run/python` and `/custom-input/python`. Reference solutions and `/generate-tests/c++` remain C++.

A submission defines the problem's function at module level. The editor provides a template from the problem's tests. For Coin Change:

```python
def coin_change(coins: list[int], amount: int) -> int:
    # Return the answer.
    return 0
```

Arguments and results are plain Python values:

| Test type | Python type |
| --- | --- |
| `Int`, `Long` | `int` |
| `Float`, `Double` | `float` |
| `Bool` | `bool` |
| `Char` | one-character `str` |
| `String` | `str` |
| List | `list` |
| Grid, Graph | `list` of `list` |

The result must have exactly the declared type. A function may return a `tuple` where a `list` is expected, and an `int` where a `float` is expected. An `int` result must fit the test's 32-bit or 64-bit type. All tests must use one function signature, and the function name must be a valid Python identifier.

The interpreter is the [CPython 3.14.7 WASI build](https://github.com/brettcannon/cpython-wasi-build/releases/tag/v3.14.7) at `/opt/acm-python`. The Dockerfile verifies its checksum and compiles its standard library to bytecode with the same CPython release.
The standard library is available. Third-party packages such as NumPy are not. Threads, subprocesses, and sockets do not work on this platform, and `time.sleep` raises `OSError`.

There is no compiler process and no job directory. Ramiel loads the interpreter once and starts a new instance in Wasmtime for every test.
Each instance gets the source and arguments on standard input, an empty environment, and a read-only view of the standard library as `/lib`. It cannot see any other file.
Printed output, including `sys.stderr`, appears as the custom-input output. `input()` has nothing to read.

Syntax errors and a missing function are reported as compilation errors with the source line. Exceptions fail the test and show a traceback limited to the submission's own lines.

Python uses the existing deadlines and memory limit. Fuel is counted from the moment the submission's module starts to load, so interpreter startup is free but imports and top-level code are not.
The fuel budget is 250 times the adjusted C++ reference budget, with a 500,000,000-unit minimum that leaves room for standard library imports. A submitted test stops at four times its budget.
This is an initial allowance, not a cross-language performance comparison. Tests without a recorded budget are limited only by the request deadline, as in the other languages.

The first Python request after startup waits for the interpreter to compile, which takes about 15 seconds on one CPU core. Later requests reuse it.

After the default local Compose runner is healthy, run from the repository root:

```sh
python3 scripts/test-python-runner.py docker://acm-local-ramiel-1
```

The script checks real execution, typed values, error reporting, fuel and memory limits, and file, network, and process denial. It does not replace a logged-in browser test.

## Compiler filesystem isolation

Both compilers run through a small, statically linked helper. Python needs no compiler process and does not use it. The helper applies [Landlock filesystem restrictions](https://docs.kernel.org/userspace-api/landlock.html) before it executes the compiler. The restrictions also apply to linker subprocesses.

A compiler can read its own job directory, toolchain files, and required system libraries. It cannot read reference or peer submission directories. Temporary files stay in the job directory. Compiler environments are cleared. Existing process-group cleanup and deadlines remain active.

> [!CAUTION]
> Startup requires the native helper to pass a filesystem-denial self-check. If the helper is missing or cannot enforce Landlock ABI 3 or later, Ramiel refuses to start. There is no unrestricted fallback. Do not disable seccomp or add container privileges to bypass a failed check.

The build selects `x86_64-unknown-linux-musl` or `aarch64-unknown-linux-musl` for the native helper. That build target is not copied into the final image.

Rosetta emulation requires access to per-process files in `/proc`. This policy does not grant that access. The `runner-isolation` CI matrix runs the real compiler checks on native Ubuntu amd64 and arm64 instead.

The real runner test script also checks reference-source access, peer-source access, and recovery after denied access. A denied access must report a permission error, not a missing file.

## Limits

Ramiel applies these request deadlines:

| Request | Deadline |
| --- | ---: |
| Submission | 360 seconds |
| Test generation | 120 seconds |
| Custom input | 60 seconds |

A submission stops at its first failed test, in test order. Later tests are not run, so the response and the stored results contain only the tests up to and including that failure. Custom input and test generation are unaffected.

A `/run/<language>` job may set `run_all` to run every test regardless of failures, and `deadline_seconds` to use a deadline shorter than 360 seconds. The API uses both when it runs a problem's public examples.

Compose runs the container with a read-only root filesystem and an executable 512 MiB `/tmp` tmpfs. It limits the container to 2 CPUs, 2 GiB memory, and 256 processes. These controls do not guarantee that untrusted code is safe.

## Focused commands

```sh
SQLX_OFFLINE=true cargo check -p ramiel --locked
SQLX_OFFLINE=true cargo test -p ramiel --locked
```
