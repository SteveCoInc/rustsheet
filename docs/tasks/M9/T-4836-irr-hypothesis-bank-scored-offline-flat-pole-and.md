---
id: T-4836
title: "IRR hypothesis bank scored offline: flat, pole and cap handlers (H1-H12)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4834, T-4835]
---

## Goal
A shortlist of at most four complete IRR models, each scored offline on type and bits with hold-out verdicts, and the cells where they disagree.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 2, hypotheses H1-H12 of section 7.

## Acceptance criteria
- [x] Every hypothesis H1-H12 and the sensible combinations scored by `irr-type`; the table (train/test type and
      bits, cells gained and lost against main) in the Outcome and in financial.md as refutations or leads.
- [x] The shortlist with each model's prediction on every probe cell, committed under docs/research/irr-2026-11-07/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Oracle probes (T-4838).

## Outcome
- irr_driver.py (`irr-type` fit): new `Options` handlers for H3 (RATE's restart), H8 (VB's flow-scaled stop), H10
  (`#DIV/0!` on a second flat), H11 (no retry after a pole), H12 (Newton fallback from v = 0 with the 1e-4 cut,
  reusing `irr_fallback_newton`), H13 (keep the older iterate), H14 (reuse the slope) and H15 (clamp inside the
  pole), and a 22-candidate `BANK` (each handler on the base at cap 200 and on RATE's driver). H1, H2, H4-H7 were
  already candidates (T-4834). Tests: every bank handler is inert on a clean run; H12 returns `IRR({42,1,1,-1})`.
- docs/research/irr-2026-11-07/bank.py scores every candidate cell by cell (type, bits, train/test, gained/lost
  against the engine) and writes bank.tsv and predictions.tsv (the shortlist's value on all 2,555 cells).
- Result (financial.md "IRR's failure handlers scored offline"): RATE's driver + the Newton fallback is the lead,
  2,046 typed / 1,942 bit-exact against the engine's 2,000 / 1,857, ahead on train (1,454 / 1,385 vs 1,432 / 1,338)
  and test (592 / 557 vs 568 / 519), +94 / -48 typed cells. H8, H13, H14, H2/H10 and H11 are refuted on this data;
  H3 is neutral. Shortlist: RATE's driver, + H12, + H5 + H12, + H5; they disagree on 235 cells. No numeric fit was
  selected (every constant is RATE's or the engine's), so no FITTED tag; T-4838 probes the disagreement cells.
- lineage.md's H15 row is corrected in the financial.md entry (formulajs's -0.999999999 clamp is the far side of the
  pole; r = 1000 was scored). The T-4835 review MINORs (blank History cells, "E (fitted)") are left to the next edit
  of lineage.md, which is not on this branch's base.
- Eval delta: none (tools and docs only).
