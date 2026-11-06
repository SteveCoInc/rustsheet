# 0316 — T-3347's flat-secant #DIV/0! cut for IRR: a fitted proxy under decision 0188 R4

- **Status:** accepted (2026-11-06, T-3347). It applies decision 0188 R4 to one new fitted constant and raises
  quality/fitted.json by one tag (financial_util.rs, 10 -> 11).
- **Context:**
  - Decision 0267 gave T-3347 nine Tier A cells of probes-numeric/t3347-irr-conventional-holdout (A123, A126,
    A130-A132, A134, A135, A138, A143), all `#DIV/0!` in Excel. T-3407's overflow rule already matched seven. A131 and
    A138 were still `#NUM!`. On both, IRR's secant from 0.1 jumps to d = -5.5E+7 and -8.5E+6, where NPV is 7.5E+275
    and 1.8E+278 (finite), comes back to d = 0.0919, takes a zero step and goes flat (d1 = d0, f1 = f0: 0/0).
  - Attempt 1, structural (every 0/0 flat step is `#DIV/0!`): moves the two cells, breaks about 150 where Excel is
    `#NUM!` or returns the root. Refuted.
  - Attempt 2, a cut on the largest |NPV| of the run (T-3411 had seen `#DIV/0!` above about 1E+228). No other
    feature tried separates better: the run's d or v^(n-1) at the far step, or |NPV| times |d| there. Over every IRR
    cell in eval, a cut anywhere in 1.556E+226..5.16E+226 moves 19 cells to Excel's `#DIV/0!` and none away. The cut
    is 3E+226, applied only to 27..147 flows with a largest |flow| of 1.3E+4..6.7E+14 (R4.3; outside that range the
    run fails as before).
  - Hold-out (R4.1, R4.2): no split of the old cells was fixed before scoring, so all of them are train. The hold-out
    is a fresh sweep, drawn and filled only after the cut and range were frozen: probes-numeric/t3347-irr-r4-fresh,
    150 long conventional series in six strata (docs/research/t3411/gen_sweep.py, seed "T-3347/r4-fresh"),
    excel-graph 2026-11-06. With the cut, 90 of 150 cells pass, against 82 without it: 8 move to Excel's `#DIV/0!`
    and none away. The rule's error type is the only thing scored, so R4.4's ulp floor does not arise.
  - Gate A envelope on that sweep (R2, against its 50-digit roots): cells outside drop from 68 to 60. All 60 are
    errors against Excel's number or another error, and none is a number outside both 15 digits and 1e-9 of the math.
    No point moves out. `rustsheet-eval accuracy` is unchanged: no table point reaches the rule.
  - Against it: two `#DIV/0!` runs peak below the cut (1.3E+215, 3.0E+221). On 12 runs past the cut (7 old, 5 fresh;
    roots -4% to -11%, and one at 1.6E-4) Excel returns the root. There the engine's `#NUM!` becomes `#DIV/0!`,
    wrong either way. So the cut is a proxy, not Excel's mechanism, and a fit, not a structural rule.
  - Not a keep-matched branch: the domain comes from the run's state (a zero step, then a flat secant), not from the
    set of cells that matched before.
- **Decision:** `irr_secant` returns `SecantStop::DivZero` (the call is `#DIV/0!`, with no retry and no fallback)
  when it goes flat after a zero step and the largest |NPV| it evaluated is above `IRR_FLAT_DIV_ZERO_NPV` (3E+226),
  inside `IRR_FLAT_DIV_ZERO_FLOWS`. It carries one `FITTED(T-3347, ...)` tag; `scripts/check_fitted.py --update`
  records it.
- **Consequences:** eval/baseline/probes.txt loses five known-failure lines (t3406-irr-num-boundary A66, A67, A68,
  A81, A82). The other 14 fixed cells and the new sweep are in probes-numeric, which is not gated.
  If T-3408 or T-3548 finds Excel's path on the long series (the 12 roots past the cut), it should replace this cut.
  The proxy then goes, and quality/fitted.json falls by one.
