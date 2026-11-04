---
id: T-3989
title: "Fit-lab oddfyield kernel reproducing T-2007 and T-3031 scores"
milestone: M9
lane: harness
status: todo
priority: P3
size: M
tier: sonnet
depends_on: []
---

## Goal
The fit lab scores ODDFYIELD slope candidates, so T-2007's and T-3031's scores can be reproduced without patching the engine.

## Context
T-2007 and T-3031 scored ODDFYIELD's slope candidates by patching `OddFirstSchedule::dprice`
(crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs) and dumping the schedule fields; the
lab was never committed. Add an `oddfyield` fit to tools/src/rustsheet_tools/fitlab/kernels/solvers.py (or its own
kernel module): the odd-first schedule (`OddFirstSchedule::new` in financial_bonds.rs: short and long periods, every
basis), ODDFPRICE's price with Excel's `^` (kernels/power.py `caret_pow`), YIELD's start (`fitted_yield_start`) and
Newton with the 1e-10 step cut, and the slope candidates: the exact slope, T-2007's regular-coupon slope, and T-3031's
(the regular coupon at every quasi-coupon date from the settlement's next one Q, every exponent shifted by the basis'
days from Q to the first coupon over E). Cases: probes/oddfyield-solver-sweep, probes/oddf-date-order-domain-sweep
sheet Y, probes-numeric/t3031-r4-oddfyield-long-first (financial.md "ODDFYIELD stops short of ODDFPRICE's root").

## Acceptance criteria
- [ ] `uv run rustsheet-fitlab` (or the lab's CLI) scores the three slopes on those cases, and T-3031's slope scores at least the engine's pass counts recorded in financial.md (1256 of 1260 date-order cells, 157 of 160 fresh-sweep cells) within the lab's tolerance.
- [ ] ruff and mypy strict pass; the kernel calls `rustsheet-eval snapshot`/`cases` rather than re-parsing xlsx values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's model.
