#!/usr/bin/env python3
"""Submit each problem's reference solution and confirm the graders agree.

This drives the real `POST /run/submit` path, so it exercises the compiler, the
sandboxed runner, and the stored expected outputs together. It mints a local
administrator token the same way `seed-problems.py` does and must only be used
against a local stack whose JWT_SECRET you own.

Usage:
    scripts/verify-problems.py --env-file .env 3:docs/examples/mvp/easy-*.json
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import pathlib
import sys
import time
import urllib.error
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent


def load_seed_helpers():
    """Reuse the token minting and dotenv parsing from the seeding script."""
    spec = importlib.util.spec_from_file_location("seed", HERE / "seed-problems.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Api:
    def __init__(self, base_url: str, token: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.token = token

    def request(self, path: str, payload: dict | None = None):
        request = urllib.request.Request(
            self.base_url + path,
            data=json.dumps(payload).encode() if payload is not None else None,
            headers={
                "Content-Type": "application/json",
                "Cookie": f"token={self.token}",
            },
            method="POST" if payload is not None else "GET",
        )
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                body = response.read().decode()
        except urllib.error.HTTPError as error:
            detail = error.read().decode(errors="replace").strip()
            raise SystemExit(f"{path} failed: {error.code} {detail}") from error
        return json.loads(body) if body.strip() else None


def run_submission(api: Api, problem_id: int, language: str, code: str, timeout: int):
    """Queue a submission and poll until the job reports a result."""
    job = api.request(
        "/run/submit",
        {"language": language, "problem_id": problem_id, "implementation": code},
    )
    job_id = job["id"]

    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        status = api.request(f"/run/check/{job_id}")
        if status is None:
            raise SystemExit(f"job {job_id} disappeared before reporting a result")
        if status.get("error"):
            return False, status["error"]
        if status.get("response"):
            return True, status["response"]
        time.sleep(1.5)
    raise SystemExit(f"job {job_id} did not finish within {timeout}s")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "targets",
        nargs="+",
        metavar="PROBLEM_ID:PATH",
        help="problem id and the JSON file holding its reference solution",
    )
    parser.add_argument("--env-file", type=pathlib.Path, default=pathlib.Path(".env"))
    parser.add_argument("--api-url", default="http://127.0.0.1:8081")
    parser.add_argument("--user-id", type=int, default=1)
    parser.add_argument("--timeout", type=int, default=300)
    arguments = parser.parse_args()

    seed = load_seed_helpers()
    api = Api(
        arguments.api_url,
        seed.mint_admin_token(
            seed.read_env_value(arguments.env_file, "JWT_SECRET"),
            arguments.user_id,
            3600,
        ),
    )

    failures = 0
    for target in arguments.targets:
        problem_id_text, _, path_text = target.partition(":")
        path = pathlib.Path(path_text)
        problem = json.loads(path.read_text())
        problem_id = int(problem_id_text)
        test_count = len(problem["tests"])

        ok, result = run_submission(
            api, problem_id, "cpp", problem["reference"], arguments.timeout
        )
        if not ok:
            print(f"FAIL {problem['title']}: reference rejected: {result}")
            failures += 1
            continue
        if not result.get("success"):
            print(
                f"FAIL {problem['title']}: reference did not pass all {test_count} "
                f"tests: {result.get('error')}"
            )
            failures += 1
            continue
        print(
            f"PASS {problem['title']}: reference passed {test_count} tests "
            f"(complexity {result.get('complexity')})"
        )

        # A solution that ignores its input must not pass, otherwise the stored
        # expected outputs are not actually discriminating.
        ok, result = run_submission(
            api, problem_id, "cpp", problem["template"], arguments.timeout
        )
        if ok and result.get("success"):
            print(f"FAIL {problem['title']}: the do-nothing template also passed")
            failures += 1
        else:
            print(f"     ...and the do-nothing template is correctly rejected")

    print("\nAll reference solutions verified." if not failures else f"\n{failures} failure(s).")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
