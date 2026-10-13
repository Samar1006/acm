#!/usr/bin/env python3
"""Real CPython-in-Wasmtime checks against a local, restricted Ramiel container.
Run: python3 scripts/test-python-runner.py http://127.0.0.1:8082
Or:  python3 scripts/test-python-runner.py docker://container-name
No API, database, browser, or authentication is mocked by this script.
"""
import json
import subprocess
import sys
import time
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


def call(a=2, b=3):
    return {"name": "add", "arguments": [{"Int": {"Single": a}}, {"Int": {"Single": b}}], "return_type": {"Int": "Single"}}


def test(index, value, expected, max_fuel=4):
    return {"id": index, "index": index, "max_fuel": max_fuel, "input": value, "expected_output": expected}


def submit(source, tests=None, user_id=987654):
    if tests is None:
        inputs = [call(0, 0), call(), call(-4, 7), call(-5, -8), call(1000000, -1000000)]
        tests = [test(i, value, {"Int": {"Single": sum(a["Int"]["Single"] for a in value["arguments"])}}) for i, value in enumerate(inputs)]
    return post("/run/python", {"language": "python", "problem_id": 1, "user_id": user_id, "implementation": source, "tests": tests, "runtime_multiplier": None})


def errors(result):
    return [item["error"] or "" for item in result["Ok"]["tests"]]


python = "def add(a, b):\n    return a + b\n"
started = time.monotonic()
result = submit(python)
assert result.get("Ok", {}).get("passed"), result
fuel = [item["fuel"] for item in result["Ok"]["tests"]]
print(f"python sample passed in {time.monotonic() - started:.1f}s (includes interpreter load); fuel:", fuel)
assert all(0 < value < 5_000_000 for value in fuel), fuel

# Lists, strings, and nested lists use native Python values in both directions.
groups = {"name": "group", "arguments": [{"String": {"List": ["b", "a", "b"]}}, {"Int": {"Grid": [[1, 2], [3]]}}], "return_type": {"String": "Grid"}}
source = "from collections import Counter\ndef group(words: list[str], rows: list[list[int]]) -> list[list[str]]:\n    counts = Counter(words)\n    return [[word] * counts[word] for word in sorted(counts)] + [[str(sum(map(sum, rows)))]]\n"
result = submit(source, [test(0, groups, {"String": {"Grid": [["a"], ["b", "b"], ["6"]]}})])
assert result.get("Ok", {}).get("passed"), result

# The standard library must load from the image's precompiled bytecode.
source = "import bisect, collections, functools, heapq, itertools, json, math, re, string\ndef add(a, b):\n    return a + b\n"
result = submit(source)
assert result.get("Ok", {}).get("passed"), result
assert max(item["fuel"] for item in result["Ok"]["tests"]) < 200_000_000, result

# Syntax errors and a missing function are compilation errors with source lines.
result = submit("def add(a, b):\n    return (a +\n")
assert result["Err"]["type"] == "CompilationError", result
assert result["Err"]["diagnostics"][0]["line"] == 2, result
result = submit("def subtract(a, b):\n    return a - b\n")
assert result["Err"]["type"] == "CompilationError" and '"add"' in result["Err"]["diagnostics"][0]["message"], result

# Exceptions, wrong return types, and missing packages fail tests with a traceback.
result = submit("def add(a, b):\n    return [a][b + 1]\n")
assert not result["Ok"]["passed"] and all('line 2, in add' in e and e.endswith("IndexError: list index out of range") for e in errors(result)), result
result = submit("def add(a, b):\n    return str(a + b)\n")
assert all(e == "TypeError: add() must return int, but returned str" for e in errors(result)), result
result = submit("import numpy\ndef add(a, b):\n    return a + b\n")
assert all("ModuleNotFoundError" in e for e in errors(result)), result
# A submission stops at its first failed test; later tests are not run or reported.
result = submit("def add(a, b):\n    return a + b if a >= 0 else 0\n")
assert not result["Ok"]["passed"] and [(item["index"], item["success"]) for item in result["Ok"]["tests"]] == [(2, False), (0, True), (1, True)], result
assert submit(python)["Ok"]["passed"], "a failed submission poisoned later submissions"

# Fuel stops loops, including loops at import time and loops that yield.
for source in [
    "def add(a, b):\n    while True:\n        pass\n",
    "while True:\n    pass\n",
    "import os\ndef add(a, b):\n    while True:\n        os.sched_yield()\n",
]:
    started = time.monotonic()
    result = submit(source)
    assert not result["Ok"]["passed"] and all("fuel" in e for e in errors(result)), result
    assert time.monotonic() - started < 120, "fuel limits did not stop the submission promptly"
    print("fuel limit stopped", repr(source.splitlines()[0]), f"in {time.monotonic() - started:.1f}s")

# Memory is capped below the container limit.
result = submit("def add(a, b):\n    return len(bytearray(1 << 30))\n")
assert all(e.endswith("MemoryError") for e in errors(result)), result

# The guest sees a read-only standard library: no host files, peers, network, or processes.
for statement, expected in [
    ("open('/etc/passwd')", "FileNotFoundError"),
    ("open('/proc/self/environ')", "FileNotFoundError"),
    ("os.listdir('/tmp')", "FileNotFoundError"),
    ("os.listdir('/')", "FileNotFoundError"),
    ("open('/lib/../etc/passwd')", "Error"),
    ("open('/lib/python3.14/acm-test', 'w')", "Error"),
    ("os.remove('/lib/python3.14/os.py')", "Error"),
    ("__import__('socket').socket()", "Error"),
    ("__import__('subprocess').run(['ls'])", "Error"),
    ("__import__('time').sleep(600)", "OSError"),
]:
    result = submit(f"import os\ndef add(a, b):\n    {statement}\n    return a + b\n")
    assert all(expected in e.splitlines()[-1] for e in errors(result)), (statement, result)
result = submit("import os\ndef add(a, b):\n    assert not os.environ and os.listdir('/lib') == ['python3.14']\n    return a + b\n")
assert result["Ok"]["passed"], result
print("python file, network, process, sleep, and environment access denied")

# Custom input compares with the C++ reference and returns printed output.
cpp = 'extern "C" int add(int a, int b) { return a + b; }'
body = {"language": "python", "problem_id": 1, "user_id": 987654, "reference": cpp, "input": call(), "implementation": "import sys\ndef add(a, b):\n    print('sum', a + b)\n    print('warn', file=sys.stderr)\n    return a + b\n", "runtime_multiplier": None}
result = post("/custom-input/python", body)
assert result["Ok"]["result"]["success"] and result["Ok"]["output"] == "sum 5\nwarn\n", result
body["implementation"] = "def add(a, b):\n    return a - b\n"
result = post("/custom-input/python", body)
assert not result["Ok"]["result"]["success"] and result["Ok"]["result"]["output"] == {"Int": {"Single": -1}}, result
body["implementation"] = "def add(a, b)\n"
assert post("/custom-input/python", body)["Err"]["type"] == "CompilationError"

# Signatures must be consistent and usable as Python identifiers.
mixed = [test(0, call(), {"Int": {"Single": 5}}), test(1, {**call(), "name": "other"}, {"Int": {"Single": 5}})]
assert submit(python, mixed)["Err"]["type"] == "UnsupportedSignature"
keyword = [test(0, {**call(), "name": "lambda"}, {"Int": {"Single": 5}})]
assert submit(python, keyword)["Err"]["type"] == "UnsupportedSignature"

assert submit(python, user_id=987655)["Ok"]["passed"]
print("python custom input, signature checks, and recovery passed")
