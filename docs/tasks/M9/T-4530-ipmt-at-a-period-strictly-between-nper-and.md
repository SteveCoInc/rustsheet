---
id: T-4530
title: "IPMT at a period strictly between nper and nper + 1: probe whether it is a value like PPMT's band or #NUM!, and match"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IPMT's period domain between nper and nper + 1 has Excel evidence, and the engine matches it: a
value (and which split gives its digits) or `#NUM!`. docs/excel-semantics/financial.md "IPMT and
PPMT take a period up to nper + 1" then is VERIFIED for IPMT as well as PPMT.

## Context
- financial.md (written by T-4109, done): PPMT is a value for nper < period < nper + 1
  (accuracy/PPMT/s1 A112 `PPMT(0.0798622533510215,139.5,139,-115683.067689867,0,0)` =
  8890.724785738455, A118 = 5.466510612214358) and `#NUM!` at nper + 1 (A128, A137, A139). IPMT is
  `#NUM!` at nper + 1 (accuracy/IPMT/s1 A113, A114, A117, A120, A123, A139) but no Excel cell has an
  IPMT period strictly inside the band: "Unprobed: IPMT at a period strictly between nper and
  nper + 1 (the engine takes PPMT's bound for both)". Status line: "IPMT's band UNVERIFIED".
- Ours: `period_payments` in crates/rustsheet-engine/src/functions/financial/annuity.rs returns
  `#NUM!` for both only when `period < 1` or `period >= nper + 1`, so IPMT in the band is the
  interest half of the discounted split at a fractional period past the last payment: a number no
  Excel cell backs.
- Opus: if Excel gives a value, its digits come from one of the annuity forms (`discounted_split`,
  the exact split, `balance_interest`, the FV recurrence), and choosing among them past nper is
  formulation work; an IPMT bound of nper (unlike PPMT) is also possible. Neighbours: T-1528
  (blocked, annuity last digits), T-2730, T-2950, T-3186 (PMT/PPMT last digits), T-3779 (arg-pairs
  batch), T-4491 (IPMT near the double range): none covers this band.
- Use the excel-probe skill (excel-graph oracle). Add the points to the IPMT Gate A table
  (eval/accuracy/tables/IPMT) when they are numbers, so their math values are checked.

## Acceptance criteria
- [x] A committed probe case with an excel-graph sidecar holds IPMT and PPMT side by side at
      periods nper + 0.5, nper + 1E-9, the double just below nper + 1, nper + 0.999, and nper itself,
      for integer and fractional nper, types 0 and 1, rates 0, 1E-10, 0.05 and -0.5, nonzero fv.
- [x] Every new cell is within half a unit in the 15th significant digit of Excel's value with the
      right type and error, or each remaining visibly-off cell is owned by a filed task.
- [x] financial.md's entry states IPMT's band from the evidence and drops "IPMT's band UNVERIFIED".
- [x] No new known-failure lines in any eval/baseline/*.txt and no new Gate A known failure other
      than the new case's owned cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Last-digit (ulp) residue inside the band: file it `defer: phase-1b`.
- CUMIPMT/CUMPRINC period bounds.

## Outcome
- Excel's IPMT is a value at every period strictly between nper and nper + 1, like PPMT: probes/t4530-ipmt-band
  (160 cells, IPMT and PPMT side by side at nper, nper + 1E-9, nper + 0.5, nper + 0.999 and the double below
  nper + 1, rates 0, 1E-10, 0.05, -0.5, types 0 and 1, nper 12 and 12.5, fv 98.7654321; excel-graph 2026-11-07).
- Predictions were the engine's current outputs (H_value), committed in `docs/research/t4530/predictions.tsv` before
  the fill beside H_nper (IPMT `#NUM!` past nper). Excel matched H_value on all 160 cells, bit for bit, so no engine
  change: `period_payments`' shared bound was already right. No fit, nothing tuned.
- The case's math sidecar (`rustsheet-accuracy math --function IPMT --function PPMT`) holds all 160 math values, so
  Gate A's envelope checks the band's points (instead of a new IPMT table batch, which would need its own fill).
- financial.md "IPMT and PPMT take a period up to nper + 1": IPMT's band is stated from the evidence, VERIFIED.
- Eval: one new case, 160/160 pass; no baseline line changes.
