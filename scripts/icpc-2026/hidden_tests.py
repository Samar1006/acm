"""Write hidden test inputs for every problem: gen_hidden.py OUT_DIR.

Each problem gets small random tests (also checked against the brute force),
targeted edge cases, and large tests sized so a problem stays well under 2 MB.
"""
import pathlib
import random
import string
import sys

OUT = pathlib.Path(sys.argv[1])
R = random.Random(2026)


def emit(slug, name, lines, small=False):
    d = OUT / slug
    d.mkdir(parents=True, exist_ok=True)
    text = "\n".join(str(x) for x in lines) + "\n"
    (d / f"{name}{'-small' if small else ''}.in").write_text(text)


def cases(T, one):
    out = [T]
    for _ in range(T):
        out += one()
    return out


# ---------- Division 2 only ----------
def bangaranga():
    def tc(q=None, d=None, only=None):
        q = q or R.randint(1, 50)
        lines = [f"{d or R.randint(1, 200)} {q}"]
        for _ in range(q):
            lines.append(f"{only or R.choice(['BANGA', 'RANGA'])} {10 ** R.randint(1, 9)}")
        return lines
    for i in range(4):
        emit("bangaranga", f"{i:02}-random", cases(50, tc), small=True)
    emit("bangaranga", "10-all-up", cases(50, lambda: tc(50, 200, "BANGA")))
    emit("bangaranga", "11-all-down", cases(50, lambda: tc(50, 1, "RANGA")))
    emit("bangaranga", "12-single", [1, "1 1", "BANGA 10"])


def cowtrain():
    def tc():
        n = R.randint(1, 50)
        al = string.ascii_lowercase[:R.randint(1, 26)]
        s = "".join(R.choice(al) for _ in range(n))
        while True:
            kind = R.random()
            if kind < 0.45:
                perm = list(string.ascii_lowercase)
                R.shuffle(perm)
                t = "".join(dict(zip(string.ascii_lowercase, perm))[c] for c in s)
            elif kind < 0.7:
                perm = list(string.ascii_lowercase)
                R.shuffle(perm)
                t = list("".join(dict(zip(string.ascii_lowercase, perm))[c] for c in s))
                t[R.randrange(n)] = R.choice(string.ascii_lowercase)
                t = "".join(t)
            else:
                t = "".join(R.choice(al) for _ in range(n))
            if t != s:
                return [s, t]
    for i in range(5):
        emit("cowtrain", f"{i:02}-random", cases(50, tc), small=True)
    alpha = string.ascii_lowercase
    emit("cowtrain", "10-edges", [8, "a", "b", "ab", "aa", "aa", "ab", alpha, alpha[::-1],
                                  alpha + alpha[:24], alpha[1:] + "a" + alpha[1:25],
                                  "ab" * 25, "ba" * 25, "abc", "abd", "zzzz", "yyyy"])


def inside_regions():
    for d in range(10):
        emit("inside-regions", f"{d:02}-digit-{d}", [d], small=True)


def iron_supplementation():
    def tc():
        return [f"{R.randint(1, 50)} {R.randint(1, 50)} {R.randint(1, 99)}"]
    for i in range(4):
        emit("iron-supplementation", f"{i:02}-random", cases(50, tc), small=True)
    emit("iron-supplementation", "10-edges", [8, "1 1 1", "50 50 1", "50 50 99", "1 50 99",
                                              "50 1 99", "1 1 99", "37 50 50", "50 49 2"])


def split_one():
    def tc(n=None):
        n = n or R.randint(1, 50)
        while True:
            ds = sorted(R.sample(range(1, 10001), n))
            if ds[-1] >= 5280:
                break
        ts = sorted(R.sample(range(1, 10001), n))
        return [n] + [f"{d} {t}" for d, t in zip(ds, ts)]
    for i in range(4):
        emit("split-one", f"{i:02}-random", cases(50, tc), small=True)
    emit("split-one", "10-edges", [6, 1, "5280 1", 1, "10000 10000", 2, "5279 1", "5280 10000",
                                   2, "5280 7", "5281 8", 3, "1 9998", "2 9999", "10000 10000",
                                   2, "5000 5000", "10000 5001"])
    emit("split-one", "11-fifty", cases(50, lambda: tc(50)))


