#!/usr/bin/env python3
"""Turn the 2026 ICPC Pacific Northwest problem sets into seedable problems.

The contest has published only statements and sample tests, so the hidden
tests here are generated (hidden_tests.py) and answered by the reference
solutions in solutions/. Each reference was checked against the slow, literal
solution in brute/ on thousands of random small cases (stress.py) and on every
small hidden test. Division 1 and 2 share seven problems; each appears once.

Eins, Zwei, Drei (D1 D / D2 I) is left out: it accepts any of many correct
outputs, and the site can only compare against one answer.

Usage:
    scripts/icpc-2026/import.py D1_PACKAGE D2_PACKAGE OUTPUT_DIR

The packages are the extracted PacNW2026Problems-1.zip and -2.zip from
https://pnw.na.icpc.global/problems/2026/real/index.html. Seed OUTPUT_DIR with
scripts/seed-problems.py.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
CONTEST = "2026 ICPC Pacific Northwest Regional"
PROGRAM_NOTE = (
    "Submit a complete C++ program. Read the input from standard input and "
    "print the answer to standard output."
)
GENERATED_NOTE = (
    "The contest has published only the sample tests so far. The hidden tests "
    "here were written by Chico ACM and checked against two independent solutions."
)
LOOSE_DECIMALS = (
    "This site accepts decimal answers within an absolute or relative error of "
    "`10^-6`, stricter than the contest. Print at least 9 digits after the decimal point."
)

# slug: (D1 letter, D2 letter, title, difficulty, tags, seconds, extra note, description)
PROBLEMS = {
    "bangaranga": (None, "A", "Bangaranga", "Easy", ["Math", "Simulation"], 1, None, """\
Dara is preparing her latest song, Bangaranga. The song will be played at a volume of `d` decibels, but Dara is unhappy with the volume and wants to test out various adjustments.

Decibels work on a logarithmic scale, so an increase from `d` decibels to `d + 10` decibels corresponds to being 10 times more intense. In general, an increase from `d` decibels to `d + 10k` decibels corresponds to being `10^k` times more intense. Symmetrically, a decrease from `d` decibels to `d - 10k` decibels corresponds to being `10^k` times less intense.

Dara's adjustments take two forms:

- `BANGA k`: make the current song `k` times more intense.
- `RANGA k`: make the current song `k` times less intense.

These changes are cumulative. After each change, Dara wants to know the volume of the song.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 50`), the number of test cases.

Each test case starts with a line containing two integers, `d` (`1 ≤ d ≤ 200`) and `q` (`1 ≤ q ≤ 50`): the initial volume in decibels and the number of adjustments.

The next `q` lines each describe one adjustment, in order. Each line is either `BANGA k` or `RANGA k`, where `k` (`10 ≤ k ≤ 10^9`) is an integer power of 10. Under these constraints the volume is always an integer number of decibels.

### Output

For each test case, for each adjustment, output the volume of the song in decibels after that adjustment."""),
    "cowtrain": (None, "B", "Cowtrain", "Easy", ["Strings", "Hash Tables"], 1, None, """\
Nick has been caught sending strange messages and claims his keyboard has been hacked: some keys, when pressed, produce different letters from the ones printed on them.

If Nick's keyboard really is hacked, two things are true:

- Every time Nick presses a given key, the same letter appears.
- No two different keys produce the same letter.

Given the string Nick wanted to type and the string he actually typed, is Nick's claim possible?

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 50`), the number of test cases.

Each test case consists of two lines. The first contains a string `s` (`1 ≤ |s| ≤ 50`) of lowercase letters, the string Nick wanted to type. The second contains a string `t` (`|t| = |s|`) of lowercase letters, the string he typed. The strings `s` and `t` are different.

### Output

