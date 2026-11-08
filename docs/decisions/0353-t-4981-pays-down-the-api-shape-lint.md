# 0353 — T-4981 pays down the API-shape lint debt; a kept site carries a statement-level expect

- **Status:** accepted (2026-11-08, T-4981). Applies decision 0335.
- **Context:**
  - Decision 0335 locks the fork's per-file lint debt in `quality/lint-debt.json`, a protected path
    (`scripts/check_governance.py`), so every change to it needs a decision record. T-4981 is the first change that
    only removes entries. It covers 11 API-shape lints: `needless_pass_by_value`, `unused_self`, `unnecessary_wraps`,
    `struct_excessive_bools`, `trivially_copy_pass_by_ref`, `struct_field_names`, `inline_always`, `ref_option`,
    `mut_mut`, `fn_params_excessive_bools` and `implicit_hasher`.
  - Some of those sites are deliberate. Examples: a struct with one `bool` per OOXML attribute, the one signature
    that every arm of a dispatch match shares, serde's `skip_serializing_if` that takes `&bool`, TAN's measured
    `#[inline(always)]`, and the public `Model` setters that take owned text.
- **Decision:**
  - `quality/lint-debt.json` goes from 914 entries in 430 files to 832 in 409, written by
    `scripts/check_lint_debt.py --update`, with no entry added or widened. Each site is either fixed, or keeps a
    statement-level `#[expect(clippy::x, reason = "...")]` that states why the shape is intended. Kept sites that
    a later change will fix name that task in the reason (T-5080 for `Function`, which is not yet `Copy`).
- **Consequences:**
  - These 11 lints are now enforced in every file. A new violation fails `make clippy`, the same as in any file
    written after the fork.
  - A later lint-debt paydown task (T-4980 to T-4986) follows this pattern. It removes lock entries only, through
    `--update`, and it puts the reason on the kept site, never in a file-level block. It still needs its own
    decision record, because the lock is a protected path.
