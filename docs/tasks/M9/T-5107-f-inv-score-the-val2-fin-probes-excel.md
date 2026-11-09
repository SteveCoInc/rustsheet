---
id: T-5107
title: "F.INV: score the val2 FIN probes (Excel for the web filled) against the search-target, derivative-storage and iteration-cap hypotheses"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Each F.INV hypothesis of the FIN probes (search-target repeatability and neighbourhood, derivative-exponent storage,
iteration cap and flush-stop class, BETA.INV twins) is scored against the filled Excel values, the confirmed rules are
recorded in docs/excel-semantics/statistics.md, and an implementation task is filed for each confirmed rule.

## Context
Owns: probes/val2-fin-01, probes/val2-fin-02 and probes/val2-fin-03 (every visibly-off cell of these cases).
- Probes: the val2 batch of docs/research/probe-requests-2026-11-08.md (questions, hypotheses, model predictions) and
  docs/research/probe-handoff-2026-11-08.md (how it was filled, CAL-01 result). Filled on excel-graph 2026-11-09 by the
  PR that filed this task.
- CAL-01 passed on every anchor (the build reproduces the cached web corpus), but in-sheet `SUM(x,-y)*2^52` residuals
  snap to 0 (numbers.md, the SUM near-zero snap): read residuals from the raw value cells, never from a SUM column.
- No engine change lands until a probe confirms its rule; no fitted special cases (handoff, "What happens next").
- Off today: 167 cells in fin-01 (F.INV at df 9/3 and 9/1186, 42 each; F.INV at 9/8 and F.INV.RT at 3/9, 17 each; BETA.INV twins;
  BETA.DIST round trips), 39 in fin-02 (the
  exact-iterate kernel columns), 176 in fin-03 (FINV, F.INV.RT, the stop-class columns, 66 Excel `#NUM!`).
- Also owns the F.INV anchor of probes/val2-cal-01, P!H2 (ours 5.814110643467815e-68, Excel 5.814110643623313e-68:
  about 2.7e-11 relative, inside the envelope but not a low-bit difference).
- Code: the F.INV search (`rg -n 'fn_f_inv' crates/rustsheet-engine/src/functions`); related open owners T-2500, T-2767.

## Acceptance criteria
- [ ] Every hypothesis of the family in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing
      the probe cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
- Desktop-only questions (M14, decision 0039).