For each test case, output `YES` if Nick's claim is possible, and `NO` otherwise."""),
    "inside-regions": (None, "D", "Inside Regions", "Easy", ["Math"], 1, None, """\
Audrey is practicing her calligraphy. She realizes that depending on how she writes the digit 4, it can have one enclosed region inside or none, and she starts to wonder how many enclosed regions each digit has.

She decides that the correct way to write each digit has this many enclosed regions:

| Digit | Enclosed regions |
| --- | --- |
| 0 | 1 |
| 1 | 0 |
| 2 | 0 |
| 3 | 0 |
| 4 | 1 |
| 5 | 0 |
| 6 | 1 |
| 7 | 0 |
| 8 | 2 |
| 9 | 1 |

Given a digit, determine how many enclosed regions it has.

### Input

The first and only line of input contains a digit `d` (`0 ≤ d ≤ 9`).

### Output

Output the number of enclosed regions the digit has."""),
    "iron-supplementation": (None, "E", "Iron Supplementation", "Easy", ["Math", "Simulation"], 1, LOOSE_DECIMALS, """\
Nick is deficient in iron, and his doctor has prescribed a supplement: he takes `m` milligrams of iron every morning for `d` days. Every night, `p` percent of the iron stored in his body disappears. Nick starts with zero milligrams of iron stored.

Compute the number of milligrams of iron Nick has stored after the night of the `d`-th day.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 50`), the number of test cases.

Each test case is a single line with three integers, `m`, `d`, and `p` (`1 ≤ m ≤ 50`, `1 ≤ d ≤ 50`, `1 ≤ p ≤ 99`).

### Output

For each test case, output the number of milligrams of iron stored after the night of the `d`-th day. In the contest, an answer with absolute or relative error at most `10^-3` was accepted."""),
    "split-one": (None, "F", "Split One", "Easy", ["Math"], 1, LOOSE_DECIMALS, """\
Nick's watch tracks his walk to work and normally tells him how long his first mile took. One day the notification never arrives, and he wants to work it out himself.

Nick starts his commute at time 0. His commute has `n` checkpoints, and between two consecutive checkpoints (and between the start and the first checkpoint) he walks at a constant speed.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 50`), the number of test cases.

Each test case starts with a line containing a single integer `n` (`1 ≤ n ≤ 50`), the number of checkpoints. The next `n` lines each describe a checkpoint in the order Nick reaches them, with two integers `d` and `t` (`1 ≤ d ≤ 10^4`, `1 ≤ t ≤ 10^4`): the checkpoint is `d` feet from the start, and Nick reaches it `t` seconds after starting.

The values of `d` are strictly increasing, the values of `t` are strictly increasing, and the last `d` is at least 5280 (the number of feet in a mile).

### Output

For each test case, output the number of seconds it took Nick to walk the first mile. In the contest, an answer with absolute error at most 1 was accepted."""),
    "floor-ceil-fiasco": ("E", "C", "Floor Ceil Fiasco", "Medium", ["Math", "Bit Manipulation"], 1, None, """\
Computers represent integers exactly but struggle with decimals, so many languages use integer operations whenever both operands are integers. Integer division is a problem when the numerator isn't divisible by the denominator: most languages round down (floor), but you could also round up (ceil).

You are given integers `n` and `k`. You must divide `n` by 2 a total of `k` times, and on each division you may choose to floor or ceil the result. Output the number of different integers you can end up with.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 50`), the number of test cases.

Each test case is a single line with two integers, `n` and `k` (`1 ≤ n ≤ 10^9`, `0 ≤ k ≤ 30`).

### Output

For each test case, output the number of possible resulting integers after any combination of `k` floors or ceils."""),
    "agentic-development": ("A", "G", "Agentic Development", "Hard", ["Probability", "Greedy", "Sorting"], 1, None, """\
You want to build a system with an AI agent, which works through it as `n` tasks. The tasks can be done in any order, but only one at a time. For each task you know how long the agent takes per attempt, and each attempt ends in exactly one of three ways:

1. The agent completes the task.
2. The agent fails, but no harm is done.
3. The agent irreversibly corrupts your system. You immediately wipe it clean and start over with no completed tasks.

For each task you know the probabilities of success, failure, and corruption. Attempts are independent, and wiping the system takes no time. You choose which task the agent attempts each time. What is the minimum expected time to complete all the tasks?

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 100`), the number of test cases.

Each test case starts with a line containing a single integer `n` (`1 ≤ n ≤ 100`), the number of tasks. The next `n` lines each describe a task with four integers `t`, `s`, `f`, and `c` (`1 ≤ t ≤ 1000`, `0 ≤ s, f, c ≤ 100`): each attempt takes time `t`, and it succeeds with probability `s/(s+f+c)`, fails with probability `f/(s+f+c)`, and corrupts the system with probability `c/(s+f+c)`. At least one of `s`, `f`, and `c` is positive.

### Output

For each test case, output the minimum expected time to complete all tasks, with relative error at most `10^-6`. If it is impossible to complete all tasks, output `impossible`."""),
    "birds-and-bread": ("B", "H", "Birds and Bread", "Hard", ["Geometry", "Math", "Sorting"], 4, None, """\
Kyle spends his Saturday afternoons throwing pieces of bread into a pond for the ducks. Each piece creates a circular ripple, and Kyle wants to know how many intersections there are between the ripples.

The pond is an infinite plane, and each piece of bread lands at integer coordinates `(x, y)`. A piece that lands at time `t` starts a ripple as a single point; the ripple's radius grows by one unit per second, so at time `t + k` it is a circle of radius `k` centered at `(x, y)`. A piece of bread thrown onto a ripple counts as 1 intersection. If several ripples meet at the same point, every pair counts as a separate intersection.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 200`), the number of test cases.

Each test case starts with a line containing two integers `n` (`1 ≤ n ≤ 2000`) and `q` (`1 ≤ q ≤ 10^5`): the number of pieces of bread and the number of times Kyle counts intersections.

The next `n` lines each contain three integers `t x y` (`0 ≤ t ≤ 10^9`, `-10^9 ≤ x, y ≤ 10^9`): a piece of bread lands at time `t` at `(x, y)`. The times are non-decreasing, and no two pieces land at the same place at the same time.

The next `q` lines each contain a single integer `t` (`0 ≤ t ≤ 10^9`), a time at which Kyle counts the intersections. These times are strictly increasing.

The sum of `n` over all test cases is at most 2000, and the sum of `q` is at most `10^5`.

### Output

For each time Kyle counts, output a line with the number of intersections at that time. The number of intersections is always finite."""),
    "pareto-optimal": ("I", "J", "Pareto Optimal", "Medium", ["Sorting"], 4, None, """\
Audrey runs a club whose members are each rated on mathematical skill and curiosity. She wants to send a representative to show off the club, and since the ratings are imprecise, she picks an integer `k`.

A person with skill `m_i` and curiosity `c_i` is *ineligible* if and only if some other person with skill `m_j` and curiosity `c_j` satisfies at least one of:

- `m_j ≥ m_i + k` and `c_j ≥ c_i`, or
- `m_j ≥ m_i` and `c_j ≥ c_i + k`.

List all people who are eligible.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 10^4`), the number of test cases.

Each test case starts with a line containing two integers `n` (`1 ≤ n ≤ 2 × 10^5`) and `k` (`1 ≤ k ≤ 10^9`). The next `n` lines each contain two integers `m` and `c` (`1 ≤ m, c ≤ 10^9`), the skill and curiosity of each person. People are numbered from 1 to `n` in input order, and no two people in a test case have the same skill and the same curiosity.

The sum of `n` over all test cases is at most `2 × 10^5`.

### Output

For each test case, output two lines: the number `p` of eligible people, then their numbers in increasing order. There is always at least one eligible person."""),
    "peppas-puddles": ("J", "K", "Peppa's Puddles", "Medium", ["Greedy", "Dynamic Programming"], 1, None, """\
Peppa the Pig loves jumping in puddles. There are `n` puddles in a line, numbered 1 to `n` from left to right, and she wants to jump in as many as possible, each at most once.

Jumping in a puddle may splash one or both of its neighbours. Once a puddle is splashed, Peppa can no longer jump in it, unless she already has; splashing a puddle she has already jumped in has no effect.

Find the maximum number of puddles Peppa can jump in if she chooses the best order.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 10^4`), the number of test cases.

Each test case starts with a line containing a single integer `n` (`2 ≤ n ≤ 2 × 10^5`). The next line is a binary string `sL` of length `n - 1`: its `i`-th character is `1` if jumping in puddle `i + 1` splashes puddle `i`. The next line is a binary string `sR` of length `n - 1`: its `i`-th character is `1` if jumping in puddle `i` splashes puddle `i + 1`.

The sum of `n` over all test cases is at most `2 × 10^5`.

### Output

For each test case, output the maximum number of puddles Peppa can jump in."""),
    "televote-resolver": ("L", "L", "Televote Resolver", "Medium", ["Math", "Greedy"], 2, None, """\
Gena is watching the International Country and Pop Contest. Each of `n` teams gets points from a jury and from a televote. The jury points are already public. The televote points are announced team by team, in nondecreasing order of jury points.

The winner is the team with the largest total of jury and televote points; if several teams tie for the largest total, one of them is picked at random.

Gena knows that exactly `p` televote points are given in total, but not how they are split until each team's result is announced. What is the smallest `c ≥ 0` such that, after hearing the televote results of the first `c` teams, Gena can be sure who has won?

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 1000`), the number of test cases.

Each test case starts with a line containing two integers `n` (`2 ≤ n ≤ 2 × 10^5`) and `p` (`1 ≤ p ≤ 10^18`). The next `n` lines each contain two integers `j` and `t` (`0 ≤ j ≤ 10^18`, `0 ≤ t ≤ p`): the team's jury points and televote points, in announcement order. The values of `j` are nondecreasing, and the values of `t` sum to exactly `p`.

The sum of `n` over all test cases is at most `2 × 10^5`.

### Output

For each test case, output the smallest `c` after which the winner is uniquely determined, or `-1` if the winner is not uniquely determined even after every result is announced."""),
    "tournament-rigging": ("M", "M", "Tournament Rigging", "Hard", ["Dynamic Programming", "Trees"], 1, None, """\
A single-elimination tournament has `2^k` players and a fixed bracket. For every pair of players you know who would win a fair match, and you know how much each player would need to be bribed to lose a match on purpose.

Before any match you may either do nothing, so the match is fair, or bribe one of the two players, who then loses. What is the minimum total you must spend on bribes to make sure player 1 wins the tournament?

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 1024`), the number of test cases.

Each test case starts with a line containing a single integer `k` (`1 ≤ k ≤ 11`): there are `2^k` players, numbered 1 to `2^k`. In the first round player 1 plays player 2, player 3 plays player 4, and so on; in the next round the winner of (1, 2) plays the winner of (3, 4), and so on.

The next `2^k` lines describe the players in order. Each contains an integer `b` (`1 ≤ b ≤ 10^6`), the cost to bribe this player, and a string `s` of `2^k` characters `0` and `1`, where the `i`-th character is `1` if and only if this player beats player `i` in a fair match. Each player's own character is `0`, and for every pair of distinct players exactly one beats the other.

The sum of `2^k` over all test cases is at most 2048.

### Output

For each test case, output the minimum total bribe needed for player 1 to win the tournament."""),
    "cable-management": ("C", None, "Cable Management", "Hard", ["Graphs", "Matching"], 4, None, """\
There are `n` data centers in the plane that you want to connect so that every pair can talk. You can lay down infinitely long axis-aligned lines (cables). A data center is attached to a cable if it lies on it, and two data centers are connected if you can follow a sequence of cables from one to the other; two cables that cross are connected at the crossing.

What is the minimum number of cables needed to connect every pair of data centers?

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 100`), the number of test cases.

Each test case starts with a line containing a single integer `n` (`2 ≤ n ≤ 10^5`). The next `n` lines each contain two integers `x` and `y` (`-10^9 ≤ x, y ≤ 10^9`), the location of a data center. No two data centers in a test case are at the same location.

The sum of `n` over all test cases is at most `10^5`.

### Output

For each test case, output the minimum number of cables needed."""),
    "hue-of-my-heritage": ("F", None, "Hue of My Heritage", "Hard", ["Trees", "Probability", "Dynamic Programming"], 2, None, """\
In one family, a rare genetic factor passes purple eyes down the generations. The family tree is rooted, and each child has a single primary ancestor: the one parent who may carry the gene.

- The root of the tree has purple eyes.
- Each child of a node with purple eyes independently has purple eyes with probability `1/2`.
- A node without purple eyes has no children with purple eyes.

Each level of the tree is a *generation*. Compute the expected number of generations with at least one purple-eyed node.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 10^4`), the number of test cases.

Each test case starts with a line containing a single integer `n` (`2 ≤ n ≤ 5 × 10^5`), the number of nodes; node 1 is the root. The next line contains `n - 1` integers `p_2, p_3, ..., p_n` (`1 ≤ p_i < i`), where `p_i` is the parent of node `i`.

The sum of `n` over all test cases is at most `5 × 10^5`.

### Output

For each test case, let `E` be the expected number of generations with at least one purple-eyed node. `E` can be written as `x/y` with `gcd(x, y) = 1` and `y` not divisible by 998 244 353. Output the integer `r` in `[0, 998 244 353)` with `r · y ≡ x (mod 998 244 353)`."""),
    "lake-viewing": ("G", None, "Lake Viewing", "Medium", ["Graphs", "Union Find"], 3, None, """\
Reggie likes walking around lakes. Each lake is a circle with waypoints around it, and bidirectional paths lead from waypoints to waypoints on other lakes.

Reggie wants a walk that goes around every lake exactly once, starting at the first waypoint of the first lake: each arc of each lake (between consecutive waypoints) must be walked exactly once, in either direction and not necessarily consecutively. A path may be walked any number of times in either direction, or not at all. Reggie can end anywhere.

Can Reggie walk around all the lakes?

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 10^4`), the number of test cases.

Each test case starts with a line containing two integers `n` and `m` (`2 ≤ n ≤ 2 × 10^5`, `0 ≤ m ≤ 2 × 10^5`), the number of lakes and paths. The next line contains `n` integers `a_1, ..., a_n` (`1 ≤ a_i ≤ 10^9`), where `a_i` is the number of waypoints around lake `i`, numbered counterclockwise.

The next `m` lines each contain four integers `l1`, `i1`, `l2`, and `i2` (`1 ≤ l1 < l2 ≤ n`, `1 ≤ i1 ≤ a_l1`, `1 ≤ i2 ≤ a_l2`): a path from waypoint `i1` of lake `l1` to waypoint `i2` of lake `l2`. No two paths in a test case connect the same pair of waypoints.

The sums of `n` and of `m` over all test cases are each at most `2 × 10^5`.

### Output

For each test case, output `YES` if Reggie can walk around every lake exactly once starting from waypoint 1 of lake 1, and `NO` otherwise."""),
    "not-regex": ("H", None, "Not Regex", "Hard", ["Strings", "Hashing", "Math"], 1, None, """\
You have finished your theory paper and want to appear early in a journal that sorts papers by a score, lowest first.

Your paper is a string `s` of lowercase letters. The value `v` of a letter is its position in the alphabet (`v(a) = 1`, ..., `v(z) = 26`). For each index `j`, let `f(j)` be the number of indices `k < j` whose letter comes strictly before `s_j` in the alphabet. The score is the sum of `v(s_j) · (f(j) + 1)` over all indices `j`.

To avoid changing too much, you do one find-and-replace with a chosen string `t`, where `n = |s|` and `m = |t|`:

1. Choose an index `i` (`1 ≤ i ≤ n - m + 1`), and let `p` be the substring `s_i ... s_(i+m-1)`.
2. Replace `s_i ... s_(i+m-1)` with `t`.
3. Find the smallest index `j ≥ i + m` such that `s_j ... s_(j+m-1)` equals `p`. If there is none, stop. Otherwise set `i` to `j` and go back to step 2.

Compute the minimum possible score after applying this exactly once.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 10^5`), the number of test cases.

Each test case is two lines: a string `s` (`1 ≤ |s| ≤ 2 × 10^5`) of lowercase letters, then a string `t` (`1 ≤ |t| ≤ |s|`) of lowercase letters.

The sum of `|s|` over all test cases is at most `2 × 10^5`.

### Output

For each test case, output the minimum possible score after applying the algorithm exactly once."""),
    "platform-nine": ("K", None, "Platform Nine", "Hard", ["Data Structures", "Sorting"], 3, None, """\
New wizards must find a magical barrier to board their train. Finding it takes effort, unless they arrive shortly after someone else passed through.

You choose how long the barrier stays visible after a wizard passes through: a positive integer `d`. When a wizard passes through at time `t`, the barrier is visible at every time `s` with `t ≤ s < t + d`. A wizard who arrives while the barrier is visible passes straight through; otherwise they search first (which takes no time) and then pass through. Muggles never pass through, but for security no Muggle may arrive while the barrier is visible.

You are given `n` people in order of arrival and `q` independent queries. In each query only people `l, l + 1, ..., r` come to the station, and you choose a single `d` for all of them. Among all values of `d` for which no Muggle arrives while the barrier is visible, find the minimum number of wizards who must search, and the largest `d` that achieves this minimum.

### Input

The first line of input contains a single integer `T` (`1 ≤ T ≤ 100`), the number of test cases.

Each test case starts with a line containing a single integer `n` (`1 ≤ n ≤ 3 × 10^5`). The next line is a string of `n` characters `W` and `M`: the `i`-th person is a wizard (`W`) or a Muggle (`M`). The next line contains `n` integers `t_1 < t_2 < ... < t_n` (`1 ≤ t_1`, `t_n ≤ 10^18`), the arrival times.

The next line contains a single integer `q` (`1 ≤ q ≤ 3 × 10^5`), and the next `q` lines each contain two integers `l` and `r` (`1 ≤ l ≤ r ≤ n`).

The sums of `n` and of `q` over all test cases are each at most `3 × 10^5`.

### Output

For each query, output two integers on one line: the minimum number of wizards who must search, and the largest `d` achieving it. If `d` can be arbitrarily large, output `-1` for `d`."""),
}

