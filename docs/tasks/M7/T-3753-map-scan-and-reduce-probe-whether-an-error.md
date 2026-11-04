---
id: T-3753
title: "MAP, SCAN and REDUCE: probe whether an error array or a bad function argument comes first"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3655]
---

## Goal
MAP, SCAN and REDUCE give Excel's error when their array argument is an error and their function argument also
cannot be called (a wrong-arity LAMBDA, a number, an error).

## Context
T-3655 (PR #2769) made every LAMBDA helper refuse a function it cannot call with one `#VALUE!`
(`Model::helper_caller`, lambda.rs; arrays.md "LAMBDA helpers refuse a function they cannot call"). MAP, SCAN and REDUCE
check the function before they read the array; BYROW and BYCOL read the array first. Which error Excel reports when
both are bad is unprobed: that section's "Unprobed" line (excel-fidelity review of #2769). Probe with the
`excel-probe` skill, for example `MAP(NA(),LAMBDA(a,b,a))`, `MAP(1/0,5)`, `SCAN(0,NA(),LAMBDA(a,a))`,
`REDUCE(0,1/0,LAMBDA(a,a))`, `REDUCE(0,NA(),NA())`, and `BYROW(NA(),LAMBDA(a,b,a))` for comparison, each read through ROWS,
INDEX and ERROR.TYPE. Code: `fn_map`, `fn_reduce` (map_reduce.rs) and `fn_scan` (scan.rs).

## Acceptance criteria
- [x] A probe case (excel-graph) covers the forms above.
- [x] The engine matches every probed cell (move the `helper_caller` call after the array read where Excel reads the
      array first); arrays.md's "Unprobed" line is replaced by the rule with the probe cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- MAKEARRAY's order (sizes first: already probed in T-3655).

## Outcome
- Probe `probes/t3753-lambda-helper-error-array-order` (16 forms x ROWS/COLUMNS/INDEX/ERROR.TYPE, excel-graph): Excel reads
  the array argument before checking the function in MAP, SCAN, REDUCE, BYROW and BYCOL.
- `fn_map`, `fn_scan` and `fn_reduce` now call `helper_caller` after the array read; arrays.md's "Unprobed" line is replaced
  by the rule. The case passes 64/64; before the change 11 formula rows (A1-A4, A6-A12) mismatched.

## Landing notes
- PR #2882. The reviewers (security, design, excel-fidelity) all approved head 302454d; the catch-up merge of main (baseline lists only,
  e0ed525) needs no re-review. CI had just restarted on that head when the session window closed.
- Next step: when CI is green on the latest head, `make merge-mode` (should print `lock`) then enable auto-merge (SQUASH). Do not run
  `make mark-ready`.
- A conflict in `eval/baseline/probes.txt` is cleared with `make catch-up PR=2882`, not by hand.
