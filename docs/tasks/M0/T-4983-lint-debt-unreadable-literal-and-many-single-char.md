---
id: T-4983
title: "Lint debt: unreadable_literal and many_single_char_names in the numeric kernels"
milestone: M0
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4856]
---

## Goal
The numeric kernels read clearly: unreadable_literal (45 files) gets digit separators and many_single_char_names (30) either gets names or a reasoned expect where the code follows a cited paper's notation (e.g. TOMS 708, Cody).

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. Semantics must not move: `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] A unit test pins that every changed literal keeps its exact f64 bits (compare to_bits before/after).
- [ ] Both lints have no file-level entry left.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
