---
id: T-4461
title: "Probe IMSQRT's per-part #NUM! cut at sqrt(DBL_MAX): the doubles either side of 1.3407807929942596E154, real and imaginary"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
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

## Outcome
Probe `probes/t4461-imsqrt-cut` (41 cells, excel-graph). Complex text keeps 15 significant digits, so the three
17-digit doubles around sqrt(DBL_MAX) all read as `1.34078079299425E154` and are accepted; `1.34078079299426E154`
and above (`1.3408E154` ...) are `#NUM!`. The cut lies between those two 15-digit texts, the finest text can show.
The engine matches all 41 cells: no engine change, no follow-up. The only baseline change is the new `C` line for
the case in `eval/baseline/probes.txt`. engineering.md updated. IMSQRT has no Gate A definition, so no sidecar.

## Handoff (resumed)
Work is complete; only a clean review round 3 and the enqueue remain (PR #3605).
- Round 1 (excel-fidelity) REQUEST_CHANGES: my first reading of the 17-digit cells was wrong (complex text truncates to
  15 digits, so they are one value). Fixed: added A37:A41 (`1.34078079299426E154`, all `#NUM!`), rewrote the
  engineering.md entry and this Outcome.
- Round 2 REQUEST_CHANGES: a blanket `sed` of "36 cells" in engineering.md also edited two unrelated lines (T-3912
  A1:A36 and the t4422 cell count). Reverted both; the engineering.md diff is now the single T-4461 hunk.
- Next: set `status: done` again, run `make check-fast`, push, re-run reviewer-excel-fidelity on the new head, mark the
  PR ready when CI is green and it approves, then enqueue. Do not use global replaces on engineering.md.

## Addendum (a second run, #3617)
A parallel run probed the same task in probes/t4461-imsqrt-cut-numbers (64 cells, excel-graph, 2026-11-06), which adds
number forms that text cannot reach. The cut is exactly where a part's square stops being finite: 2^512 - 2^459 gives a
value and 2^512 `#NUM!` (A1:D2). COMPLEX rounds to 15 digits (A3:D8, B15), and `=` treats the two doubles as equal (A15).
Ours matches all 64 cells. engineering.md's entry is extended with these.

