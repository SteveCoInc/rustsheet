---
id: T-4038
title: "IRR_ROOT_RESIDUAL follow-ups from T-3411's review: the non-finite extra Newton step, the cut above 1E+16, named range consts"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3411]
---

## Goal
The MINOR findings of T-3411's review (PR #3063, round 2) are settled with evidence: what IRR's fallback does when
its extra Newton step is not finite, and how far up in flow size the absolute residual cut is evidenced.

## Context
- T-3411 makes `irr_fallback` (crates/rustsheet-engine/src/functions/financial/financial_util.rs) `#NUM!` when
  |`irr_npv`| one Newton step past its stop is `IRR_ROOT_RESIDUAL` = 1E-4 or more, within `IRR_ROOT_RESIDUAL_FLOWS`
  (financial.md "IRR's fallback is `#NUM!` when NPV at its root is 1E-4 or more").
- Excel-fidelity MINORs:
  - When the extra step is not finite (P'(next) = 0 or overflow), the result is now `#NUM!`; before T-3411 the root
    was returned. No oracle cell covers that path. Find or build flows that reach it (excel-probe), and return the
    stop iterate if Excel returns there.
  - The range's largest |c| (6.4E-12..2.2E+82) takes the extremes of every scored IRR flow set, but the cut is
    bracketed and scale-tested only on flows of about 1E+5..1E+16 (probes-numeric/t3411-irr-residual-scale). Above
    that every fallback root is `#NUM!` (consistent with t3478-irr-r4-wide A140 at 1E+35). Probe a scale family past
    1E+16 (and toward 1E+300) and narrow the range or say in the tag and financial.md that it rests on extrapolation.
  - 26 of the 300 points of accuracy/IRR/r4-T-3411 have more than 148 flows (up to 199), so the cut is not applied
    to them; note it in T-3411's records, or widen the flow bound with evidence.
- Design MINOR: `IRR_ROOT_RESIDUAL_FLOWS` is a positional `(usize, f64, f64)` tuple; use named consts or a small
  struct.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the non-finite-step path and flows past 1E+16; the code and FITTED range
      follow it, with financial.md updated.
- [ ] `IRR_ROOT_RESIDUAL_FLOWS` replaced by named items.
- [ ] No eval regressions; new failing cells have owners.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The level annuities (T-4027), digits (T-3408), `#DIV/0!` (T-3407).