# ---------- shared and Division 1 ----------
def floor_ceil_fiasco():
    def tc(small):
        n = R.choice([R.randint(1, 10 ** 9), R.randint(1, 1000), 2 ** R.randint(0, 29),
                      2 ** R.randint(1, 29) - 1, 2 ** R.randint(1, 29) + 1, 10 ** 9, 1])
        return [f"{n} {R.randint(0, 14 if small else 30)}"]
    for i in range(3):
        emit("floor-ceil-fiasco", f"{i:02}-random-small-k", cases(50, lambda: tc(True)), small=True)
    for i in range(3):
        emit("floor-ceil-fiasco", f"10-random-{i}", cases(50, lambda: tc(False)))
    emit("floor-ceil-fiasco", "20-edges", [8, "1 0", "1 30", "1000000000 30", "1000000000 0",
                                           "536870912 29", "536870911 30", "536870913 30", "3 1"])


def agentic_development():
    def task(allow_zero_s):
        while True:
            s, f, c = (R.choice([0, R.randint(0, 100), 100, 1]) for _ in range(3))
            if not allow_zero_s and s == 0:
                s = R.randint(1, 100)
            if s + f + c > 0:
                return f"{R.randint(1, 1000)} {s} {f} {c}"
    def tc(nmax, impossible=0.05):
        n = R.randint(1, nmax)
        zero = R.random() < impossible
        return [n] + [task(zero) for _ in range(n)]
    for i in range(4):
        emit("agentic-development", f"{i:02}-random", cases(30, lambda: tc(5, 0.15)), small=True)
    for i in range(3):
        emit("agentic-development", f"10-large-{i}", cases(100, lambda: tc(100)))
    worst = [100] + ["1000 1 100 100"] * 100
    safe = [100] + [f"{R.randint(1, 1000)} {R.randint(1, 100)} {R.randint(0, 100)} 0" for _ in range(100)]
    emit("agentic-development", "20-edges", [4] + worst + safe + [1, "7 5 0 0", 2, "1 0 0 1", "1 1 0 0"])


def birds_and_bread():
    def small():
        n = R.randint(1, 7)
        V = R.choice([2, 4, 8])
        ts = sorted(R.randint(0, 10) for _ in range(n))
        seen, B = set(), []
        for t in ts:
            while True:
                p = (t, R.randint(-V, V), R.randint(-V, V))
                if p not in seen:
                    seen.add(p)
                    B.append(p)
                    break
        qs = sorted(R.sample(range(0, 30), R.randint(1, 8)))
        return [f"{n} {len(qs)}"] + [" ".join(map(str, b)) for b in B] + qs
    for i in range(4):
        emit("birds-and-bread", f"{i:02}-random", cases(20, small), small=True)
    trip = [(3, 4, 5), (5, 12, 13), (8, 15, 17), (7, 24, 25), (20, 21, 29), (12, 35, 37)]
    def big(n, q):
        B, seen = [], set()
        base = R.randint(0, 10 ** 8)
        for _ in range(n):
            while True:
                a, b, _ = R.choice(trip)
                s = R.randint(1, 20000000)
                x = R.choice([a * s, -a * s, R.randint(-10 ** 9, 10 ** 9), 10 ** 9, -10 ** 9])
                y = R.choice([b * s, -b * s, R.randint(-10 ** 9, 10 ** 9), 10 ** 9, -10 ** 9])
                t = base + R.randint(0, 10 ** 8)
                if (t, x, y) not in seen:
                    seen.add((t, x, y))
                    B.append((t, x, y))
                    break
        B.sort()
        qs = sorted(R.sample(range(base, 10 ** 9 + 1), q))
        return [f"{n} {q}"] + [" ".join(map(str, b)) for b in B] + qs
    emit("birds-and-bread", "10-big-coordinates", [3] + big(60, 300) + big(200, 2000) + big(5, 50))
    def grid(n, q, span):
        B, seen = [], set()
        for _ in range(n):
            while True:
                p = (R.randint(0, span), R.randint(-span, span), R.randint(-span, span))
                if p not in seen:
                    seen.add(p)
                    B.append(p)
                    break
        B.sort()
        qs = sorted(R.sample(range(0, max(4 * span, 3 * q)), q))
        return [f"{n} {q}"] + [" ".join(map(str, b)) for b in B] + qs
    emit("birds-and-bread", "11-dense-grid", [2] + grid(800, 5000, 60) + grid(400, 2000, 20))
    emit("birds-and-bread", "12-max-n", [1] + grid(2000, 20000, 3000))
    emit("birds-and-bread", "13-same-spot", [1, "4 5", "0 0 0", "1 0 0", "2 0 0", "3 0 0", 0, 1, 2, 3, 1000000000])


