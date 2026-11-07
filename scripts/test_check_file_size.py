from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_file_size as size  # noqa: E402 (import after sys.path setup)


class CountLines(unittest.TestCase):
    def test_a_lint_debt_block_is_not_counted(self) -> None:
        text = (
            "//! Docs.\n"
            "// LINT-DEBT-BEGIN (decision 0335)\n"
            "#![expect(\n    clippy::float_cmp,\n)]\n"
            "// LINT-DEBT-END\n"
            "fn f() {}\n"
        )
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "a.rs"
            p.write_text(text)
            self.assertEqual(size.count_lines(p), 2)

    def test_a_file_without_a_block_counts_every_line(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "a.rs"
            p.write_text("a\nb\nc\n")
            self.assertEqual(size.count_lines(p), 3)


if __name__ == "__main__":
    unittest.main()
