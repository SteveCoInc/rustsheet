---
id: T-4488
title: "PMT, PPMT and CUMIPMT are #NUM! where IPMT's discounted balance fails (A past the double range, 1-Q subnormal, a subnormal rate)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PMT, PPMT and CUMIPMT return `#NUM!` where Excel's do on the inputs that make IPMT `#NUM!` (T-4314), so the
visibly-off cells below pass.

## Context
T-4314 found that Excel's IPMT is `#NUM!` where its discounted balance `A = (pv+fv)/(1-Q(-nper*L))` is past the
double range, where `1-Q` is below the normal range, and at a subnormal rate (financial.md "IPMT is #NUM! where its
discounted balance fails"). Its probe shows PMT, PPMT and CUMIPMT erring on the same inputs, where the engine
returns a finite value:
- probes/t4314-ipmt-balance-overflow, Sheet1!B2, B4:B6 (A past the double range at rate 0.5, nper 2, pv
  `2*5E+307`: `PPMT(0.5,1,2,2*5E+307)`, `PMT(0.5,2,2*5E+307)`, `PPMT(0.5,2,2,2*5E+307)`,
  `CUMIPMT(0.5,2,2*5E+307,1,1,0)`);
- probes/t4314-ipmt-balance-overflow, Sheet1!C7:C8 (a subnormal rate: `PPMT((7.245817E-308-6.217028E-308),2,1E+9,1E-10,0,0)` and the PMT);
- probes/t4314-ipmt-balance-overflow, Sheet1!D3 (`1-Q` subnormal: `PPMT(1E-300,1,1E-10,1E-300)`).
Code: `period_payments` sets `interest_fails` from `discounted_balance` and `payment_factor` for IPMT only
(crates/rustsheet-engine/src/functions/financial/annuity.rs, `TODO(T-4488)`); PMT goes through `compute_payment`,
CUMIPMT through cumulative.rs. Check every gated PMT/PPMT/CUMIPMT cell where those helpers decline before applying
the rule (PPMT's `discounted_principal` past-range form, `PPMT(1E+307,7,292,176644,89)` = 0, keeps a value where
PMT overflows).

## Acceptance criteria
- [x] The cells above pass, or the ones that do not are named with their owner; no other cell regresses
      (`make eval-baseline`).
- [x] financial.md records which of PMT, PPMT and CUMIPMT follow the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- IPMT (T-4314).

## Outcome
PMT (`compute_payment`), PPMT (`period_payments`) and CUMIPMT (`cumulative_of`) now return `#NUM!` where
`discounted_fails` (the discounted balance `A` or factor `d` declines), the rule IPMT already had. All 7 cells
of probes/t4314-ipmt-balance-overflow (B2, B4:B6, C7:C8, D3) pass; `make eval-baseline` shrank by those 7
cells and refused nothing. The PPMT past-range `discounted_principal` cells stayed (no regression). CUMPRINC is
unprobed there and unchanged. financial.md updated.