DIRECTORIES = {
    "agentic-development": "Agenticdevelopment", "bangaranga": "Bangaranga", "birds-and-bread": "Birdsandbread",
    "cable-management": "Cablemanagement", "cowtrain": "Cowtrain", "floor-ceil-fiasco": "Floorceilfiasco",
    "hue-of-my-heritage": "Hueofmyheritage", "inside-regions": "Insideregions",
    "iron-supplementation": "Ironsupplementation", "lake-viewing": "Lakeviewing", "not-regex": "Notregex",
    "pareto-optimal": "Paretooptimal", "peppas-puddles": "Peppaspuddles", "platform-nine": "Platformnine",
    "split-one": "Splitone", "televote-resolver": "Televoteresolver", "tournament-rigging": "Tournamentrigging",
}


def program_test(index: int, stdin: str, stdout: str) -> dict:
    return {
        "index": index,
        "max_fuel": None,
        "input": {"name": "main", "arguments": [{"String": {"Single": stdin}}], "return_type": {"String": "Single"}},
        "expected_output": {"String": {"Single": stdout}},
    }


def sample_dir(d1: pathlib.Path, d2: pathlib.Path, slug: str, letters: tuple) -> pathlib.Path:
    name = DIRECTORIES[slug]
    if letters[0]:
        return d1 / f"{letters[0]}-{name}-1" / "sample"
    return d2 / f"{letters[1]}-{name}-2" / "sample"


