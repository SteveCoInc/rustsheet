"""Tests for scripts/gh_client.py: the fake and `gh api` build the same requests (SPEC-0027)."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from collections.abc import Callable
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gh_client  # noqa: E402 (the sys.path line above must run first)
from gh_client import Client, FakeGh, GhApi, GhError, Reply  # noqa: E402


# Recorded fixtures: each operation, the (method, url, body) it sends and the reply it parses.
FIXTURES: list[tuple[str, Callable[[Client], object], str, str, object, object]] = [
    ("pull", lambda c: c.pull(7), "GET", "repos/o/r/pulls/7", None, {"state": "open"}),
    # `call` is the one place a body is sent; no kept method has one, so it is checked directly.
    (
        "call with query and body",
        lambda c: c.call("POST", "issues/7/labels", [("a", "b c")], {"labels": ["x"]}),
        "POST",
        "repos/o/r/issues/7/labels?a=b+c",
        {"labels": ["x"]},
        [],
    ),
]


class SameRequests(unittest.TestCase):
    def test_the_fake_and_gh_api_send_each_fixture(self) -> None:
        for name, operation, method, url, body, reply in FIXTURES:
            with self.subTest(name):
                fake = FakeGh("o", "r", {(method, url): Reply(reply)})
                operation(fake)
                [request] = fake.requests
                self.assertEqual((request.method, request.url(), request.body), (method, url, body))

                calls: list[tuple[list[str], str]] = []

                def run(argv: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                    calls.append((argv, str(kwargs["input"])))
                    return subprocess.CompletedProcess(argv, 0, json.dumps(reply), "")

                with mock.patch("gh_client.subprocess.run", side_effect=run):
                    operation(GhApi("o", "r"))
                [(argv, stdin)] = calls
                accept = ["-H", "Accept: application/vnd.github+json"]
                tail = ["--input", "-"] if body is not None else []
                self.assertEqual(argv, ["gh", "api", "--method", method, *accept, url, *tail])
                self.assertEqual(json.loads(stdin) if body is not None else None, body)


class Refusals(unittest.TestCase):
    def test_no_caller_text_reaches_another_path(self) -> None:
        fake = FakeGh()
        refused: list[Callable[[], object]] = [
            lambda: fake.pull("7/../../x"),
            lambda: fake.pull(0),
            lambda: fake.pull("٧"),
            lambda: fake.pull("1" * 5000),
            lambda: FakeGh("o/x", "r"),
            lambda: FakeGh("..", "r"),
            lambda: gh_client.branch_name("a/../../../pulls"),
            lambda: gh_client.branch_name("a//b"),
            lambda: gh_client.branch_name("a/.x"),
            lambda: gh_client.branch_name("a.lock"),
            lambda: gh_client.branch_name("-x"),
        ]
        for call in refused:
            with self.assertRaises(GhError):
                call()
        self.assertEqual(fake.requests, [])


class Failures(unittest.TestCase):
    def test_gh_failures_raise(self) -> None:
        def failing(*_: object, **__: object) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess([], 1, "", "gh: Not Found (HTTP 404)\n")

        with mock.patch("gh_client.subprocess.run", side_effect=failing):
            with self.assertRaisesRegex(GhError, "HTTP 404"):
                GhApi("o", "r").pull(7)
        self.assertRaises(GhError, FakeGh().pull, 7)
        with mock.patch("gh_client.subprocess.run", side_effect=FileNotFoundError("gh")):
            self.assertRaises(GhError, GhApi("o", "r").pull, 7)
        bad = subprocess.CompletedProcess([], 0, "<html>", "")
        with mock.patch("gh_client.subprocess.run", return_value=bad), self.assertRaises(GhError):
            GhApi("o", "r").pull(7)

    def test_a_remote_names_the_repository(self) -> None:
        self.assertEqual(gh_client.owner_repo("https://github.com/o/r.git"), ("o", "r"))
        self.assertEqual(gh_client.owner_repo("git@github.com:o/r"), ("o", "r"))
        self.assertIsNone(gh_client.owner_repo("https://example.com/o/r"))


if __name__ == "__main__":
    unittest.main()
