"""Check a reference solution: stress.py SLUG SAMPLE_DIR COUNT [GENERATOR_ARGS...]

Runs solutions/SLUG.cpp on the official samples, then compares it with the
literal brute/SLUG.py on COUNT small cases from random/SLUG.py, using the
site's answer checker (tokens, decimals within 1e-6).
"""
import pathlib, subprocess, sys, tempfile
HERE = pathlib.Path(__file__).resolve().parent; BIN = pathlib.Path(tempfile.mkdtemp())
slug, samples, count, extra = sys.argv[1], pathlib.Path(sys.argv[2]), int(sys.argv[3]), sys.argv[4:]
def is_decimal(t):
    try: float(t)
    except ValueError: return False
    return any(c in t for c in '.eE')
def match(want, got):
    w, g = want.split(), got.split()
    if len(w) != len(g): return False
    for a, b in zip(w, g):
        if a == b: continue
        if not (is_decimal(a) or is_decimal(b)): return False
        try: x, y = float(a), float(b)
        except ValueError: return False
        if not (abs(x - y) <= 1e-6 * max(1.0, abs(x))): return False
    return True
def build(name):
    src = HERE / 'solutions' / f'{slug}.cpp' if name == 'sol' else HERE / f'{name}.cpp'
    if not src.exists(): return None
    exe = BIN / f'{slug}-{name}'
    if not exe.exists() or exe.stat().st_mtime < src.stat().st_mtime:
        subprocess.run(['nice', 'g++', '-O2', '-std=c++20', '-o', exe, src], check=True)
    return [str(exe)]
sol = build('sol')
brute = ['python3', '-I', str(HERE / 'brute' / f'{slug}.py')] if (HERE / 'brute' / f'{slug}.py').exists() else None
run = lambda cmd, inp: subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=120)
bad = 0
for f in sorted(samples.glob('*.in')):
    out = run(sol, f.read_text())
    ok = out.returncode == 0 and match(f.with_suffix('.ans').read_text(), out.stdout)
    bad += not ok; print(f'sample {f.name}: {"ok" if ok else "FAIL"}')
    if brute:
        b = run(brute, f.read_text()); okb = match(f.with_suffix('.ans').read_text(), b.stdout)
        bad += not okb; print(f'  brute on sample: {"ok" if okb else "FAIL"}')
if brute and (HERE / 'random' / f'{slug}.py').exists():
    for seed in range(count):
        inp = run(['python3', '-I', str(HERE / 'random' / f'{slug}.py'), str(seed), *extra], '').stdout
        a, b = run(sol, inp), run(brute, inp)
        if a.returncode or b.returncode or not match(b.stdout, a.stdout):
            bad += 1; print('MISMATCH seed', seed); print(inp[:600]); print('sol:', a.stdout[:300], a.stderr[:200]); print('brute:', b.stdout[:300], b.stderr[:200]); break
    else: print(f'{count} random cases agree')
sys.exit(1 if bad else 0)
