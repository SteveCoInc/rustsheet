---
id: T-4985
title: "Lint debt: float_cmp entries classified as exact-by-design (reasoned expect) or a comparison bug"
milestone: M0
lane: quality
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4856]
---

## Goal
Every float_cmp entry (125 files) is either an exact comparison by design, given a statement-level expect with the reason (e.g. a bit-exact test against an oracle value, a whole-number serial), or a real bug, fixed with evidence.

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. A change that moves a value needs Excel evidence (excel-probe skill) and a baseline update (`make eval-baseline`); otherwise `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] float_cmp has no file-level entry left.
- [ ] Any comparison changed in non-test code cites Excel evidence (excel-probe skill) and moves no eval case without a baseline update.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
