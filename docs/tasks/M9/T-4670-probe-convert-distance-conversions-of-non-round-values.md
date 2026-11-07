---
id: T-4670
title: "Probe CONVERT distance conversions of non-round values near the overflow path (1E298 to 1E308, m to m, m to ft, mi to km)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for CONVERT distance conversions whose `value * F_from` overflows now that the distance factors are
in angstroms (T-4630), and the engine matched to it.

## Context
- T-4630 (#3721) moved the distance factors to angstroms (m = 1E10). `value * F_from` now overflows from about 1.8E298
  even for m to m, and convert.rs then divides first (`value / F_to * F_from`; engineering.md "Overflow").
- Only round values are evidenced there (probes/convert-temperature-ov00, -ov01, -ov20: 1E290, 1E300). A non-round
  value such as `CONVERT(1.234567E300,"m","m")` may now be 1 ulp off (reviewer finding, #3721 rounds 1 and 2). Area in
  square angstroms has the same exposure.
- One probe case, probes-numeric/t4670-convert-distance-overflow (excel-probe skill, tag M9): about 30 non-round values
  from 1E297 to 1.7E308, for m to m, m to ft, mi to km, in to cm, and m2 to m2 and m2 to ft2.

## Acceptance criteria
- [ ] The case is committed with excel-graph values, plus its math sidecar.
- [ ] Every cell passes, or the rule for the overflow path is changed to match it; engineering.md "Overflow" cites the
      cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The distance factors themselves (T-4630) and non-distance classes other than area.
