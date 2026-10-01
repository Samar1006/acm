#!/usr/bin/env python3
"""Real compiler/Wasmtime checks against a local, restricted Ramiel container.
Run: python3 scripts/test-rust-runner.py http://127.0.0.1:8082
Or:  python3 scripts/test-rust-runner.py docker://container-name
No API, database, browser, or authentication is mocked by this script.
"""
import json
from pathlib import Path
import subprocess
import sys
import urllib.request

url = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8082"


def post(route, body):
    if url.startswith("docker://"):
        result = subprocess.run([
            "docker", "exec", "-i", "--", url.removeprefix("docker://"),
            "curl", "--silent", "--show-error", "--fail", "--max-time", "370",
            "--header", "Content-Type: application/json", "--data-binary", "@-",
            "http://127.0.0.1:8082" + route,
        ], input=json.dumps(body).encode(), capture_output=True, check=True, timeout=380)
        response_bytes = result.stdout
    else:
        request = urllib.request.Request(url + route, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(request, timeout=380) as response:
            response_bytes = response.read()
    try:
        return json.loads(response_bytes)
    except json.JSONDecodeError as error:
        raise AssertionError(f"{route} returned invalid JSON") from error


def call(a=2, b=3, kind="Int"):
    return {"name": "add", "arguments": [{kind: {"Single": a}}, {kind: {"Single": b}}], "return_type": {kind: "Single"}}


def submit(source, language="rust", inputs=None, user_id=987654):
    inputs = inputs or [call(0, 0), call(), call(-4, 7), call(-5, -8), call(1000000, -1000000)]
    tests = []
    for i, value in enumerate(inputs):
        kind = next(iter(value["arguments"][0]))
        total = sum(argument[kind]["Single"] for argument in value["arguments"])
        tests.append({"id": i, "index": i, "max_fuel": 4, "input": value, "expected_output": {kind: {"Single": total}}})
    body = {"problem_id": 1, "user_id": user_id, "implementation": source, "tests": tests, "runtime_multiplier": None}
    if language == "rust":
        body["language"] = "rust"
    return post("/run/" + ("rust" if language == "rust" else "c++"), body)


cpp = 'extern "C" int add(int a, int b) { return a + b; }'
rust = 'fn add(a: i32, b: i32) -> i32 { a + b }'
for language, source in [("cpp", cpp), ("rust", rust)]:
    result = submit(source, language)
    assert result.get("Ok", {}).get("passed"), result
    print(language, "sample passed; fuel:", [test["fuel"] for test in result["Ok"]["tests"]])

result = submit('fn add(a: i64, b: i64) -> i64 { a + b }', inputs=[
    call(-(2**40), 2**40 + 7, "Long"),
    call(-(2**63), 2**63 - 1, "Long"),
    call(2**63 - 1, 0, "Long"),
])
assert result["Ok"]["passed"], result
assert submit('fn __acm_entry(a: i32, b: i32) -> i32 { a + b } fn add(a: i32, b: i32) -> i32 { __acm_entry(a, b) }')["Ok"]["passed"]
assert not submit('fn add(a: i32, b: i32) -> i32 { a - b }')["Ok"]["passed"]
result = submit('fn add(a: i32, b: i32) -> i32 { "wrong type" }')
assert result["Err"]["type"] == "CompilationError", result
assert any(d["line"] == 1 and d["diagnostic_type"] == "Error" for d in result["Err"]["diagnostics"])
assert submit(rust)["Ok"]["passed"], "compile failure poisoned later submissions"
result = submit('fn add(_: i32, _: i32) -> i32 { loop { std::hint::black_box(1); } }', inputs=[call()])
assert not result["Ok"]["passed"], result
assert "fuel" in (result["Ok"]["tests"][0]["error"] or "").lower(), result
print("fuel limit:", result["Ok"]["tests"][0]["error"])

# memory.grow is a single guest instruction, with no vector initialization loop.
# Both probes must return normally; arbitrary traps/fuel exhaustion cannot pass.
for pages, expected_denial in [(4096, False), (9600, True)]:  # 256 MiB / 600 MiB
    source = ('fn add(a: i32, b: i32) -> i32 { '
              f'let denied = core::arch::wasm32::memory_grow::<0>({pages}) == usize::MAX; '
              f'if denied == {str(expected_denial).lower()} {{ a + b }} else {{ -12345 }} }}')
    result = submit(source, inputs=[call()])
    assert result.get("Ok", {}).get("passed"), result
    assert result["Ok"]["tests"][0]["error"] is None, result
print("memory limit: 256 MiB growth allowed; 600 MiB growth denied")

body = {"language": "rust", "problem_id": 1, "user_id": 987654, "reference": cpp, "input": call(), "implementation": 'fn add(a: i32, b: i32) -> i32 { let values = vec![a, b]; println!("sum = {}", values.iter().sum::<i32>()); values.iter().sum() }', "runtime_multiplier": None}
result = post("/custom-input/rust", body)
assert result["Ok"]["result"]["success"], result
assert "sum = 5" in result["Ok"]["output"], result
print("standard library custom input passed; fuel:", result["Ok"]["result"]["fuel"])
list_call = {"name": "sum_list", "arguments": [{"Int": {"List": [2, 3, 5]}}], "return_type": {"Int": "Single"}}
body = {
    "language": "rust",
    "problem_id": 1,
    "user_id": 987654,
    "implementation": 'fn sum_list(values: Vec<i32>) -> i32 { values.iter().sum() }',
    "tests": [{"id": 0, "index": 0, "max_fuel": 4, "input": list_call, "expected_output": {"Int": {"Single": 10}}}],
    "runtime_multiplier": None,
}
result = post("/run/rust", body)
assert result.get("Ok", {}).get("passed"), result
print("list argument passed; fuel:", result["Ok"]["tests"][0]["fuel"])
string_call = {"name": "shout", "arguments": [{"String": {"Single": "hi"}}], "return_type": {"String": "Single"}}
body["implementation"] = 'fn shout(s: String) -> String { s.to_uppercase() }'
body["tests"] = [{"id": 0, "index": 0, "max_fuel": 4, "input": string_call, "expected_output": {"String": {"Single": "HI"}}}]
result = post("/run/rust", body)
assert result.get("Ok", {}).get("passed"), result
grid_call = {"name": "flatten", "arguments": [{"Int": {"Grid": [[1, 2], [3]]}}], "return_type": {"Int": "List"}}
body["implementation"] = 'fn flatten(rows: Vec<Vec<i32>>) -> Vec<i32> { rows.into_iter().flatten().collect() }'
body["tests"] = [{"id": 0, "index": 0, "max_fuel": 4, "input": grid_call, "expected_output": {"Int": {"List": [1, 2, 3]}}}]
result = post("/run/rust", body)
assert result.get("Ok", {}).get("passed"), result
print("string and grid round-trips passed")
body = {"language": "rust", "problem_id": 1, "user_id": 987654, "reference": cpp, "input": call(), "implementation": 'extern crate serde; fn add(a: i32, b: i32) -> i32 { a + b }', "runtime_multiplier": None}
assert post("/custom-input/rust", body)["Err"]["type"] == "CompilationError"


def assert_file_denied(result):
    assert result.get("Err", {}).get("type") == "CompilationError", result
    assert any("permission denied" in diagnostic["message"].lower()
               for diagnostic in result["Err"]["diagnostics"]), result


# These synthetic reference/peer sources must exist. Missing-file errors do not
# prove isolation. Both attacks worked before compiler filesystem restrictions.
body["implementation"] = 'fn add(a: i32, b: i32) -> i32 { let _reference = include_str!("../reference/implementation.cpp"); a + b }'
assert_file_denied(post("/custom-input/rust", body))
body["language"] = "cpp"
body["implementation"] = '#define add reference_add\n#define alloc reference_alloc\n#include "/tmp/acm/custom_input/987654/1/reference/implementation.cpp"\n#undef alloc\n#undef add\nextern "C" int add(int a, int b) { return reference_add(a, b); }'
assert_file_denied(post("/custom-input/c++", body))
peer_rust = 'fn add(a: i32, b: i32) -> i32 { let _peer = include_str!("/tmp/acm/rust/submissions/987654/1/implementation.rs"); a + b }'
assert_file_denied(submit(peer_rust, user_id=987655))
peer_cpp = '#define add peer_add\n#define alloc peer_alloc\n#include "/tmp/acm/submissions/987654/1/implementation.cpp"\n#undef alloc\n#undef add\nextern "C" int add(int a, int b) { return peer_add(a, b); }'
assert_file_denied(submit(peer_cpp, "cpp", user_id=987655))
assert submit(rust, user_id=987655)["Ok"]["passed"]
assert submit(cpp, "cpp", user_id=987655)["Ok"]["passed"]
print("PASS: compiler reference/peer-file isolation and recovery after access denial")
print("PASS: sample, i64, wrong output, diagnostics, stale output, fuel, memory, std, custom input, lists/strings/grids, and external crate rejection")

# Run the committed practice references, not a second copy of their algorithms.
for problem_id, name in enumerate([
    "digit-sum", "reverse-digits", "greatest-common-divisor", "fibonacci", "steps-to-zero",
], start=100):
    path = Path(__file__).resolve().parents[1] / "docs/examples/local-demo" / f"{name}.json"
    try:
        problem = json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise AssertionError(f"Cannot load practice fixture {name}") from error
    result = post("/run/c++", {
        "problem_id": problem_id,
        "user_id": 987654,
        "implementation": 'extern "C" {\n' + problem["reference"] + '\n}',
        "tests": [{**test, "id": test["index"]} for test in problem["tests"]],
        "runtime_multiplier": None,
    })
    assert result.get("Ok", {}).get("passed"), f"{name}: {result}"
    print("PASS: practice fixture", name)
