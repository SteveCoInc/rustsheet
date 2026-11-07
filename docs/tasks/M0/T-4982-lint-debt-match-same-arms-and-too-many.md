---
id: T-4982
title: "Lint debt: match_same_arms and too_many_lines merged or split where order-safe, a reasoned expect where arms enumerate Excel cases"
milestone: M0
lane: quality
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4856]
---

## Goal
match_same_arms (46 files) and too_many_lines (50) are paid down: identical arms merged where the merge keeps match order (a guarded arm stays first), and long functions split only along a cohesive seam.

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. Semantics must not move: `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] Each remaining arm list that enumerates Excel cases on purpose carries a statement-level expect naming why.
- [ ] Both lints have no file-level entry left, or the task's Outcome lists the files left and why.
- [ ] `make eval-check`: 0 regressions, 0 fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
