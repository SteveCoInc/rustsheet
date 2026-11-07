---
id: T-4892
title: "Math values for five gen-m9 multi-call GAMMAINV/CHIINV cells on the huge-shape closed-form branch, then mark them R3"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3364]
---

## Goal
The five gen-m9 cells that T-3364 left visibly off on GAMMA.INV's huge-shape closed-form branch get math values, are
marked R3 (decision 0188, case 3) and leave the visible-parity lock.

## Context
- Decision 0337 (T-3364) raised five gen-m9 visible-parity lock lines by one each. Each is a call on the closed-form
  branch where Excel / engine = 1.000266 (the branch's signature) and shape is at least 1E9, but `eval_math` gives no
  math value. Each wraps the call or reads `Data!` cells:
  - 0a614cfe974b59c3 A327 `GAMMAINV(1-1E-10,Data!C7,0.5)`
  - 4e55500bb8ad3cd8 A341 `CHIINV(0.001,ROUND(9999999999999999,2)+…)`
  - 7bd219ec0c0c5e22 A239 `CHIINV(1E-300,ROUND(9999999999999999,2))`
  - a013a45a8fbcab4f A338 `CHIINV(0.5,(Data!C$1+1E15)*RADIANS(1))`
  - d8266bc15533702a A296 `GAMMA.INV((2^Data!A2)^(…),AVERAGE(…),1E-300)`
- Hand checks in review: `CHIINV(1E-300,1E16)` is about df + 37.05 sqrt(2 df) = 1.00000052E16 (ours 1.0000005239E16,
  Excel 1.0002663E16). A338 is about df - 2/3 at df 1.7453292519943E13 (ours 17453292519942.33).
- Related: T-4777 (math for an argument that reads another formula cell) and T-3993 / decision 0297 (nested calls as
  composites). Find why `rustsheet_tools.accuracy.eval_math` skips these five (a `Data!` literal cell, or ROUND, RADIANS
  or AVERAGE as wrappers) and extend it there, not with a one-off value file.

## Acceptance criteria
- [ ] The five cells have 50-digit math values from `eval_math` (or a decision record says why one cannot).
- [ ] Each is marked R3 in eval/accuracy/math-over-excel.tsv, and the five visible-parity lock lines come back down
      (via `make eval-baseline`, never by hand).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other gen-m9 cells without math values.
