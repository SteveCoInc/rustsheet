---
id: T-5108
title: "FVSCHEDULE: score the val2 FVS probes (Excel for the web filled) against the overflow, exponent-range and product-rounding hypotheses"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Each FVSCHEDULE hypothesis of the FVS probes (what the overflowing call is, the overflow cut, the retained exponent
range, zero and unit factors, underflow, plain versus twice-rounded product) is scored against the filled Excel values,
the confirmed rules are recorded in docs/excel-semantics/financial.md, and an implementation task is filed for each.

## Context
Owns: probes/val2-fvs-01, probes/val2-fvs-02 and probes/val2-fvs-03 (every visibly-off cell of these cases).
- Probes: the val2 batch of docs/research/probe-requests-2026-11-08.md (questions, hypotheses, model predictions) and
  docs/research/probe-handoff-2026-11-08.md (how it was filled, CAL-01 result). Filled on excel-graph 2026-11-09 by the
  PR that filed this task.
- CAL-01 passed on every anchor (the build reproduces the cached web corpus), but in-sheet `SUM(x,-y)*2^52` residuals
  snap to 0 (numbers.md, the SUM near-zero snap): read residuals from the raw value cells, never from a SUM column.
- No engine change lands until a probe confirms its rule; no fitted special cases (handoff, "What happens next").
- Web carrier answered (fvs-01): on excel-graph the overflowing `FVSCHEDULE(5916,{-1E+300,1E+300})` reads inside the
  sheet as a live number 0 (TYPE 1, ISNUMBER TRUE, `B1+1` is 1, `"["&B1&"]"` is "[0]"), except `B1/(2^1023)`, which is
  `#NUM!`; ours is `#DIV/0!` throughout (financial.md, "FVSCHEDULE whose running product overflows"). The desktop live
  and seeded steps and the isolated fvs-02 variants are still open (T-4250).
- fvs-02: `FVSCHEDULE((2^1023),1)` and its neighbours are 0 in Excel, ours `#DIV/0!`; fvs-03: the product-difference
  columns are 0 in Excel where ours differs by 0.03125.
- The FORMULATEXT cell of fvs-01 is a separate display issue, owned by T-5111.
- Code: `fn_fvschedule` (functions/financial).

## Acceptance criteria
- [ ] Every hypothesis of the family in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing
      the probe cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
- Desktop-only questions (M14, decision 0039).
