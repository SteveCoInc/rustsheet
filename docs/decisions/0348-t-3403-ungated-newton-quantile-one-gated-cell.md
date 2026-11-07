# 0348 — T-3403: ungated Newton quantile, one gated cell one ulp off and five farther

- **Status:** accepted (2026-11-07, T-3403). It records an owner ruling under the standing rule that a regression
  outside decision 0083's allowance needs its own record; it adds no rule.
- **Context:** T-3403 makes Excel's lower-tail gamma quantile (CHISQ.INV, CHISQ.INV.RT, CHIINV, GAMMA.INV, GAMMAINV)
  at shapes 1 to 20 and lower tails up to 1.0000000827E-10 the nearest-F Newton steps from the leading-factor root
  alone (`incomplete_gamma::closed_form::excel_newton_quantile`; docs/excel-semantics/statistics.md, "Ours (T-3403)").
  It drops T-2561's root-first check (`TINY_QUANTILE_CHECK`) and T-3311's 1E-6 start cut (`NEWTON_START_MAX`) and
  shape band, both fits: quality/fitted.json falls by one. `NEWTON_P_MAX` keeps T-3311's measured top of p.
- **Evidence:** over 716 literal-argument quantile cells in the region, 491 bit-exact and 3 over 16 ulp against 352
  and 116 for the gated rules; T-3311's fresh sweeps 414 of 511 against 272; T-3403's own fresh sweep
  (probes-numeric/t3403-fresh-sweep, 180 cells, committed frozen at 3fa09266 before its fill) 132 against 77.
- **Eval, per cell against the T-3412 branch (7dbb70677):** gated 1 fixed, 11 closer, 1 visibly fixed
  (gen-m9/c2974c061384fdd9 A355 `CHISQ.INV(1E-20,10)`, 5 ulp to 0), 1 lost, 5 farther; ungated 189 fixed, 178 visibly
  fixed. Gate B 468 of 506 on `rustsheet-eval run` (the T-3412 branch: 467 on `make eval`).
- **The regressed gated cells** (all numeric, within 15 digits; ulps of Excel, before to after):
  - gen-m9/eced4bb6a0234c9e A265 `CHIINV(1-1E-10,7)`: 0 to 1 (a new baseline line);
  - gen-m9/06b4e8f5de409d5e A311 `CHISQ.INV(1E-16,9)` 1 to 3; 530c636749a85209 A90 `CHIINV(1-1E-16,9)` 1 to 2;
    7bd219ec0c0c5e22 A235 `CHISQ.INV(1E-16,8)` 2 to 3; caa1eed48b1883fe A151 and ecc87f80077959e0 A119
    `CHISQ.INV.RT(1-1E-10,5)` 4 to 5 (already known failures).
- **Ungated cells newly visibly off (18), not covered by the ruling below:** six quantiles of probes-numeric/
  t3311-gamma-inv-lower-tail-cdf and t3311-fresh-sweep-2 (2 to 12 ulp: W!A37, W!G55, W!H55, W!G71, W!H71, S!A321),
  where nearest-F picks a different iterate of the cycle than Excel; and 12 GAMMA.DIST cells that take those
  quantiles as input. Against them 178 ungated cells are visibly fixed. T-5014 owns them.
- **Owner ruling.** On 2026-11-07, in this session (https://claude.ai/code/session_017265sRpNacu3bZTBp3fnoU), asked
  about the gated cells the gamma and chi-square changes move off Excel (this PR's six, named with T-3412's), the
  owner chose "Allow, with records".
- **Decision:**
  - `rustsheet-eval update-baseline --allow-regressions` runs once in the T-3403 PR, for A265 alone (the other five
    are already known failures and stay so).
  - quality/fitted.json falls by one (`scripts/check_fitted.py --update`; the count only fell).
- **Consequences:** eval/baseline/gen-m9.txt gains exactly the A265 line and loses the fixed cells' lines; reviewers
  confirm the baseline diff adds no other line. Decision 0210's `NEWTON_START_MAX` and T-2561's `TINY_QUANTILE_CHECK`
  are retired; T-5014 owns which iterate Excel returns.
