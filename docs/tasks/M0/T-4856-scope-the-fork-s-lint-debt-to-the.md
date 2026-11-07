---
id: T-4856
title: "Scope the fork's lint debt to the files that still violate each lint (per-file expect), and file the duplicate-concept audit"
milestone: M0
lane: quality
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The fork's lint exemptions cover only the files that still violate them, so post-fork code gets the full workspace lint set, and the debt can only shrink. Also: the same Excel concept implemented several times with different rules is audited and filed as tasks.

## Context
91 crate-wide `allow` entries (engine, xlsx and the Python binding `Cargo.toml`) were never reduced after the fork, and because they applied crate-wide, they switched pedantic clippy off for every file written since. A codebase review also found Excel concepts (15-digit rounding, text ordering, booleans, name keys, reference parsing) implemented several times with differing rules. Decision 0335.

## Acceptance criteria
- [x] No `LINT-DEBT` block is left in any `Cargo.toml`; each remaining exemption is a per-file `#![expect]` in a `LINT-DEBT` block, locked per file by `scripts/check_lint_debt.py` (tests: `scripts/test_check_lint_debt.py`).
- [x] Machine-applicable fixes applied and lints at zero hits removed.
- [x] The duplicate-concept audit is filed as tasks (one per concept).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the cast and float lints file by file (they need per-site reasoning; `quality` lane follow-ups).
- Unifying the duplicated concepts (T-4847 to T-4855; each needs Excel evidence first).

## Outcome
- Measured with every debt lint switched on: about 23,200 engine hits and 1,950 xlsx hits. 20,357 were
  `used_underscore_items` from 17 underscore-named test helpers on `Model`/`UserModel`, renamed with an `_a1` /
  `_at` suffix (`_set` → `set_a1`, `_get_text` → `get_text_a1`, `_get_text_at` → `get_text_by_index`, ...).
- `cargo clippy --fix` for: cast_lossless, manual_string_new, needless_raw_string_hashes, implicit_clone,
  redundant_closure_for_method_calls, unnecessary_semicolon, single_match_else, inconsistent_struct_constructor,
  if_not_else (engine only: xlsx's fixes did not compile), explicit_iter_loop, redundant_else, map_unwrap_or,
  wildcard_imports, cloned_instead_of_copied, range_plus_one, enum_glob_use, unnested_or_patterns, manual_assert_eq,
  ignored_unit_patterns, range_minus_one, bool_to_int_with_if, single_char_pattern, match_bool,
  elidable_lifetime_names, ref_as_ptr, unnecessary_join, manual_assert. Not `manual_midpoint`: `f64::midpoint`
  can round differently near overflow, so that needs evidence.
- Lock: 91 crate-wide entries became 916 (file, lint) entries for 37 distinct lints, in 431 files. Every other file
  of the two crates is now under full pedantic clippy. `expect` fails the build once a file is clean, so the
  entry must then be deleted.
- Python Sphinx docs (T-4257) and the engine/xlsx crate docs and READMEs no longer present the IronCalc API as the
  entry point. They name the `rustsheet` crate and `rustsheet.Workbook` instead.
- The xlsx examples `deep_chain` and `forward_chain` share `examples/chain/mod.rs` and write through a locked
  stdout handle instead of carrying exemptions; duplicated lines 2872 → 2806.
- Follow-ups: T-4847 to T-4855 (duplicate concepts); the per-file cast/float debt remains for `quality`-lane tasks.
- Eval delta: none expected. The renames are test-only, and the clippy fixes do not change semantics.
