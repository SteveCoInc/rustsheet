---
id: T-4691
title: "T-4541 review follow-ups: note the t4541 M5 comparator, mark CONFIDENCE.T's tiny-p order unprobed, add the 2·DBL_MIN cut to the Student t Gate A defs"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4541]
---

## Goal
The docs and Gate A definitions around T-4541's two-tailed tiny-p cut (PR #3736) say exactly what is evidenced.

## Context
Excel-fidelity review of T-4541:
- probes/t4541-t-inv-2t-tiny-p M5 was predicted TRUE and is FALSE in Excel. The probe design caused this: it compares
  `4.45014771701441E-308/2` = 2.225073858507205E-308 with the literal 2.22507385850721E-308, which is itself above DBL_MIN
  (2.2250738585072014E-308). T-4541's Outcome says "every cell matches the hypothesis". Add a line there or in
  statistics.md saying M5's comparator was above DBL_MIN and does not bear on the cut.
- statistics.md (the "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308" entry): CONFIDENCE.T's
  tiny-alpha check runs after its size checks, so n = 1 with a tiny alpha stays `#DIV/0!`. Only the normal.rs comment
  and the Outcome say this is unprobed; add it to the entry, or probe it.
- tools/src/rustsheet_tools/accuracy/defs/student_t.py (`_two_tailed`, `_confidence_t_domain`) still gives a finite math
  value at p < 2·DBL_MIN (sidecar A1 = 2.86E+307). Gate A passes because it judges these points on Excel's error. Optionally
  add a `domain` rule citing T-4541, as the T.DIST density cut has, and regenerate only those functions' math sidecars.

## Acceptance criteria
- [ ] The M5 note and the CONFIDENCE.T order note are in statistics.md (or the order is probed).
- [ ] The Student t defs carry the 2·DBL_MIN domain rule, or the entry says why not; `rustsheet-eval accuracy` 0 failing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T.DIST's huge-x density cut (T-3554).
