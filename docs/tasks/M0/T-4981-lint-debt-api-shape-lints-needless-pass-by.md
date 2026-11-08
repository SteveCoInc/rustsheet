---
id: T-4981
title: "Lint debt: API-shape lints (needless_pass_by_value, unused_self, unnecessary_wraps, excessive bools) fixed or given a reasoned expect"
milestone: M0
lane: quality
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4856]
---

## Goal
The API-shape lints of the fork's debt are fixed or carry a reason at the site: needless_pass_by_value (28), unused_self (14), unnecessary_wraps (12), struct_excessive_bools (12), trivially_copy_pass_by_ref (4), struct_field_names (4), inline_always (3), ref_option (2), mut_mut (1), fn_params_excessive_bools (1), implicit_hasher (1).

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. Semantics must not move: `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [x] Those 11 lints have no file-level entry left; each kept site has a statement-level expect with a reason.
- [x] No public API of crates/rustsheet changes; engine-internal signatures may.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).

## Outcome
- The 11 lints have no file-level entry left: quality/lint-debt.json goes from 914 entries in 430 files to 832 in 409
  (82 entries removed; 21 files lose their LINT-DEBT block entirely).
- Fixed outright (signatures borrow instead of move, no wrap, no unused `self`): `mut_mut` (3, reborrow),
  `trivially_copy_pass_by_ref` (the weekend masks go by value), `ref_option` (2, `Option<&T>`), `inline_always`
  on `fetch_cell` and `apply_ifs_at` (now `#[inline]`), 28 of the `needless_pass_by_value` sites (`&str`, `&[T]`,
  `&T`; `Option<&Error>` in INDEX's lift), most `unused_self` sites (associated or free functions), and the
  `unnecessary_wraps` of `Worksheet::remove_cell`, AMORDEGRC's `degressive`/`first_depreciation`,
  `z_test_stats`, the xlsx `end_row`. Two struct fields named after their struct are renamed (`Keys::tuples`,
  `Lines::made`). `ParsedReference`, `TableSpecifier`, `SubTotalMode`, `LiftCost` and the xlsx `CellArrayKind`
  (small, plain-data types) derive `Copy`, which removes their by-value and by-reference findings.
- Kept with a statement-level `#[expect(..., reason = "...")]`: the 16 `struct_excessive_bools` structs (each
  mirrors independent OOXML attributes, format-section properties, PCRE option bits or pass flags), the
  `includes` const fn (one flag per `apply*` attribute), `Evaluation`'s counters, xlsx `Sheet::sheet_id`
  (the `sheetId` attribute), TAN's measured `#[inline(always)]`, serde's `is_false(&bool)`,
  `new_parser_english`'s `HashMap` (the parser stores it), the units dispatch and the five zero-argument `fn_*`
  builtins (one dispatch signature), xlsx `patch` (its documented outer `Result`), `set_user_input` and
  `update_cell_with_formula` (public Model API called with owned text), test helpers that build an
  `Option`/`Result` or take what the test builds inline, and four `kind: Function` parameters: `Function` is
  not `Copy`, and deriving it makes about 75 `&Function` parameters fail `trivially_copy_pass_by_ref`, so that
  is follow-up T-5080 (filed here); `token::Error` has the same cascade (noted in T-5080's out of scope).
- Decision 0353 records the paydown (the lock is a protected path). No public API of crates/rustsheet changes. Eval delta: none (`make check-fast`: 0 regressions, 0 fixed in
  every suite). `cargo test -p rustsheet_engine -p rustsheet_xlsx`: 6540 passed.
