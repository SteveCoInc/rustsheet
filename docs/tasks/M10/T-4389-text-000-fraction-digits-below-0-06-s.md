---
id: T-4389
title: "TEXT .000 fraction digits below 0.06 s: the offset past the 15-digit cut"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
TEXT's `.000` fraction digits match Excel where the remainder of a second is below about 0.06 s, which Excel rounds up
past what T-4370's 15-digit cut allows: `TEXT(0.0045/86400-36*2^-77,"[h]:mm:ss.000")` is "0:00:00.005", ours
"0:00:00.004".

## Context
T-4370 reads the fraction digits from the half-step cut's remainder `r` of a second rounded to 15 significant digits
(formatter/elapsed.rs `fraction_digits`; docs/excel-semantics/number-formats.md "Fraction digits of a second at a
tie"). Under `.000` with `r` below about 0.06 Excel behaves as if 1.6E-17 to 1.9E-17 s were added before that cut:
0.0045 flips between 2.12E-17 and 2.35E-17 s below the tie, 0.0075 between 1.91E-17 and 2.37E-17, 0.0015 and 0.0025
round up at every probed offset (down to 1.8E-17 below), 0.0545 at 5.9E-17 below. `.0` and `.00` show no offset
(0.005 under `s.00` flips 0.45 to 0.68 units of the 15th digit below). Neither a decimal grid (14 to 17 digits) nor a
binary snap of `r` fits all of it (docs/research/t4370/README.md). Probe the `.000` remainder between 1E-4 and 0.1 s
more densely, and check whether the offset is absolute in seconds or in serial days.

Visibly-off cells this task owns (all `.000` with a remainder below 0.06 s):
- probes/t4370-ulp-map A52, A86:A91
- probes/t4370-ulp-map-2 A1:A15, A18:A29, A47:A50, A52:A63, A76:A83, A243:A244, A313:A314

## Acceptance criteria
- [ ] The listed cells pass and no other cell of the t4323, t4332 and t4370 fraction cases moves.
- [ ] A structural model: the R4 hold-out and a fresh sweep generated after it is frozen confirm it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `[s]`/`[h]`/`[m]` counts before the fraction (T-4333).
