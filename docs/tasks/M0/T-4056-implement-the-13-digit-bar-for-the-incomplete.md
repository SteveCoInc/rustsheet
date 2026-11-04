---
id: T-4056
title: "Implement the 13-digit bar for the incomplete-beta family per its spec (decision 0267)"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: [T-4055]
---

## Goal
The eval and the progress report count the incomplete-beta family at the 13-digit bar, as T-4055's spec defines it.
`docs/progress.md` then shows the headline of visibly-off numbers at the 13th digit or earlier, with the family's
residue at the 14th and 15th digits on a separate line.

## Context
- Decision 0267 and T-4055's spec. Every consumer the spec names changes in this one PR, so that no two reports
  disagree.
- On the 2026-11-04 run, the change moves about 84,900 probes-numeric cells from "visibly off" to residue. The
  type and error cells and the cells at the 13th digit or earlier stay counted.

## Acceptance criteria
- [x] Unit tests in crates/rustsheet-eval cover the boundary: half a unit in the 13th digit is within, and just over
      it is off. A non-family function at the same gap stays visibly off, and a type or error cell always counts.
- [x] A `rustsheet-eval run --suite probes-numeric` run (a full `make eval` is the steward's) shows the probes-numeric visibly-off numbers drop by the family's 14th and 15th-digit count,
      and shows the residue line. No suite's bit-exact or regression counts change.
- [x] `make check` is green; reviewers approve.

## Out of scope
- Changing which cells are bit-exact, or any baseline file by hand.

## Outcome
`closeness::bucket` gains `Residue13`. The family list is `THIRTEEN_DIGIT_FAMILY`, and `within_significant_digits` is the one half-unit rule; `within_fifteen_digits` is that rule at 15. `Closeness.residue13` is `serde(default)`, and `Tier::of` no longer calls a function with residue `ulp4` or `digits15`. report.md's visibly-off table gains a residue column. The dashboard model, labels, palette, history rows and "Since the last run" carry the residue. Unit tests cover the boundary, family and non-family formulas, composites, array members and type cells; a tools test covers older reports. `rustsheet-eval run --suite probes-numeric` at a262d6b0: attributed visibly-off numbers 89,170 → 4,238, residue13 84,932. A full `make eval` (calipers, poi, corpus-bench) was not run here; the steward's next run shows it.