def footer(letters, seconds, note) -> str:
    where = " and ".join(
        f"problem {letter} of Division {division}"
        for division, letter in ((1, letters[0]), (2, letters[1])) if letter
    )
    unit = "second" if seconds == 1 else "seconds"
    lines = ["", "---", "", PROGRAM_NOTE]
    if note:
        lines += ["", note]
    lines += ["", GENERATED_NOTE, "",
              f"*{where[0].upper() + where[1:]} of the {CONTEST}. The contest time limit was {seconds} {unit}.*"]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("division1", type=pathlib.Path)
    parser.add_argument("division2", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    arguments = parser.parse_args()
    arguments.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as scratch:
        scratch = pathlib.Path(scratch)
        subprocess.run([sys.executable, "-I", str(HERE / "hidden_tests.py"), str(scratch / "hidden")], check=True)
        for slug, (d1, d2, title, difficulty, tags, seconds, note, description) in PROBLEMS.items():
            source = HERE / "solutions" / f"{slug}.cpp"
            binary = scratch / slug
            subprocess.run(["g++", "-O2", "-std=c++20", "-o", binary, source], check=True, capture_output=True)
            solve = lambda text: subprocess.run([binary], input=text, capture_output=True, text=True, check=True).stdout
            tests = []
            for f in sorted(sample_dir(arguments.division1, arguments.division2, slug, (d1, d2)).glob("*.in")):
                tests.append(program_test(len(tests), f.read_text(), f.with_suffix(".ans").read_text()))
            examples = len(tests)
            for f in sorted((scratch / "hidden" / slug).glob("*.in")):
                text = f.read_text()
                tests.append(program_test(len(tests), text, solve(text)))
            divisions = [f"Division {n}" for n, letter in ((1, d1), (2, d2)) if letter]
            problem = {
                "title": title,
                "description": description + footer((d1, d2), seconds, note),
                "reference": source.read_text(),
                "template": "",
                "runner": "stdio",
                "tags": tags + ["ICPC 2026"] + divisions,
                "examples": [{"test_number": i, "explanation": ""} for i in range(examples)],
                "hidden_tests": list(range(examples, len(tests))),
                "tests": tests,
            }
            name = f"{difficulty.lower()}-{slug}.json"
            (arguments.output / name).write_text(json.dumps(problem) + "\n")
            print(f"{name}: {examples} examples, {len(tests) - examples} hidden tests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
