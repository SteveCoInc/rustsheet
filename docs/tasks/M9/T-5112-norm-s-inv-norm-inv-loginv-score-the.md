---
id: T-5112
title: "NORM.S.INV, NORM.INV, LOGINV: score the val2 NSI probes (Excel for the web filled) against the inverse-erf exposure, central-reset and jump-spectrum hypotheses"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
usage: expert
depends_on: []
---

## Goal
Each NORM.S.INV / NORM.INV / LOGINV hypothesis of the NSI probes (that NORM.INV(p,0,1/SQRT(2)) exposes the inverse-erf
double, the leading constant, LOGINV's association, where the central noise reset is and which predicate selects it,
the adjacent-input jump spectra) is scored against the filled Excel values, the confirmed rules are recorded in
docs/excel-semantics/statistics.md, and an implementation task is filed for each.

## Context
Owns: probes/val2-nsi-01, probes/val2-nsi-02 and probes/val2-nsi-03.
- Probes: the val2 batch of docs/research/probe-requests-2026-11-08.md (questions, hypotheses, model predictions) and
  docs/research/probe-handoff-2026-11-08.md (how it was filled, CAL-01 result). Filled on excel-graph 2026-11-09 by the
  PR that filed this task.
- CAL-01 passed on every anchor (the build reproduces the cached web corpus), but in-sheet `SUM(x,-y)*2^52` residuals
  snap to 0 (numbers.md, the SUM near-zero snap): read residuals from the raw value cells, never from a SUM column.
- No engine change lands until a probe confirms its rule; no fitted special cases (handoff, "What happens next").
- No cell of these cases is visibly off today (the engine is inside the envelope on all 1,524), so the verdicts rest on
  the low bits of the raw cells: compare bits, not the 15-digit display.
- Also owns the NORM.S.INV anchor of probes/val2-cal-01, P!H8 (low bits). P!H31 (LN near-tie) is CAL-01's noise
  calibration and is expected to vary at one ulp; it needs no owner fix.
- Already read from NSI-01 block A (not yet scored): D, E, H and I are equal in all 12 rows (NORM.INV and NORMINV with
  sd 1/SQRT(2), and with the scale moved by 2^20 either way), F equals G (NORM.S.INV and NORM.INV(..,0,1)), B14
  `(1/SQRT(2))*SQRT(2)` is 1 and B15 `SQRT(0.5)*SQRT(2)` is 1.0000000000000002.
- Related: T-0925 (inverse normal, AS241).

- val3 batch, docs/research/probe-handoff-2026-11-09-val3.md (excel-graph fill, 2026-11-09). Owns: probes/val3-nsi-01 (no visibly-off cell at the fill) and probes/val3-nsi-02 (every visibly-off cell).

## Acceptance criteria
- [ ] Every NSI hypothesis in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing the probe
      cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
