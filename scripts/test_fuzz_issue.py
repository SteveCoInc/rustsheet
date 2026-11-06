"""Tests for scripts/fuzz_issue.py, the nightly fuzz failure issue (decision 0302, T-4490)."""

import base64
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import fuzz_issue  # noqa: E402


def job(name: object, conclusion: str) -> dict[str, object]:
    return {"name": name, "conclusion": conclusion}


class FailedTargets(unittest.TestCase):
    def test_only_failed_matrix_jobs_count(self) -> None:
        jobs = {
            "jobs": [
                job("fuzz (bitcode_workbook)", "failure"),
                job("fuzz (formula_eval)", "success"),
                job("fuzz (xlsx_import)", "failure"),
                job("fuzz (xlsx_import)", "failure"),
                job("report", "failure"),
                job("fuzz (Bad Name)", "failure"),
                job("fuzz (a); rm -rf /", "failure"),
                job(7, "failure"),
                "not a job",
            ]
        }
        self.assertEqual(fuzz_issue.failed_targets(jobs), ["bitcode_workbook", "xlsx_import"])

    def test_no_jobs(self) -> None:
        self.assertEqual(fuzz_issue.failed_targets({}), [])
        self.assertEqual(fuzz_issue.failed_targets({"jobs": "x"}), [])


class Body(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_no_artifact_points_to_the_log(self) -> None:
        text = fuzz_issue.body("formula_eval", "https://example.test/run/1", self.dir / "missing")
        self.assertIn("failed without saving an input", text)
        self.assertIn("`formula_eval`", text)

    def test_inputs_are_listed_and_embedded(self) -> None:
        data = bytes(range(256)) * 2
        (self.dir / "timeout-abc").write_bytes(data)
        (self.dir / "crash-def").write_bytes(b"\x00\x01")
        text = fuzz_issue.body("bitcode_workbook", "https://example.test/run/2", self.dir)
        self.assertIn("timeout (one input", text)
        self.assertIn("crash (a panic", text)
        self.assertIn("| `timeout-abc` |", text)
        embedded = text.split("<code>timeout-abc</code>")[1].split("```")[1]
        self.assertEqual(base64.b64decode("".join(embedded.split())), data)
        # The smallest input comes first.
        self.assertLess(text.index("crash-def"), text.index("timeout-abc"))
        self.assertIn("cfg(fuzzing)", text)
        self.assertNotIn("Too large", text)

    def test_large_inputs_are_listed_not_embedded(self) -> None:
        (self.dir / "oom-big").write_bytes(b"\xff" * 40_000)
        (self.dir / "crash-small").write_bytes(b"x")
        text = fuzz_issue.body("xlsx_import", "https://example.test/run/3", self.dir)
        self.assertIn("Too large to embed: `oom-big`", text)
        self.assertIn("<code>crash-small</code>", text)
        self.assertNotIn("<code>oom-big</code>", text)
        self.assertLess(len(text), 65_536)
        self.assertNotIn("cfg(fuzzing)", text)

    def test_again_is_a_comment(self) -> None:
        text = fuzz_issue.body("formula_parse", "u", self.dir, again=True)
        self.assertTrue(text.startswith("**Nightly fuzz failed again**"))


class Main(unittest.TestCase):
    def test_title_is_fixed_per_target(self) -> None:
        self.assertEqual(fuzz_issue.title("formula_eval"), "Nightly fuzz: formula_eval failed")

    def test_rejects_names_that_are_not_targets(self) -> None:
        self.assertEqual(fuzz_issue.main(["x", "title", "Bad`name"]), 2)
        self.assertEqual(fuzz_issue.main(["x", "body", "$(id)", "u", "."]), 2)
        self.assertEqual(fuzz_issue.main(["x", "body", "ok", "u", ".", "--oops"]), 2)
        self.assertEqual(fuzz_issue.main(["x"]), 2)


if __name__ == "__main__":
    unittest.main()
