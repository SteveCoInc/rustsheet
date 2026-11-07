---
id: T-4676
title: "Discriminating probe for BETA.INV's 0.5 at a DBL_MIN shape: candidate rules with predictions committed before the fill (for T-4663)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
An oracle-filled probe tells which rule gives Excel's 0.5, and where it gives `#NUM!` instead, for BETA.INV and
BETAINV at a shape of DBL_MIN. Each candidate rule's predictions are written down before the fill. T-4663 then
implements the rule that survives, or records that none did. It does not fit the 13 cells on file.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main ebe81baf6). T-4663 is a numeric-fitting group with too few Excel
  points. T-2808's Outcome says no probe was filled, because the candidate was refuted in-sample and "there was no
  rule to commit predictions for". Triager rule: such a group "is not attempted blind". It depends
  on a research-lane sweep, and none was filed.
- Cells on file. All are Gate A known failures (T-4663) in eval/accuracy/known-failures.tsv, plus
  probes/t2435-dist-extremes S!A73, A111:
  - Excel 0.5:
    - BETA.INV/s1 A115 `(0.81771115485045, DBL_MIN, 8.15218150358274)`
    - BETA.INV/s1 A118 `(0.0131334898328037, 0.733069178470246, DBL_MIN)`
    - BETA.INV/s1 A125 `(0.652010757756514, DBL_MIN, 2.21183760221804)`
    - BETA.INV/s1 A128 `(0.440528985304203, 1.5844517339205, DBL_MIN)`
    - BETA.INV/s1 A135 `(0.70533546313545, DBL_MIN, 0.118581547662447)`
    - BETA.INV/s1 A138 `(0.0023103926432828, 12.5755736233707, DBL_MIN)`
    - BETAINV/s1 A115 `(0.682406674876215, DBL_MIN, 0.238930059589786)`
    - BETAINV/s1 A118 `(0.00331784802833339, 0.260453610511014, DBL_MIN)`
    - BETAINV/s1 A128 `(0.0264083784045944, 2.28800746041701, DBL_MIN)`
    - BETAINV/s1 A135 `(0.692122660100602, DBL_MIN, 0.517627748014771)`
    - BETAINV/s1 A138 `(0.00652094707679638, 1.79288113823995, DBL_MIN)`
  - Excel `#NUM!`: BETAINV/s1 A125 `(0.826804627471484, DBL_MIN, 0.478796631378503)`.
  Here DBL_MIN is 2.22507385850721E-308. The 0.5 cells have a tiny first shape with p above 1/2, or a tiny second
  shape with p below 1/2. That is the orientation in which AS 109 flips. A125 against A135 is the open question.
- What T-2808 ruled out (statistics.md "BETAINV is #NUM! where its quantile underflows, but 0.5 at a tiny alpha and a
  high p (T-2435)"): the rule "AS 109's first approximation is 1 and the search resets to the midpoint" (12 of 13).
  Also ruled out: "the first Newton step at 0.5 cannot move". The ln of the density factor is 707.34 at A125 and
  707.37 at A135, so that rule does not separate them either.
- Work: before the fill, state two or three candidate rules. Each must separate A125 from A135. Examples: Excel's
  own double cdf at the start, BETA.DIST(0.5, the shapes), rounding to 1 or not; the AS 109 start's overflow; a
  bound on p against I_0.5. For each rule, write its predicted value or error for every cell in the case
  `description`. Then design the grid so the rules disagree on at least a third of it:
  - `BETAINV(p,DBL_MIN,b)` and `BETA.INV(p,DBL_MIN,b)`: p from 0.55 to 0.95, b from 0.1 to 10. Bracket A125 and
    A135: b 0.4788 to 0.5176, p 0.69 to 0.83.
  - The mirrored `(p, a, DBL_MIN)` for p from 0.001 to 0.45.
  - `BETA.DIST(0.5,DBL_MIN,b,TRUE)` and `BETA.DIST(0.5,a,DBL_MIN,TRUE)` at the same shapes.
  - Shape 1E-300 and 1E-100 against DBL_MIN at a few points.
  The case goes in `probes-numeric/t4676-beta-inv-tiny-shape-half.case.json`, tag M9, about 150 cells.
- Code the result informs, not changed here: crates/rustsheet-engine/src/functions/statistical/beta.rs
  `beta_quantile`, inverse_beta.rs.

## Acceptance criteria
- [ ] The case exists with the candidate rules and their per-cell predictions in its `description`, committed
      before the fill. Its `.oracle.json` comes from excel-graph with no pending cell, and its `.math.json` comes
      from `rustsheet-accuracy math`.
- [ ] statistics.md's T-2435 BETAINV entry cites the case. It gives each candidate's hit count on the case, and
      names the surviving rule, or says that none survived and gives the cells that refute each one.
- [ ] T-4663 names the new case in its cell list, and every visibly-off cell of the case has an owner
      (`scripts/new_case_owners.py` passes).
- [ ] If the oracle is unavailable, this task is `blocked` with the reason. `make check` green; reviewers approve.

## Out of scope
- Implementing the rule (T-4663).
- The far-tail saturation of the search (T-4662, T-4675) and BETA.INV's 1-2^-53 at p near 1 (T-3351).
