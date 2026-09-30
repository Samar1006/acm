#!/usr/bin/env python3
"""Load problem JSON files into a local Chico ACM database through the real API.

This mints a short-lived administrator session token from the same JWT_SECRET the
API was started with, then uses the existing `POST /problems/new` and
`POST /problems/<id>/edit` endpoints. It is a convenience for local development
and single-operator setups where no Discord login has happened yet; it is not a
substitute for a real login, and it must never be pointed at a shared
deployment whose secret you do not own.

Problems are created in the order given on the command line. The home page
features the most recently created problem, so pass the problem you want
featured last. `create_dt` only has second granularity, so `--delay` spaces the
creations out far enough for that ordering to be unambiguous.

Usage:
    scripts/seed-problems.py --env-file .env docs/examples/mvp/hard-*.json ...
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import hmac
import json
import pathlib
import sys
import time
import urllib.error
import urllib.request

DIFFICULTIES = ("Easy", "Medium", "Hard")


def read_env_value(env_file: pathlib.Path, key: str) -> str:
    """Return `key` from a dotenv-style file without executing it."""
    for line in env_file.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        name, _, value = line.partition("=")
        if name.strip() != key:
            continue
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
            value = value[1:-1]
        return value
    raise SystemExit(f"{env_file}: {key} is not set")


def b64url(raw: bytes) -> str:
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode()


def mint_admin_token(secret: str, user_id: int, lifetime_seconds: int) -> str:
    """Build the same HS256 claims the server issues after a Discord login."""
    header = {"alg": "HS256", "typ": "JWT"}
    claims = {
        "user_id": user_id,
        "auth": "ADMIN",
        "exp": int(time.time()) + lifetime_seconds,
    }
    signing_input = ".".join(
        b64url(json.dumps(part, separators=(",", ":")).encode())
        for part in (header, claims)
    )
    signature = hmac.new(
        secret.encode(), signing_input.encode(), hashlib.sha256
    ).digest()
    return f"{signing_input}.{b64url(signature)}"


def post(api_url: str, path: str, token: str, payload: dict) -> object:
    request = urllib.request.Request(
        api_url.rstrip("/") + path,
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json", "Cookie": f"token={token}"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            body = response.read().decode()
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace").strip()
        raise SystemExit(f"POST {path} failed: {error.code} {detail}") from error
    except urllib.error.URLError as error:
        raise SystemExit(f"POST {path} failed: {error.reason}") from error
    return json.loads(body) if body.strip() else None


def difficulty_for(path: pathlib.Path, override: str | None) -> str:
    """Use an explicit difficulty, else infer it from the filename prefix."""
    if override:
        return override
    stem = path.stem.lower()
    for difficulty in DIFFICULTIES:
        if stem.startswith(difficulty.lower()):
            return difficulty
    raise SystemExit(
        f"{path.name}: cannot infer difficulty from the filename. "
        f"Name it easy-*.json, medium-*.json, or hard-*.json, or pass --difficulty."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("problems", nargs="+", type=pathlib.Path)
    parser.add_argument("--env-file", type=pathlib.Path, default=pathlib.Path(".env"))
    parser.add_argument("--api-url", default="http://127.0.0.1:8081")
    parser.add_argument("--user-id", type=int, default=1)
    parser.add_argument("--difficulty", choices=DIFFICULTIES)
    parser.add_argument("--token-lifetime", type=int, default=300)
    parser.add_argument("--delay", type=float, default=1.5)
    arguments = parser.parse_args()

    token = mint_admin_token(
        read_env_value(arguments.env_file, "JWT_SECRET"),
        arguments.user_id,
        arguments.token_lifetime,
    )

    for position, path in enumerate(arguments.problems):
        if position:
            time.sleep(arguments.delay)
        problem = json.loads(path.read_text())
        difficulty = difficulty_for(path, arguments.difficulty)

        created = post(arguments.api_url, "/problems/new", token, problem)
        problem_id = created["id"]

        # `POST /problems/new` has no difficulty field, so every problem starts as
        # Easy. The edit endpoint rewrites the columns it receives, so resend the
        # values from the file unchanged.
        post(
            arguments.api_url,
            f"/problems/{problem_id}/edit",
            token,
            {
                "title": problem["title"],
                "description": problem["description"],
                "template": problem["template"],
                "runtime_multiplier": problem.get("runtime_multiplier") or 1.0,
                "difficulty": difficulty,
                "visible": True,
            },
        )
        print(
            f"{path.name}: created problem {problem_id} "
            f"({difficulty}, {len(problem['tests'])} tests) -> /problems/{problem_id}"
        )

    return 0


if __name__ == "__main__":
    sys.exit(main())
