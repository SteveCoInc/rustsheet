#!/usr/bin/env python3
"""The one GitHub client of the repository's scripts (decision 0148).

`Client` builds the requests; `make catch-up` and `make renumber` read the PR with it (the merge
lock, readiness handoff and merge train that made the other requests were deleted, decision 0247,
T-3827, and their request builders with them, T-3866). `GhApi` sends them with `gh api` (REST
only: the session proxy refuses GraphQL), and `FakeGh` answers them from a table for tests. Both
subclasses share the request building, so a test of the fake exercises the same request shapes the
real client sends (scripts/test_gh_client.py).

Every value that lands in a URL is validated first (PR numbers, branch names, owner and repository
names), so no caller-supplied text can reach another API path.
"""

from __future__ import annotations

import json
import re
import subprocess
import urllib.parse
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# `gh api` gets this long per request; a PR read takes well under that.
TIMEOUT_SECONDS = 30

BRANCH = re.compile(r"[A-Za-z0-9][A-Za-z0-9._/-]{0,199}")
# Owner, repository and input names; a name of dots only (`..`) is refused, it would change the path.
NAME = re.compile(r"(?!\.+$)[A-Za-z0-9_.-]{1,100}")
REMOTE_URL = re.compile(r"github\.com[:/]+([^/\s]+)/([^/\s]+?)(?:\.git)?/?$")


class GhError(Exception):
    """A GitHub request failed, or an argument was refused before it was sent."""


@dataclass(frozen=True)
class Request:
    """One REST call: `method` on `path` (relative to the API root), `query` pairs, a JSON body."""

    method: str
    path: str
    query: tuple[tuple[str, str], ...] = ()
    body: Mapping[str, object] | None = None

    def url(self) -> str:
        """The path with its query string, as `gh api` takes it."""
        return f"{self.path}?{urllib.parse.urlencode(self.query)}" if self.query else self.path


def checked(pattern: re.Pattern[str], value: str, what: str) -> str:
    """`value` if it is ASCII and matches `pattern` in full; GhError otherwise."""
    if not (isinstance(value, str) and value.isascii() and pattern.fullmatch(value)):
        raise GhError(f"refused {what}: {value!r}")
    return value


def number(value: int | str, what: str = "PR number") -> int:
    """`value` as a positive integer below 10^9 (ASCII digits only); GhError otherwise."""
    text = str(value)
    if not (text.isascii() and text.isdigit() and len(text) <= 9) or int(text) == 0:
        raise GhError(f"refused {what}: {value!r}")
    return int(text)


def branch_name(name: str) -> str:
    """A branch name safe in a URL path: `BRANCH`, and no empty, `.`-led or `.lock` segment, so
    no `..` can climb out of `git/refs/heads/`."""
    checked(BRANCH, name, "branch")
    if any(not part or part.startswith(".") or part.endswith(".lock") for part in name.split("/")):
        raise GhError(f"refused branch: {name!r}")
    return name


def as_dict(value: object) -> dict[str, object]:
    """`value` if it is a JSON object; GhError otherwise."""
    if not isinstance(value, dict):
        raise GhError(f"expected a JSON object, got {type(value).__name__}")
    return value


class Client:
    """The operations of SPEC-0027 section 4 on one repository; subclasses implement `send`."""

    def __init__(self, owner: str, repo: str) -> None:
        self.base = f"repos/{checked(NAME, owner, 'owner')}/{checked(NAME, repo, 'repository')}"

    def send(self, request: Request) -> object:
        raise NotImplementedError

    def call(
        self,
        method: str,
        path: str,
        query: Sequence[tuple[str, str]] = (),
        body: Mapping[str, object] | None = None,
    ) -> object:
        return self.send(Request(method, f"{self.base}/{path}", tuple(query), body))

    def pull(self, pr: int | str) -> dict[str, object]:
        return as_dict(self.call("GET", f"pulls/{number(pr)}"))


class GhApi(Client):
    """Sends requests with `gh api`, which carries the session's or CI's GitHub credentials."""

    @staticmethod
    def argv(request: Request) -> list[str]:
        command = [
            "gh",
            "api",
            "--method",
            request.method,
            "-H",
            "Accept: application/vnd.github+json",
            request.url(),
        ]
        return command + ["--input", "-"] if request.body is not None else command

    def send(self, request: Request) -> object:
        return self.parse(request, self.run(request, self.argv(request)))

    def run(self, request: Request, argv: list[str]) -> str:
        """`gh api`'s output for `argv`; GhError when it fails."""
        stdin = json.dumps(request.body) if request.body is not None else ""
        try:
            result = subprocess.run(
                argv,
                input=stdin,
                capture_output=True,
                text=True,
                timeout=TIMEOUT_SECONDS,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise GhError(f"gh api {request.method} {request.path}: {error.__class__.__name__}") from error
        if result.returncode != 0:
            detail = (result.stderr.strip().splitlines() or ["no output"])[0][:200]
            raise GhError(f"gh api {request.method} {request.path}: {detail}")
        return result.stdout

    @staticmethod
    def parse(request: Request, out: str) -> object:
        try:
            return json.loads(out) if out.strip() else None
        except ValueError as error:
            raise GhError(f"gh api {request.method} {request.path}: not JSON") from error


@dataclass
class Reply:
    """A canned answer of `FakeGh`: a JSON value, or the error to raise."""

    value: object = None
    error: str | None = None


class FakeGh(Client):
    """Answers requests from `replies`, keyed by (method, url); records every request it gets."""

    def __init__(
        self, owner: str = "o", repo: str = "r", replies: Mapping[tuple[str, str], Reply] | None = None
    ) -> None:
        super().__init__(owner, repo)
        self.replies = dict(replies or {})
        self.requests: list[Request] = []

    def send(self, request: Request) -> object:
        self.requests.append(request)
        reply = self.replies.get((request.method, request.url()))
        if reply is None:
            raise GhError(f"fake: no reply for {request.method} {request.url()}")
        if reply.error is not None:
            raise GhError(reply.error)
        return reply.value


def owner_repo(url: str) -> tuple[str, str] | None:
    """The (owner, repository) of a github.com remote URL, or None."""
    match = REMOTE_URL.search(url)
    return (match[1], match[2]) if match else None


def from_remote(root: Path = ROOT, remote: str = "origin") -> GhApi | None:
    """The real client for `remote`'s repository, or None when it is not a github.com remote."""
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "remote", "get-url", remote],
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    pair = owner_repo(result.stdout.strip())
    if pair is None:
        return None
    try:
        return GhApi(*pair)
    except GhError:
        return None


# GitHub objects are JSON; these read one field of one without trusting its shape.
def field_of(value: object, *keys: str) -> object:
    for key in keys:
        value = value.get(key) if isinstance(value, dict) else None
    return value


def text_of(value: object, *keys: str) -> str:
    found = field_of(value, *keys)
    return found if isinstance(found, str) else ""