def cable_management():
    def small():
        V, W = R.randint(1, 6), R.randint(1, 6)
        cells = [(x, y) for x in range(1, V + 1) for y in range(1, W + 1)]
        if len(cells) < 2:
            cells = [(1, 1), (1, 2)]
        P = R.sample(cells, R.randint(2, min(len(cells), 12)))
        return [len(P)] + [f"{x} {y}" for x, y in P]
    for i in range(4):
        emit("cable-management", f"{i:02}-random", cases(40, small), small=True)
    def pts(P):
        R.shuffle(P)
        return [len(P)] + [f"{x} {y}" for x, y in P]
    def scale(P):
        xs = sorted({x for x, _ in P})
        ys = sorted({y for _, y in P})
        mx = dict(zip(xs, sorted(R.sample(range(-10 ** 9, 10 ** 9), len(xs)))))
        my = dict(zip(ys, sorted(R.sample(range(-10 ** 9, 10 ** 9), len(ys)))))
        return [(mx[x], my[y]) for x, y in P]
    full = [(x, y) for x in range(100) for y in range(100)]
    rect = [(x, y) for x in range(70) for y in range(110)]
    line = [(7, y) for y in range(5000)]
    rnd = list({(R.randint(0, 3000), R.randint(0, 3000)) for _ in range(5000)})
    blocks = [(b * 10 + x, b * 10 + y) for b in range(100) for x in range(10) for y in range(10)]
    emit("cable-management", "10-full-grid", [1] + pts(scale(full)))
    emit("cable-management", "11-rectangle", [1] + pts(scale(rect)))
    emit("cable-management", "12-mixed", [3] + pts(line) + pts(scale(rnd)) + pts(scale(blocks)))
    emit("cable-management", "13-edges", [5] + pts([(1, 1), (2, 2)]) + pts([(1, 1), (1, 2)])
         + pts([(1, 1), (2, 1), (3, 1)]) + pts([(1, 1), (1, 2), (2, 1), (2, 2), (3, 3)])
         + pts([(10 ** 9, -10 ** 9), (-10 ** 9, 10 ** 9)]))


