---
id: T-4461
title: "Probe IMSQRT's per-part #NUM! cut at sqrt(DBL_MAX): the doubles either side of 1.3407807929942596E154, real and imaginary"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells pin where IMSQRT's per-part `#NUM!` starts, to adjacent doubles, for the real and the imaginary part and
for both signs. The engine either matches every new cell or a follow-up task is filed.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 16a00ebc7). engineering.md (the IM* overflow notes, T-1996)
  says: "The threshold per part lies between 1.34E154 and 1.35E154, consistent with sqrt(DBL_MAX) = 1.34078E154
  (exact boundary unprobed)".
  - Bracketing cells: `IMSQRT("1.34E154")` is `1.15758369027902E+77`, while `"1.35E154"`, `"1E160i"`, `"-1E160"` and
    `"1E154+1.4E154i"` are `#NUM!` (probes/eng-imsqrt-large, eng-imsqrt-large-b, excel-graph).
- The engine (`imsqrt` in crates/rustsheet-engine/src/functions/engineering/complex/mod.rs) squares each part in
  doubles with `flushed_square` and is `#NUM!` when a square is not finite. Its cut is therefore the largest x whose
  rounded square is finite: x*x <= DBL_MAX plus half an ulp. That is the double just above sqrt(DBL_MAX) =
  1.3407807929942596E154 or the one at it, depending on rounding. If Excel squared in x87 extended precision before
  storing, the cut would be the same, so this probe mainly confirms the inference.
- Probe design: the text forms of the three doubles around 1.3407807929942596E154 (the oracle needs 17 significant
  digits; write them as COMPLEX text or concatenated literals), each as a real part, an imaginary part and negated.
  Add a row at the cut with a tiny other part, such as `1.3407807929942596E154+1i`, which the sum rule makes "0"
  when it overflows.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes, oracle-filled from excel-graph, holds the cells above.
- [ ] engineering.md replaces "(exact boundary unprobed)" with the measured bracket, citing the cells.
- [ ] Every new cell passes. If any fails, the PR adds its known-failure lines and files a follow-up task with `make
      new-task` that names them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSQRT's digits away from the cut, and the other IM* overflow edges (T-2631, T-4425).
