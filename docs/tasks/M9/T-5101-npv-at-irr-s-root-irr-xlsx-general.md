---
id: T-5101
title: "NPV at IRR's root (IRR.xlsx General!M17) is a Phase 1 exception owned by IRR's fallback fit T-2928"
milestone: M9
lane: quality
status: done
priority: P1
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
The foundational tier (decision 0355) no longer counts IRR.xlsx `General!M17` against NPV: that cell's only miss is
the rate IRR's fallback path returns, an expert function's open fit.

## Context
- Full eval of main e53d43fa (2026-11-08): the foundational tier's only number outside the envelope is
  ironcalc-calc/FINANCIAL/IRR.xlsx `General!M17` `=NPV(L17,A17:F17)`, Excel -3.066e-11, ours -3.485e-13.
- L17 is `=IRR(A17:F17)`, which takes IRR's fallback path (T-2131 Context, gap triage 2026-10-27); T-2928 owns that
  fit and is blocked (time-boxed, decision 0056) until T-3117's evidence.
- Owner ruling (2026-11-08): list the cell as a Phase 1 exception owned by T-2928.

## Acceptance criteria
- [x] Decision 0359 records the rule and the cell.
- [x] eval/milestones/phase1-exceptions.tsv lists `ironcalc-calc/FINANCIAL/IRR.xlsx!General!M17	0359	T-2928`, with a
  comment; the list's header and `phase1_exceptions.rs` describe the new kind.
- [x] The cell keeps its eval/baseline `F` and `E` lines (the list hides nothing from the gate).
- [ ] `make check` green; reviewers approve.

## Outcome
- Decision 0359 (amends 0134): a near-zero residue cell whose only miss is an input an expert function computes with
  an open fit may be a Phase 1 exception, owned by that fit's task. First and only cell: IRR.xlsx `General!M17`,
  owned by T-2928.
- eval/milestones/phase1-exceptions.tsv gains that line and its comment; its header and the module doc of
  crates/rustsheet-eval/src/phase1_exceptions.rs name the new kind. T-2928 notes the line, to be removed when its fit
  lands and the line goes stale.
- Effect: the foundational tier's envelope misses go from 1 to 0 on the next full eval (NPV leaves the contributors);
  TEXT's 28 and EDATE's 1 wrong-type cells remain, so the tier stays unmet. eval/baseline is unchanged: the cell is
  still a known failure (`F` and `E`).
- Eval delta: none.