def hue_of_my_heritage():
    def small():
        n = R.randint(2, 11)
        mode = R.random()
        return [n, " ".join(str(i - 1 if mode < 0.2 else R.randint(max(1, i - 2), i - 1) if mode < 0.5
                                else R.randint(1, i - 1)) for i in range(2, n + 1))]
    for i in range(4):
        emit("hue-of-my-heritage", f"{i:02}-random", cases(30, small), small=True)
    def tree(n, kind):
        if kind == "chain":
            return [i - 1 for i in range(2, n + 1)]
        if kind == "star":
            return [1] * (n - 1)
        if kind == "caterpillar":
            return [i - 1 if i % 2 == 0 else max(1, i - 2) for i in range(2, n + 1)]
        if kind == "binary":
            return [i // 2 for i in range(2, n + 1)]
        if kind == "shallow":
            return [R.randint(max(1, i - 3), i - 1) for i in range(2, n + 1)]
        return [R.randint(1, i - 1) for i in range(2, n + 1)]
    for kind, n in [("chain", 100000), ("star", 20000), ("binary", 20000), ("shallow", 20000),
                    ("random", 20000), ("caterpillar", 20000)]:
        emit("hue-of-my-heritage", f"10-{kind}", [1, n, " ".join(map(str, tree(n, kind)))])
    many = [1000]
    for _ in range(1000):
        n = R.randint(2, 40)
        many += [n, " ".join(map(str, tree(n, R.choice(["chain", "random", "shallow", "star"]))))]
    emit("hue-of-my-heritage", "20-many", many)


def lake_viewing():
    def small():
        n = R.randint(2, 4)
        A = [R.randint(1, 3) for _ in range(n)]
        while sum(A) > 8:
            A[R.randrange(n)] = 1
        cand = [(l1, i1, l2, i2) for l1 in range(1, n + 1) for l2 in range(l1 + 1, n + 1)
                for i1 in range(1, A[l1 - 1] + 1) for i2 in range(1, A[l2 - 1] + 1)]
        P = R.sample(cand, R.randint(0, min(len(cand), n + 1)))
        return [f"{n} {len(P)}", " ".join(map(str, A))] + [" ".join(map(str, p)) for p in P]
    for i in range(4):
        emit("lake-viewing", f"{i:02}-random", cases(40, small), small=True)
    def big(n, m, connected):
        A = [R.choice([1, 2, R.randint(1, 10 ** 9)]) for _ in range(n)]
        edges = set()
        order = list(range(1, n + 1))
        R.shuffle(order)
        cut = None if connected else R.randint(1, n - 1)
        if connected:
            for k in range(1, n):
                j = order[R.randrange(k)]
                edges.add((min(order[k], j), max(order[k], j)))
        left = set(order[:cut]) if cut else None
        tries = 0
        while len(edges) < m and tries < 50 * m + 100:
            tries += 1
            a, b = R.sample(range(1, n + 1), 2)
            if left is not None and ((a in left) != (b in left)):
                continue
            edges.add((min(a, b), max(a, b)))
        edges = [e for e in edges if e[0] != e[1]]
        P = {(a, R.randint(1, A[a - 1]), b, R.randint(1, A[b - 1])) for a, b in edges}
        return [f"{n} {len(P)}", " ".join(map(str, A))] + [" ".join(map(str, p)) for p in P]
    emit("lake-viewing", "10-connected", [1] + big(12000, 13000, True))
    emit("lake-viewing", "11-two-parts", [1] + big(12000, 13000, False))
    emit("lake-viewing", "12-edges", [4, "2 0", "1 1", "2 1", "1 1", "1 1 2 1", "3 1", "5 5 5",
                                      "1 3 2 3", "5 2", "1000000000 1 1 1 1000000000",
                                      "1 1000000000 5 1000000000", "2 1 3 1"])
    many = [2000]
    for _ in range(2000):
        n = R.randint(2, 30)
        many += big(n, R.randint(0, n + 3), R.random() < 0.5)
    emit("lake-viewing", "13-many", many)


def not_regex():
    def small():
        al = string.ascii_lowercase[:R.choice([1, 2, 3, 26])]
        n = R.randint(1, 25)
        if R.random() < 0.4:
            unit = "".join(R.choice(al) for _ in range(R.randint(1, 3)))
            s = (unit * n)[:n]
        else:
            s = "".join(R.choice(al) for _ in range(n))
        m = R.randint(1, min(n, 6))
        return [s, "".join(R.choice(string.ascii_lowercase[:R.choice([3, 26])]) for _ in range(m))]
    for i in range(4):
        emit("not-regex", f"{i:02}-random", cases(40, small), small=True)
    n = 200000
    emit("not-regex", "10-all-a", [1, "a" * n, "b"])
    emit("not-regex", "11-all-z-to-a", [1, "z" * n, "a"])
    emit("not-regex", "12-periodic", [1, ("abcab" * n)[:100000], "zz"])
    emit("not-regex", "13-random", [1, "".join(R.choice(string.ascii_lowercase) for _ in range(100000)),
                                    "".join(R.choice(string.ascii_lowercase) for _ in range(40))])
    emit("not-regex", "14-long-t", [1, "".join(R.choice("ab") for _ in range(100000)), "a" * 1000])
    many = [10000]
    for _ in range(10000):
        k = R.randint(1, 15)
        s = "".join(R.choice("abz") for _ in range(k))
        many += [s, "".join(R.choice("abyz") for _ in range(R.randint(1, k)))]
    emit("not-regex", "20-many", many)


def pareto_optimal():
    def small():
        n = R.randint(1, 9)
        V = R.randint(1, 8)
        pts = set()
        while len(pts) < n and len(pts) < V * V:
            pts.add((R.randint(1, V), R.randint(1, V)))
        pts = list(pts)
        R.shuffle(pts)
        return [f"{len(pts)} {R.randint(1, V)}"] + [f"{a} {b}" for a, b in pts]
    for i in range(4):
        emit("pareto-optimal", f"{i:02}-random", cases(60, small), small=True)
    def big(n, k, kind):
        pts = set()
        while len(pts) < n:
            if kind == "anti":
                x = R.randint(1, 10 ** 9)
                pts.add((x, 10 ** 9 + 1 - x))
            elif kind == "band":
                x = R.randint(1, 10 ** 6)
                pts.add((x, 10 ** 6 - x + R.randint(0, 50)))
            else:
                pts.add((R.randint(1, 10 ** 9), R.randint(1, 10 ** 9)))
        pts = list(pts)
        R.shuffle(pts)
        return [f"{n} {k}"] + [f"{a} {b}" for a, b in pts]
    emit("pareto-optimal", "10-random", [1] + big(14000, 10 ** 6, "random"))
    emit("pareto-optimal", "11-antichain", [1] + big(12000, 1, "anti"))
    emit("pareto-optimal", "12-band", [2] + big(10000, 30, "band") + big(3000, 10 ** 9, "random"))


def peppas_puddles():
    def small():
        n = R.randint(2, 7)
        q = R.random()
        b = lambda: "".join("1" if R.random() < q else "0" for _ in range(n - 1))
        return [n, b(), b()]
    for i in range(4):
        emit("peppas-puddles", f"{i:02}-random", cases(30, small), small=True)
    n = 200000
    emit("peppas-puddles", "10-all-ones", [1, n, "1" * (n - 1), "1" * (n - 1)])
    emit("peppas-puddles", "11-all-zeros", [1, 50000, "0" * 49999, "0" * 49999])
    q, n = 0.8, 100000
    emit("peppas-puddles", "12-random", [1, n, "".join("1" if R.random() < q else "0" for _ in range(n - 1)),
                                         "".join("1" if R.random() < q else "0" for _ in range(n - 1))])
    many = [10000]
    for _ in range(10000):
        k = R.randint(2, 18)
        many += [k, "".join(R.choice("01") for _ in range(k - 1)), "".join(R.choice("011") for _ in range(k - 1))]
    emit("peppas-puddles", "13-many", many)


def platform_nine():
    def small():
        n = R.randint(1, 7)
        s = "".join(R.choice("WM" if R.random() < 0.7 else "WWM") for _ in range(n))
        ts = sorted(R.sample(range(1, 30), n))
        q = R.randint(1, 6)
        lines = [n, s, " ".join(map(str, ts)), q]
        for _ in range(q):
            l = R.randint(1, n)
            lines.append(f"{l} {R.randint(l, n)}")
        return lines
    for i in range(4):
        emit("platform-nine", f"{i:02}-random", cases(40, small), small=True)
    def big(n, q, wiz, huge):
        s = "".join("W" if R.random() < wiz else "M" for _ in range(n))
        ts = set()
        while len(ts) < n:
            ts.add(R.randint(1, 10 ** 18) if huge else R.randint(1, 10 * n))
        ts = sorted(ts)
        lines = [n, s, " ".join(map(str, ts)), q]
        for _ in range(q):
            l = R.randint(1, n)
            lines.append(f"{l} {R.randint(l, min(n, l + R.choice([5, 100, n])))}")
        return lines
    emit("platform-nine", "10-huge-times", [1] + big(12000, 12000, 0.7, True))
    emit("platform-nine", "11-dense", [1] + big(20000, 20000, 0.8, False))
    emit("platform-nine", "12-mostly-muggles", [2] + big(10000, 10000, 0.1, False) + big(5000, 5000, 1.0, False))


def televote_resolver():
    def small():
        n = R.randint(2, 4)
        p = R.randint(1, 5)
        J = sorted(R.randint(0, 6) for _ in range(n))
        cuts = sorted(R.randint(0, p) for _ in range(n - 1))
        V = [b - a for a, b in zip([0] + cuts, cuts + [p])]
        R.shuffle(V)
        return [f"{n} {p}"] + [f"{a} {b}" for a, b in zip(J, V)]
    for i in range(4):
        emit("televote-resolver", f"{i:02}-random", cases(60, small), small=True)
    def big(n, p, jmax):
        J = sorted(R.randint(0, jmax) for _ in range(n))
        V = [0] * n
        for _ in range(min(n, 50)):
            V[R.randrange(n)] += 0
        rem = p
        for i in R.sample(range(n), min(n, 20)):
            give = R.randint(0, rem)
            V[i] += give
            rem -= give
        V[R.randrange(n)] += rem
        return [f"{n} {p}"] + [f"{a} {b}" for a, b in zip(J, V)]
    emit("televote-resolver", "10-large", [3] + big(15000, 10 ** 18, 10 ** 18) + big(5000, 1, 10)
         + big(3000, 10 ** 9, 10 ** 9))
    many = [1000]
    for _ in range(1000):
        many += big(R.randint(2, 12), R.choice([1, 5, 100, 10 ** 18]), R.choice([3, 100, 10 ** 18]))
    emit("televote-resolver", "11-many", many)
    top = 10 ** 18
    emit("televote-resolver", "12-edges", [4, f"2 {top}", f"0 {top}", f"{top} 0", f"2 {top}",
                                           f"{top} 0", f"{top} {top}", "3 3", "5 1", "5 1", "5 1",
                                           "2 1", "0 1", "0 0"])


def tournament_rigging():
    def bracket(k, mode):
        N = 1 << k
        W = [[0] * N for _ in range(N)]
        for i in range(N):
            for j in range(i + 1, N):
                win = (R.random() < 0.5) if mode == "random" else (i > j if mode == "player1-weakest" else i < j)
                if win:
                    W[i][j] = 1
                else:
                    W[j][i] = 1
        return [k] + [f"{R.randint(1, 10 ** 6 if k > 3 else 9)} {''.join(map(str, W[i]))}" for i in range(N)]
    for i in range(4):
        tests = [3]
        for _ in range(3):
            tests += bracket(R.randint(1, 3), "random")
        emit("tournament-rigging", f"{i:02}-random", tests, small=True)
    emit("tournament-rigging", "10-k9", [1] + bracket(9, "random"))
    emit("tournament-rigging", "11-weakest", [2] + bracket(8, "player1-weakest") + bracket(8, "random"))
    emit("tournament-rigging", "12-strongest", [1] + bracket(8, "player1-strongest"))


for fn in [bangaranga, cowtrain, inside_regions, iron_supplementation, split_one, floor_ceil_fiasco,
           agentic_development, birds_and_bread, cable_management, hue_of_my_heritage, lake_viewing,
           not_regex, pareto_optimal, peppas_puddles, platform_nine, televote_resolver, tournament_rigging]:
    fn()
print("done")
