---
id: T-4202
title: "Correct T-3396's CHISQ.INV wording: df above 1E10 is #NUM! by the domain limit, not the band"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
statistics.md and T-3396 say why `CHISQ.INV(3E-k,2a)` (probes/t3396-gamma-inv-band-huge column B) is `#NUM!`: its df,
1.4E10 to 2.4E10, is above CHISQ.INV's deg_freedom limit of 1E10, so the column says nothing about the band.

## Context
The excel-fidelity reviewer on #3165 (T-3396) found the statistics.md passage "CHISQ.INV(3E-k,2a) is #NUM! at every
probed k ... inside the band GAMMA.INV has at the same shape" misleading. The case's math sidecar gives every B cell
"deg_freedom outside [1, 1e+10]". T-3396's attempt note and the case description also call column B a hold-out, but
it cannot separate any band rule. The statistics.md line that says CHISQ.INV is "not probed above 1E10" (near line
5070) can now cite these cells as the probe.

## Acceptance criteria
- [x] The statistics.md passage and T-3396's attempt note give the domain limit as the reason, not the band, and stop
      calling column B a hold-out.
- [x] The "not probed above 1E10" line cites probes/t3396-gamma-inv-band-huge column B.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (the engine already matches column B).

## Outcome
statistics.md and T-3396's attempt note now give the deg_freedom limit (1E10) as the reason column B is `#NUM!` and no longer call it a hold-out; the "not probed above 1E10" line cites the column as the probe. Docs only; no eval delta. The case file's own description still says "hold-out" (eval case files are left untouched here).
