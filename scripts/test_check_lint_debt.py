from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_lint_debt as debt  # noqa: E402 (import after sys.path setup)


class RustBlocks(unittest.TestCase):
    def test_expect_entries_are_read_from_the_block(self) -> None:
        text = (
            "//! Module docs.\n"
            "// LINT-DEBT-BEGIN (fork-era lint debt, decision 0335: entries may only be removed)\n"
            "#![expect(clippy::float_cmp, clippy::cast_sign_loss, elided_lifetimes_in_paths)]\n"
            "#![cfg_attr(test, expect(clippy::too_many_lines))]\n"
            "// LINT-DEBT-END\n"
            "#[allow(clippy::panic)]\nfn f() {}\n"
        )
        self.assertEqual(
            debt.rs_entries(text),
            [
                "clippy::cast_sign_loss",
                "clippy::float_cmp",
                "clippy::too_many_lines",
                "rust::elided_lifetimes_in_paths",
            ],
        )

    def test_text_outside_the_block_is_not_debt(self) -> None:
        self.assertEqual(debt.rs_entries("#![expect(clippy::float_cmp)]\nfn f() {}\n"), [])

    def test_a_block_must_start_a_line(self) -> None:
        self.assertEqual(
            debt.rs_entries("x // LINT-DEBT-BEGIN\n#![expect(clippy::a)]\n// LINT-DEBT-END\n"), []
        )


if __name__ == "__main__":
    unittest.main()
