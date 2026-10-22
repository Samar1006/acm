#!/usr/bin/env python3
"""Turn the 2025 ICPC Pacific Northwest Division 2 package into seedable problems.

Each problem is solved with a complete C++ program that reads standard input
and writes standard output, judged against the official sample and secret data.
Sample tests become public examples; secret tests are hidden. The largest
secret sets are trimmed to a representative subset so a submission stays small.

Usage:
    scripts/import-icpc-2025.py PACKAGE_DIR OUTPUT_DIR

PACKAGE_DIR is the extracted PNW2025Solutions-2.zip from
https://pnw.na.icpc.global/problems/2025/solutions/index-2.html (SHA-256
55d2e9b13c3eaee66378129ad24533798c62391f8c5e2da3047403457b4cd844 when imported). Seed the
output with scripts/seed-problems.py once the server supports stdio problems.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

CONTEST = "2025 ICPC Pacific Northwest Regional, Division 2"
TAG = "ICPC 2025"
PROGRAM_NOTE = (
    "Submit a complete C++ program. Read the input from standard input and "
    "print the answer to standard output."
)

# Secret tests beyond this total size are trimmed, keeping the largest ones.
# Each problem is sent in one request, and the server accepts at most 2 MB.
MAX_SECRET_BYTES = 1_200_000


def footer(letter: str, seconds: int) -> str:
    unit = "second" if seconds == 1 else "seconds"
    return (
        f"\n\n---\n\n{PROGRAM_NOTE}\n\n"
        f"*Problem {letter} of the {CONTEST}. "
        f"The contest time limit was {seconds} {unit}.*"
    )


PROBLEMS = [
    {
        "directory": "A-Anelephantproblem-2",
        "letter": "A",
        "title": "An Elephant Problem",
        "difficulty": "Easy",
        "tags": ["Math"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
Gena is training an elephant to compete in the Iteratively Counting Peanuts Confrontation. As part of the training, Gena puts out `p` peanuts and the elephant has to count all of them.

However, elephants are not that good at counting peanuts, and Gena's elephant crushes some of the peanuts during the training. Gena is tasked with cleaning up the peanut debris that is left over. Gena cleans up `d` milligrams of peanut debris.

Gena knows that each peanut weighed exactly `m` milligrams, but forgot how many peanuts he put out for his elephant. Although there could be up to `m × p` milligrams of peanut debris, for various reasons the amount of debris could be much lower.

Help Gena compute the minimum number of peanuts he could have put out for the elephant!

### Input

The first and only line of input contains two integers, `m` and `d` (`1 ≤ m, d ≤ 50`).

### Output

Output a single integer, the minimum number of peanuts that Gena could have put out that could have resulted in his elephant leaving `d` milligrams of peanut debris.""",
    },
    {
        "directory": "C-Gpacomputation-2",
        "letter": "C",
        "title": "GPA Computation",
        "difficulty": "Easy",
        "tags": ["Math", "Simulation"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
Lydia wants to know if she is valedictorian!

Lydia's school computes the grade point averages of its students as follows. For each class that a student takes, they are assigned a letter grade from A to E. An A is worth 4 points, a B is worth 3 points, a C is worth 2 points, a D is worth 1 point, and an E is worth no points. The *unweighted* grade point average is the sum of these point values divided by the number of classes Lydia took.

To compute the *weighted* grade point average, each class is assigned a tier from 1 to 3. If a student gets an A, B, or C in a tier 1 class, they get an additive bonus of 0.05 points. If a student gets an A, B, or C in a tier 2 class, they get an additive bonus of 0.025 points. These are the only ways to get additive bonuses. The *weighted* grade point average is the *unweighted* grade point average plus all the additive bonuses.

Given Lydia's transcript, compute her weighted grade point average!

### Input

The first line of input contains a single integer, `n` (`1 ≤ n ≤ 50`).

Each of the next `n` lines contains a two-character string: the letter grade for one of the classes Lydia took, followed by the tier of the class. The first character is guaranteed to be one of `ABCDE` and the second one of `123`.

### Output

Output a single number, Lydia's weighted grade point average. Your answer will be considered correct if it has absolute or relative error at most `10^-6`.""",
    },
    {
        "directory": "D-Honkaistressrelief-2",
        "letter": "D",
        "title": "Honkai Stress Relief",
        "difficulty": "Medium",
        "tags": ["Math", "Probability"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
Audrey, having finally finished her last exam, decided to lie in bed and open her favorite video game, Honkai: Star Rail.

Over each of the next `n` days, Audrey has decided to carve out a block of time where she will play Honkai: Star Rail. This block of time may vary from day to day. Formally, if Audrey has carved out the interval `(s, e)` to play, she is considered to be playing at all times `t` where `s < t < e`.

Audrey's smartwatch tracks her stress level by doing a single check each day. If Audrey is playing at the instant when her smartwatch performs the check, it reports that she is relaxed. Otherwise, it reports that she is stressed. Audrey has configured her smartwatch to perform the check at some point within the time interval `(a, b)` each day. Formally, the smartwatch picks a value `t0` uniformly at random from all real values with `a < t0 < b`. If `s < t0 < e`, the watch reports that Audrey is relaxed; otherwise it reports that she is stressed. The value of `t0` is independent across days, but `a` and `b` are the same every day.

Audrey does not want her smartwatch to report that she is stressed. Compute the probability that there is at least one day where her smartwatch reports that she is stressed.

### Input

The first line of input contains three integers, `n`, `a`, and `b` (`1 ≤ n ≤ 20`, `1 ≤ a < b ≤ 50`).

Each of the next `n` lines contains two integers, `s` and `e` (`1 ≤ s < e ≤ 50`), meaning Audrey will play from time `s` to time `e`, exclusive, on that day.

### Output

Output a single number, the probability that there is at least one day where Audrey's smartwatch reports that she is stressed. Your answer will be accepted if its absolute or relative error is within `10^-6` of the judge's answer.""",
    },
    {
        "directory": "E-Missedalarm-2",
        "letter": "E",
        "title": "Missed Alarm",
        "difficulty": "Easy",
        "tags": ["Strings"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
It is November 14, 2025. You are doing your last-minute preparations for the 2025 Pacific Northwest Regional. You have Eurovision music blasting in your dorm room as you code up your final solution to Free Solo. You submit your solution to Kattis and watch as the green check marks slowly accumulate... you hit F5 one last time and see that all test cases have passed!

With your coding preparations complete, you decide to prepare all the materials that you will bring to the regional contest. You pack your trusty copies of CLRS and KT, along with language reference books for C, C++, Java, Kotlin, and Python. You print out a copy of the KACTL team reference document and also print your personal templates.

You check your phone. It is November 15, 2025. You realize you need to get a good night's sleep. You set your alarm and start preparing for bed. You're excited for the regional contest, but not so excited it'll keep you up tonight. With your alarm set to go off at some point on the 15th, you get into bed and you're asleep before your head hits the pillow.

You suddenly wake up and look at the clock. It's still November 15, 2025. Did you sleep through your alarm?

### Input

The input consists of two lines.

The first line contains a time in the form `HH:MM`, indicating when you set your alarm. It is guaranteed to be a valid time between `10:00` and `12:59`, inclusive.

The second line contains a time in the form `HH:MM`, indicating when you look at the clock. It is guaranteed to be a valid time between `10:00` and `12:59`, inclusive, and not exactly the same as your alarm.

A valid time always has a minute value that is an integer between 0 and 59, inclusive. The minute value is always written with two digits, possibly with a leading zero.

### Output

Output `YES` if you woke up after your alarm. Output `NO` otherwise.""",
    },
    {
        "directory": "F-Strawberry-2",
        "letter": "F",
        "title": "Strawberry",
        "difficulty": "Medium",
        "tags": ["Math", "Strings"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
How many `r`'s are there in the word `strawberry`? That's easy, the answer is obviously 3. But to LLMs the answer is not so obvious, and early models would count only 2.

To further trick LLMs, you decide to take a string `s` and repeat it `N` times. Each time it is repeated, you change each letter to the previous letter in the alphabet, with `a` instead becoming `z`. For example, the word `abacus` becomes `zazbtr`.

If we take the word `strawberry` and repeat it 3 times in total, we get the string `strawberryrsqzvadqqxqrpyuzcppw`. This string contains 5 occurrences of the letter `r`.

Given a word and the total number of times it is repeated, count the number of times `r` appears in the result!

### Input

The first line of input contains a string `s` of lowercase letters (`1 ≤ |s| ≤ 1000`).

The next line contains a single integer `N` (`1 ≤ N ≤ 10^15`), the total number of times `s` is repeated, including the initial string. Each letter is replaced by the one before it in each repetition.

### Output

Output the number of times `r` appears in the final string.""",
    },
    {
        "directory": "G-Bouquetofballoons-2",
        "letter": "G",
        "title": "Bouquet of Balloons",
        "difficulty": "Medium",
        "tags": ["Greedy", "Sorting", "Math"],
        "reference": "xiaowuc1.cpp",
        "seconds": 2,
        "description": """\
On their journey to World Finals, Team Shuchiin Academy first needs to conquer the North America Championship. They have `n` problems in the problem set, and problem `i` takes `s_i` minutes to solve. The team can solve problems in any order but cannot work on problems in parallel: they must finish the problem they are working on before moving on to another one. For each problem solved, the judges instantly bring the team a fully inflated balloon.

Chika Fujiwara, the troll of the team, decides to tie all the balloons to the team's lucky dice so that it floats. Each balloon starts with 1 liter of helium when the team receives it, which lets it lift 1 gram. All balloons have lifetime `d`. Balloons deflate at a constant rate of `1/d` liters per minute, and their lifting capacity in grams decreases at the same rate. For example, a balloon can lift `2/3` grams after `d/3` minutes, and stops at 0 grams after `d` minutes. The dice floats if, at any time, the sum of the lifting capacities of all the team's balloons is greater than or equal to the dice's mass `m`. The mass of the balloons themselves is negligible.

Given the solve times `s_1` through `s_n`, the balloon lifetime `d`, and the dice mass `m`, output the minimum number of problems the team needs to solve to float the dice! If the dice cannot be floated, output `-1`.

### Input

The first line of input contains three integers `n`, `d`, and `m` (`1 ≤ n, d, m ≤ 10^5`): the size of the problem set, the balloons' lifespan, and the mass of the dice.

The second line contains `n` integers: the solve times `s_1` through `s_n` (`1 ≤ s_i ≤ 10^5`).

### Output

Output a single integer: the minimum number of problems the team needs to solve to float the dice, or `-1` if it is impossible.""",
    },
    {
        "directory": "I-Servercorridor-2",
        "letter": "I",
        "title": "Server Corridor",
        "difficulty": "Medium",
        "tags": ["Greedy", "Strings"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
Audrey is the system administrator for miHoYo! It is her job to manage the server room and keep her favorite video game, Honkai: Star Rail, up and running.

The miHoYo server room is a single corridor of sections, some of which have servers installed. Some of the servers are already turned on.

Audrey urgently needs to turn on more servers. However, the server room has bad ventilation, so if two servers in adjacent sections are both on, the room will overheat.

Audrey cannot turn off servers that are already on. Compute the maximum number of servers Audrey can turn on, from the ones that are currently off, without causing the room to overheat.

### Input

The first line of input contains a single integer, `n` (`1 ≤ n ≤ 3 × 10^5`).

The second line contains a string `s` of length `n` made of the characters `0`, `1`, and `2`, representing the room. `0` is a section with no server, `1` is a section with a server that is off, and `2` is a section with a server that is on.

It is guaranteed that the server room is not currently overheating.

### Output

Output a single number: the number of additional servers Audrey can turn on from the ones that are currently off.""",
    },
    {
        "directory": "J-Shh-2",
        "letter": "J",
        "title": "Shh",
        "difficulty": "Hard",
        "tags": ["Dynamic Programming", "Strings", "Combinatorics"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "explanations": {
            0: "At least 5 characters must be changed. The four passwords that work are `shhovishhn`, `eshhvishhn`, `eushhishhn`, and `eurshhshhn`.",
        },
        "description": """\
Theo is a little overconfident. His Spotify password has just been leaked and he needs to change it. However, he likes to say his password out loud as he types it, so he changes it so that it has `k` different instances of the substring `shh`.

A string `b` is a *substring* of a string `a` if `b` can be obtained from `a` by deleting several (possibly zero or all) characters from the beginning and several (possibly zero or all) characters from the end. In particular, a string is a substring of itself. Two substrings are different instances if a different number of characters is deleted from the beginning, the end, or both, even if the resulting strings are the same.

Given the original password, compute the minimum number of characters Theo needs to change so that it has exactly `k` different instances of the substring `shh`. Also compute the number of distinct passwords Theo could construct by changing exactly this many characters that have exactly `k` different instances of `shh`.

### Input

The first line contains two integers, `n` and `k` (`1 ≤ n ≤ 67`, `0 ≤ 3k ≤ n`). The second line contains a string of `n` lowercase letters, Theo's original password.

### Output

Let `c` be the minimum number of characters Theo needs to change, and let `w` be the number of distinct passwords with exactly `k` different instances of the substring `shh` that can be obtained by changing exactly `c` characters. Output two integers: `c`, and the remainder when `w` is divided by the prime 67.""",
    },
    {
        "directory": "K-Solidarityofthehappycats-2",
        "letter": "K",
        "title": "Solidarity of the Happy Cats",
        "difficulty": "Hard",
        "tags": ["Greedy", "Simulation"],
        "reference": "xiaowuc1.cpp",
        "seconds": 1,
        "description": """\
Professor Pattengale has run into a huge problem: his robot is getting wrong digital signals, causing it to move around unpredictably! His circuitry students, the Happy Cats, conclude that the wires in the motherboard are too close together. As an electrical current passes through a wire, it creates a magnetic field that induces currents in other nearby wires, causing false signals.

The Engineering Family finds that the motherboard has `N` slots lined up in a row, and each slot has exactly one wire installed in it. Each wire has a range `r_i`, covering `r_i` slots both to the left and to the right of that wire; other wires within this range may be affected by its magnetic field. Each wire also has a type of signal `s_i` running through it.

There are `M` different types of signals passed through the wires. Certain types of signals can impact other types. For two different types `a` and `b`, if a signal of type `b` can impact a signal of type `a`, then a wire with signal type `a` cannot be within the range of any wire with signal type `b`; otherwise there will be interference. If type `b` does not impact type `a`, then wires with signal type `a` may be within the range of any wire with signal type `b`. Interference only happens when a signal of type `b` impacts a signal of type `a` and the wire with type `a` is within the range of the wire with type `b`.

A signal of type `a` may impact a signal of type `b` while type `b` does not impact type `a`. A signal of type `a` may also impact other wires of type `a`.

The Engineering Family wants to build a new motherboard for the wires. It will also be a single row of slots. All `N` wires must be installed in the same left-to-right order as on the old motherboard. At most one wire can be installed in a slot, and some slots may be left empty. There must be no interference.

Given the original layout of the `N` wires, compute the minimum number of slots in the new motherboard such that there is no interference.

### Input

The first line of input contains two positive integers, `N` and `M` (`1 ≤ M ≤ N ≤ 200`).

`N` lines follow, each with a pair of integers `r_i` and `s_i` (`1 ≤ r_i ≤ 10^9`, `1 ≤ s_i ≤ M`): the range and signal type of the `i`-th wire. The wires are given from left to right as they are installed in the old motherboard.

`M` lines follow, each containing a binary string of length `M`. The `i`-th line describes which signals impact signals of type `i`. If its `j`-th digit is `1`, signals of type `j` do not impact signals of type `i`. Otherwise, signals of type `j` do impact signals of type `i`.

### Output

Output a single integer: the minimum number of slots the new motherboard needs to avoid interference.""",
    },
    {
        "directory": "L-Traininground4-2",
        "letter": "L",
        "title": "Training, Round 4",
        "difficulty": "Medium",
        "tags": ["Greedy"],
        "reference": "xiaowuc1_greedy.cpp",
        "seconds": 1,
        "description": """\
T1 Ashley is training for another programming contest on Brandon's Online Judge. Brandon's Online Judge still has the feature that lets Ashley's coach, Tom, load in a list of problems for Ashley to work on.

Tom has curated some problems for Ashley. Each problem has an "implementation difficulty" and a "thinking difficulty", both positive integers. Ashley must solve them in order.

Ashley starts with a given implementation skill level and thinking skill level. She can solve a problem if and only if her implementation skill is at least the problem's implementation difficulty and her thinking skill is at least its thinking difficulty. After solving a problem, exactly one of her implementation skill and thinking skill increases by 1, and Ashley picks which one.

Compute the minimum possible sum of implementation and thinking skill levels that Ashley can start with such that she can solve all the problems on Tom's list in order.

### Input

The first line contains a single integer `n` (`1 ≤ n ≤ 50`).

The next `n` lines each contain two integers, `i` and `t` (`1 ≤ i, t ≤ 10^9`): the implementation and thinking difficulties of one of the problems on Tom's list. The problems are given in the order Ashley must solve them.

### Output

Output a single integer: the minimum possible sum of implementation and thinking skill levels that Ashley can start with such that she can solve all the problems on Tom's list in order.""",
    },
    {
        "directory": "M-Triangleoftriangles-2",
        "letter": "M",
        "title": "Triangle of Triangles",
        "difficulty": "Medium",
        "tags": ["Math", "Geometry"],
        "reference": "xiaowuc1.cpp",
        "seconds": 2,
        "description": """\
Audrey has a favorite triangle. Its angles are `a1`, `b1`, and `c1` degrees.

Ruan Mei only likes two types of triangles: a triangle whose angles are `a2`, `b2`, and `c2` degrees, or one whose angles are `a3`, `b3`, and `c3` degrees. The order of the angles does not matter.

Audrey and Ruan Mei only like multiples of 5, so every angle of all of these triangles is guaranteed to be divisible by 5.

Audrey wants to draw a line segment from one vertex of her favorite triangle to the opposite side, so that cutting the triangle along it divides it into two smaller triangles that Ruan Mei both likes. Is this possible?

### Input

The first line contains a single integer, `T` (`1 ≤ T ≤ 10^4`). `T` test cases follow, each spanning three lines.

- The first line contains three integers `a1`, `b1`, and `c1` (`5 ≤ a1 ≤ b1 ≤ c1 ≤ 170`, `a1 + b1 + c1 = 180`).
- The second line contains three integers `a2`, `b2`, and `c2` (`5 ≤ a2 ≤ b2 ≤ c2 ≤ 170`, `a2 + b2 + c2 = 180`).
- The third line contains three integers `a3`, `b3`, and `c3` (`5 ≤ a3 ≤ b3 ≤ c3 ≤ 170`, `a3 + b3 + c3 = 180`).

It is guaranteed that `a2 ≠ a3`, `b2 ≠ b3`, or `c2 ≠ c3`, and that all values are divisible by 5.

### Output

Output `T` lines. On the `i`-th line, output `YES` if Audrey can cut her favorite triangle exactly once into two triangles that Ruan Mei likes in the `i`-th test case. Otherwise, output `NO`.""",
    },
]


def case_numbers(directory: pathlib.Path) -> list[str]:
    def order(name: str):
        number = re.match(r"\d+", name)
        return (int(number.group()) if number else 0, name)

    return sorted((path.stem for path in directory.glob("*.in")), key=order)


def read_case(directory: pathlib.Path, name: str) -> tuple[str, str]:
    return (
        (directory / f"{name}.in").read_text(),
        (directory / f"{name}.ans").read_text(),
    )


def choose_secret(directory: pathlib.Path) -> list[str]:
    """Every secret case while they are small. Otherwise the largest cases, then
    small ones in order, up to MAX_SECRET_BYTES."""
    names = case_numbers(directory)
    size = {
        name: (directory / f"{name}.in").stat().st_size
        + (directory / f"{name}.ans").stat().st_size
        for name in names
    }
    if sum(size.values()) <= MAX_SECRET_BYTES:
        return names
    chosen: set[str] = set()
    total = 0
    by_size = sorted(names, key=lambda name: -size[name])
    for name in by_size[:3] + sorted(names, key=lambda name: size[name]):
        if name in chosen or total + size[name] > MAX_SECRET_BYTES:
            continue
        chosen.add(name)
        total += size[name]
    return [name for name in names if name in chosen]


def program_test(index: int, stdin: str, stdout: str) -> dict:
    return {
        "index": index,
        "max_fuel": None,
        "input": {
            "name": "main",
            "arguments": [{"String": {"Single": stdin}}],
            "return_type": {"String": "Single"},
        },
        "expected_output": {"String": {"Single": stdout}},
    }


def build(package: pathlib.Path, spec: dict) -> dict:
    root = package / spec["directory"]
    samples = case_numbers(root / "sample")
    secret = choose_secret(root / "secret")
    tests = [
        program_test(index, *read_case(root / "sample", name))
        for index, name in enumerate(samples)
    ]
    first_secret = len(tests)
    tests += [
        program_test(first_secret + index, *read_case(root / "secret", name))
        for index, name in enumerate(secret)
    ]
    explanations = spec.get("explanations", {})
    return {
        "title": spec["title"],
        "description": spec["description"] + footer(spec["letter"], spec["seconds"]),
        "reference": (root / "solutions" / spec["reference"]).read_text(),
        "template": "",
        "runner": "stdio",
        "tags": spec["tags"] + [TAG],
        "examples": [
            {"test_number": index, "explanation": explanations.get(index, "")}
            for index in range(len(samples))
        ],
        "hidden_tests": list(range(first_secret, len(tests))),
        "tests": tests,
    }


def slug(title: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    arguments = parser.parse_args()
    arguments.output.mkdir(parents=True, exist_ok=True)
    for spec in PROBLEMS:
        problem = build(arguments.package, spec)
        name = f"{spec['difficulty'].lower()}-{slug(spec['title'])}.json"
        (arguments.output / name).write_text(json.dumps(problem, indent=1) + "\n")
        samples = len(problem["examples"])
        hidden = len(problem["hidden_tests"])
        print(f"{name}: {samples} examples, {hidden} hidden tests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
