# Runs one test inside the WebAssembly sandbox. Ramiel passes this file to
# CPython with -c. The job arrives as JSON on stdin and exactly one result
# record leaves on file descriptor 2. Everything the solution prints goes to
# file descriptor 1.
import json
import os
import sys

MARK = "\x1eacm-result "
FILE = "solution.py"
EXACT = {"int": int, "bool": bool, "str": str}
MAX_ERROR = 8000


class Mismatch(Exception):
    pass


def emit(record):
    data = memoryview((MARK + json.dumps(record) + "\n").encode())
    while data:
        data = data[os.write(2, data):]
    os._exit(0)


def type_name(kind, depth):
    name = "str" if kind == "char" else kind
    for _ in range(depth):
        name = f"list[{name}]"
    return name


def decode(value, kind, depth):
    # Floats travel as text so that NaN and the infinities survive JSON.
    if kind != "float":
        return value
    if depth == 0:
        return float(value)
    return [decode(item, kind, depth - 1) for item in value]


def encode_scalar(value, kind):
    actual = type(value)
    if kind == "float":
        if actual is float or actual is int:
            return repr(float(value))
    elif kind == "char":
        if actual is str and len(value) == 1:
            return value
    elif actual is EXACT[kind]:
        return value
    raise Mismatch(value)


def encode(value, kind, depth):
    if depth == 0:
        return encode_scalar(value, kind)
    if type(value) not in (list, tuple):
        raise Mismatch(value)
    if depth > 1:
        return [encode(item, kind, depth - 1) for item in value]
    # Check a flat list without a Python-level loop when every item is exact.
    actual = set(map(type, value))
    if kind == "float":
        if actual <= {float, int}:
            return list(map(repr, map(float, value)))
    elif kind == "char":
        if actual <= {str} and set(map(len, value)) <= {1}:
            return list(value)
    elif actual <= {EXACT[kind]}:
        return list(value)
    return [encode_scalar(item, kind) for item in value]


def describe(error, source):
    lines = source.splitlines()
    frames = []
    trace = error.__traceback__
    while trace is not None:
        code = trace.tb_frame.f_code
        if code.co_filename == FILE:
            frames.append((trace.tb_lineno, code.co_name))
        trace = trace.tb_next
    skipped = len(frames) - 10
    if skipped > 0:
        frames = frames[:4] + [None] + frames[-6:]
    out = ["Traceback (most recent call last):"]
    for frame in frames:
        if frame is None:
            out.append(f"  ... {skipped} more calls ...")
            continue
        line, name = frame
        out.append(f'  File "{FILE}", line {line}, in {name}')
        if 0 < line <= len(lines):
            out.append("    " + lines[line - 1].strip())
    try:
        text = str(error)
    except Exception:
        text = ""
    name = type(error).__name__
    out.append(f"{name}: {text}" if text else name)
    return "\n".join(out)[:MAX_ERROR]


def main():
    job = json.loads(sys.stdin.buffer.read())
    source, name = job["source"], job["name"]
    arguments = [
        decode(value, kind, depth)
        for value, (kind, depth) in zip(job["args"], job["types"])
    ]
    kind, depth = job["returns"]
    del job

    sys.stderr = sys.stdout
    module = type(sys)("solution")
    module.__file__ = FILE
    sys.modules["solution"] = module
    # The first compile() call sets up the interpreter's syntax-tree types.
    # Pay for that here so it is not charged to the submission.
    compile("", FILE, "exec")

    # Ramiel starts metering fuel here. Everything below is charged to the
    # submission, including loading its module.
    os.sched_yield()

    try:
        code = compile(source, FILE, "exec")
    except SyntaxError as error:
        emit({"syntax": {
            "line": error.lineno or 0,
            "col": error.offset or 0,
            "message": f"{type(error).__name__}: {error.msg}",
        }})
    except Exception as error:
        emit({"syntax": {
            "line": 0,
            "col": 0,
            "message": f"{type(error).__name__}: {error}"[:MAX_ERROR],
        }})

    try:
        exec(code, module.__dict__)
        function = getattr(module, name, None)
        if not callable(function):
            emit({"missing": name})
        result = function(*arguments)
    except BaseException as error:
        emit({"error": describe(error, source)})

    try:
        value = encode(result, kind, depth)
    except Mismatch as error:
        found = error.args[0]
        actual = "None" if found is None else type(found).__name__
        where = "returned" if found is result else "the returned value contains"
        expected = type_name(kind, depth)
        if kind == "char":
            expected += " (single characters)"
        emit({"error": f"TypeError: {name}() must return {expected}, but {where} {actual}"})
    except BaseException as error:
        emit({"error": describe(error, source)})
    emit({"value": value})


main()
