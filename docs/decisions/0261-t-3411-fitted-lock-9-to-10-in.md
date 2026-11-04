# 0261 — T-3411: fitted lock 9 to 10 in financial_util.rs for IRR_ROOT_RESIDUAL

- **Status:** accepted (2026-11-04, T-3411). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3411 makes IRR's fallback (`irr_fallback` in
  crates/rustsheet-engine/src/functions/financial/financial_util.rs, Newton on P(v) from v = 0) `#NUM!` at a root
  whose NPV, as `irr_npv` evaluates it, is 1E-4 or more in magnitude (docs/excel-semantics/financial.md, "IRR's
  fallback is `#NUM!` when NPV at its root is 1E-4 or more"). The cut `IRR_ROOT_RESIDUAL` sits between evidenced
  cells, so it is a fit (decision 0188 R4):
  - It carries `FITTED(T-3411, 710 cells, IRR whose secant runs fail: 2..150 flows, |c| 1E-6..1E+13; cut bracketed
    9.54E-5..1.106E-4)`. Over every IRR cell in eval whose secant runs fail, no Excel return has a residual above
    9.54E-5 (probes-numeric/t3478-irr-r4-wide A83), and every cell from 1.106E-4 up is an error
    (probes/t3406-irr-num-boundary A131).
  - No hold-out split: every IRR case was used to find the bracket (task file, "R4 plan, round 3"). The out-of-sample
    test is the fresh sweep probes-numeric/t3411-irr-r4-fresh-3, drawn and oracle-filled after the freeze at
    262b20cfa and scored once against main: 0 cells lost, 3 gained, type mismatches 52 to 49.
- **Decision:** quality/fitted.json rises by one for financial/financial_util.rs (9 to 10), for this one new fit,
  written by `scripts/check_fitted.py --update`.
- **Consequences:** one counted tag. The fitted ratchet stays one-way for new fits, each with its own record. No
  gated or ungated cell is lost (`make eval-check`: 0 regressions). The rule fixes 12 cells over the existing IRR
  sweeps and 3 on fresh-3.
