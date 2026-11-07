---
id: T-1812
title: "Bench the lifted and scalar LARGE paths in benches/lift.rs"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1772]
---

## Goal
crates/rustsheet/benches/lift.rs measures the T-1772 paths: a lifted `=SUM(LARGE(A1:A1000000,SEQUENCE(200)))` (one read, one sort, 200 O(1) elements) and a filled-down scalar `=LARGE(A1:A100000,2)` (the unlifted call, which goes through `lift_with_sorted_data` and sorts on its one element).

## Context
T-1772's performance review (follow-up, not a blocker): `functions/statistical/sorted_data.rs` reads the data once, and the kernel sorts it lazily after its own checks. SPEC-0011 "Performance" lists the benches; record main's and the branch's release numbers in the Outcome as a smoke baseline (no budget: the lifted path is new).

## Acceptance criteria
- [ ] Both cases are in benches/lift.rs with the fixture filled (no blank-cell shortcut).
- [ ] Their release timings are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the kernels.

## Outcome
Two cases added to `crates/rustsheet/benches/lift.rs` (`large_cases`), column A filled (no blank shortcut). Release
numbers (11 runs, min / median), identical on main since the kernels are unchanged:
- `=SUM(LARGE(A1:A1000000,SEQUENCE(200)))`, 1,000,000 rows: 208 ms / 221 ms (C1 = 19800).
- `=LARGE(A1:A100000,2)`, 100,000 rows: 10.2 ms / 10.6 ms (C1 = 9).
A smoke baseline, no budget. Eval: no change.
