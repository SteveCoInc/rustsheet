---
id: T-4986
title: "manual_midpoint: keep (a+b)/2 where Excel rounds that way, with evidence, instead of f64::midpoint"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4856]
---

## Goal
The 10 files that compute a midpoint as (a+b)/2 (decimal_round tests, complex trig, bessel_excel, the bond yield solver, hyperbolic, ETS model, f_density, forecast, rank_eq_avg, student_t) keep or change it on evidence. `f64::midpoint` differs from `(a+b)/2` only when `a+b` overflows (an operand above f64::MAX/2): `(a+b)/2` gives ±inf (#NUM! in a cell) where `midpoint` stays finite. For every other pair, subnormals included, the two agree to the bit, so only probes with operands near 1e308 can tell them apart. Excel decides which one RustSheet matches (directive 1).

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. A change that moves a value needs Excel evidence (excel-probe skill) and a baseline update (`make eval-baseline`); otherwise `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] Each site keeps (a+b)/2 with a statement-level expect citing the probe or eval cases it matches, or moves to f64::midpoint with probe evidence.
- [ ] manual_midpoint has no file-level entry left.
- [ ] `make eval-check`: 0 regressions; any fix recorded with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
