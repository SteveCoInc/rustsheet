---
id: T-4880
title: "DOLLARDE/DOLLARFR math definition: model Excel's D at 10^15+1 (lower cut) and 1E300 (10^300), then draw fractions below 1 and past 1E9"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The Gate A math definition for DOLLARDE and DOLLARFR (tools/src/rustsheet_tools/accuracy/defs/dollar_fractions.py) agrees with Excel at the 10^15+1 and 1E300 points probes/t4608 pins, and the table draws fractions below 1 and past 1E9 with Excel's results.

## Context
- T-4608 (financial.md "DOLLARDE and DOLLARFR scale the fraction...") found that Excel's D is 10^15 at `1E15+1` (a double
  `log10` rounds to 15) and 10^300 at 1E300, where the math definition takes the next power of ten above the double
  (10^301 for 1E300). The committed t4608 math sidecar therefore differs from Excel at A27:B27, A54:B54 and A81:B81 (1E300) and at every 1E15+1 / 10^15+1 row (A82:B82, A85:B85,
  A87:B87, A90:B90, A92:B92, A95:B95; sidecar A82 is 5.999999999999995 against Excel's 1.4999999999999996).
  Ours matches Excel (192/192).
- The defs' `NoValueError` for fractions below 1 and past 1E9 is now stale: the probe supplies the evidence.

## Acceptance criteria
- [ ] The math definition returns Excel's D at those points; regenerate the t4608 sidecar and check it against the oracle.
- [ ] Fractions below 1 give `#NUM!`/`#DIV/0!` in the Gate A table and fractions past 1E9 are drawn, or each is left out with a reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
