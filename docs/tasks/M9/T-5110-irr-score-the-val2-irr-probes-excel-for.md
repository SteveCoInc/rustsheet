---
id: T-5110
title: "IRR: score the val2 IRR probes (Excel for the web filled) against the error-kind invariance, overflow and seed hypotheses"
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
Each IRR hypothesis of the IRR probes (error kind invariant under 2^k scaling, negation and trailing zeros; the two
overflow rules; the seed of the attempt that decides the error kind) is scored against the filled Excel values, the
confirmed rules are recorded in docs/excel-semantics/financial.md, and an implementation task is filed for each.

## Context
Owns: probes/val2-irr-01 and probes/val2-irr-02 (every visibly-off cell of these cases).
- Probes: the val2 batch of docs/research/probe-requests-2026-11-08.md (questions, hypotheses, model predictions) and
  docs/research/probe-handoff-2026-11-08.md (how it was filled, CAL-01 result). Filled on excel-graph 2026-11-09 by the
  PR that filed this task.
- CAL-01 passed on every anchor (the build reproduces the cached web corpus), but in-sheet `SUM(x,-y)*2^52` residuals
  snap to 0 (numbers.md, the SUM near-zero snap): read residuals from the raw value cells, never from a SUM column.
- No engine change lands until a probe confirms its rule; no fitted special cases (handoff, "What happens next").
- Off today: 212 cells in irr-01 and 262 in irr-02, mostly the error kind (`#DIV/0!` in Excel where ours is `#NUM!`, and
  the reverse) and their `ERROR.TYPE` columns; Excel gives 136 + 198 `#DIV/0!` and 122 + 180 `#NUM!` across the two.
- Related: T-2131, T-2103 (IRR's returned iterate and error order).

- val3 batch, docs/research/probe-handoff-2026-11-09-val3.md (excel-graph fill, 2026-11-09). Owns: probes/val3-irr-01 (every visibly-off cell).

## Acceptance criteria
- [ ] Every hypothesis of the family in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing
      the probe cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
- Desktop-only questions (M14, decision 0039).
