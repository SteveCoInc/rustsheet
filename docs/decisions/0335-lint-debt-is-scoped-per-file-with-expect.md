# 0335 — Lint debt is scoped per file with expect, not crate-wide

- **Status:** accepted (2026-11-07, T-4856). Replaces the crate-wide `LINT-DEBT` allow-lists of the fork
  (quality-gates.md "Lint debt").
- **Context:**
  - rustsheet-engine, rustsheet-xlsx and the Python binding carried 91 crate-wide `allow` entries in their
    `Cargo.toml` `[lints]` tables, inherited from the IronCalc fork. The ratchet only stopped new entries; none
    had been removed since the fork. Because they were crate-wide, they also switched those lints off for every
    file written after the fork, so pedantic clippy was effectively off for new code in the two biggest crates.
  - Measured on this branch with every debt lint switched back on (`cargo clippy -- -W <lint>`, lib/bins/examples
    and test builds, T-4856): about 23,200 hits in the engine and 1,950 in xlsx. 20,357 were `used_underscore_items`
    from 17 test helpers on `Model`/`UserModel` named `_set`, `_get_text`, and so on.
- **Decision:**
  - Each remaining exemption sits at the top of the one file that still violates it, between
    `// LINT-DEBT-BEGIN` and `// LINT-DEBT-END`, as `#![expect(...)]`, or as `#![cfg_attr(test, expect(...))]`
    when only the file's test code violates it. Crate manifests carry no debt.
  - `expect`, not `allow`: once a file stops violating a lint, the expectation is unfulfilled, clippy warns, and
    `-D warnings` fails until the entry is deleted. So the debt can only shrink, and it shrinks as files are touched.
  - `scripts/check_lint_debt.py` locks the per-file entries in `quality/lint-debt.json`. A file that is not in the
    lock, or a lint a file's lock entry does not list, fails the check. New files therefore get the full workspace
    lint set.
- **Consequences:**
  - Paid down in the same change: the test helpers were renamed (`_set` → `set_a1`, `_get_text` → `get_text_a1`,
    and so on; the `_a1` suffix marks an A1-text argument). Every machine-applicable fix was applied with
    `cargo clippy --fix` for the lints listed in T-4856's Outcome, except `manual_midpoint`, which can round
    differently and needs Excel evidence. In xlsx, `if_not_else`'s fixes did not compile, so it stays debt.
    The lock went from 91 crate-wide entries to 914 file-level entries for 37 distinct lints, across 430 of
    roughly 1,090 files. Every crate-wide entry is gone, and 25 of the fork's lints are at zero.
  - The progress report's "lint allows" count (`quality/lint-debt.json`, summed per key) now counts
    (file, lint) pairs, so it jumps from 91 to 914 at this change. That jump is a change of unit, not a regression.
  - A cast or float lint is fixed per file, with a reason and evidence for any semantic choice, as `quality`-lane
    tasks. Deleting the file's whole block once it is empty is part of the fix.
  - `scripts/check_file_size.py` does not count a `LINT-DEBT` block's lines: the blocks would otherwise push
    files just under the 1,000-line limit over it, and inherited oversized files would grow.
  - CLAUDE.md's rule against new lint-debt entries now covers new `LINT-DEBT` blocks and new entries in existing
    blocks.
