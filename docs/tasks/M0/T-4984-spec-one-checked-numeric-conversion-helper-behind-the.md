---
id: T-4984
title: "Spec: one checked numeric-conversion helper behind the 423 fork-era cast lints"
milestone: M0
lane: quality
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4856]
---

## Goal
A spec (docs/specs/) for one helper module for f64-to-integer and integer-to-f64 conversions (saturating, checked, Excel's truncation toward zero for row/column/count arguments), so the 423 cast entries (cast_possible_truncation 173, cast_precision_loss 99, cast_sign_loss 98, cast_possible_wrap 53) are paid down by calling it instead of 423 local fixes. Directive 3: search for the existing helpers first (rg 'fn .*to_(u32|usize|i32|i64)' crates/rustsheet-engine/src) and fold them in.

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. Semantics must not move: `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] The spec names the helper set, which existing helpers it replaces, how each Excel argument kind converts (cite docs/excel-semantics/coercion.md), and splits the migration into S/M tasks by directory.
- [ ] The follow-up tasks are filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
