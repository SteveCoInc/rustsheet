---
id: T-4683
title: "T-4504 review follow-ups: tighten the AMORLINC/AMORDEGRC salvage-equals-cost citations and probe a salvage one ulp below the cost"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4504]
---

## Goal
financial.md "AMORLINC and AMORDEGRC check order" cites exactly the cells behind each claim, and the boundary of the
salvage-equals-cost rule (exact equality) is probed or marked unprobed.

## Context
Excel-fidelity review of T-4504 (PR #3732), probes/amor-salvage-equal-cost:
- The first bullet cites "A1-C18, E7-E9" for "both functions are 0 at periods 0, 0.0001, 0.5". Columns A/C cover
  AMORDEGRC at 0, 1-8 and 1.5 only; AMORDEGRC at 0.0001 and 0.5 is evidenced by F1, F7, F8 (the 485 + 0 sums). "1 to 8"
  for AMORLINC generalises: column B has periods 1-3, 5, 7, 8 (not 4 or 6). Cite F1/F7/F8 and narrow the wording.
- "AMORDEGRC at rates 0.5, 0.6, 1 and 2 is #NUM! (periods 0, 0.5 and 1)": only rate 0.5 is probed at periods 0 and 0.5
  (E4, E5); the others at period 1. Say so.
- The rule compares `salvage == cost` exactly (crates/rustsheet-engine/src/functions/financial/amortization.rs,
  tools/src/rustsheet_tools/accuracy/defs/amortization.py). A salvage one ulp below the cost, or an equality reached by
  rounding, is unprobed. Probe it (excel-probe skill, predictions first) or mark it unprobed in the entry.

## Acceptance criteria
- [ ] The entry's citations match the cells (F1/F7/F8 added; AMORLINC periods and AMORDEGRC rate/period pairs stated as probed).
- [ ] The one-ulp-below and rounded-equality cases are probed (and pass) or marked unprobed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The lift's work charge for salvage == cost (over-charges a full walk; safe, by design).
