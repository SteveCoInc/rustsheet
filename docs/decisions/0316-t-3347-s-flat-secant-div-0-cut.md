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
  - Attempt 2 is a cut on the largest |NPV| of the run (T-3411 had seen `#DIV/0!` above about 1E+228). No other
    feature tried separates better: v^(n-1) at the far step, or |NPV| times |d| there.
  - **Hold-out (R4.1).** The fit lab's split (`rustsheet_tools.fitlab.holdout.is_holdout`, keyed on the case id
    without `.case.json` and `Sheet!A1`) over the 209 cases with IRR cells, at the default margin of 5 points.
    - The cut is placed from the train side only. The largest train `#NUM!` run reaches 1.556E+226 (probes/
      t3406-irr-num-boundary A65), and the smallest train `#DIV/0!` run above it 6.57E+230 (t3406 A81).
      3E+226 lies in that bracket. The test side's 5.16E+226 (probes-numeric/t3411-irr-r4-fresh-4 A86) is in it too.
    - Main -> rule, cells passing: train 2,086 -> 2,099 of 2,580 (80.85% -> 81.36%; type/error mismatches 344 -> 331);
      test 883 -> 889 of 1,080 (81.76% -> 82.31%; 155 -> 149).
    - The test rate is within the margin of the train rate. The rule beats the no-rule candidate on test, and there is
      no correctly rounded candidate for an error type. 0 cells are worse on either side: 13 train and 6 test cells
      move to Excel's `#DIV/0!`.
  - **Range (R4.3).** The cut applies only to runs from 0.1 (the default guess or the retry from 0.1): every
    evidenced run starts there. It also applies only to 27..144 flows with a largest |flow| of 1.3E+4..6.7E+14, the
    36 such runs past 1E+200 in eval. A run from any other guess fails as before.
  - **Fresh sweep (R4.2).** probes-numeric/t3347-irr-r4-fresh is 150 long conventional series in six strata
    (docs/research/t3411/gen_sweep.py, seed "T-3347/r4-fresh"), drawn and filled (excel-graph, 2026-11-06) only after
    the cut was frozen. With the cut, 90 of 150 cells pass, against 82 without it: 8 move to Excel's `#DIV/0!` and none
    away. Basis under decision 0267 item 5: it is the R4.2 fresh sweep the fit owes, and the fit fixes T-3347's own
    Decision 0267 cells (A131, A138), so the case is "for the cells it owns". That follows the practice of T-3858,
    T-3764, T-4349 and T-3988, which committed their R4.2 sweeps after the freeze. Its cells are evidence: none joins
    T-3347's Decision 0267 set. Its 60 off cells are owned by T-3548, beside T-3411's sweeps from the same generator.
  - **Gate A envelope (R2)** on that sweep, against its 50-digit roots: cells outside drop from 68 to 60. All 60 are
    errors against Excel's number or another error, and none is a number outside both 15 digits and 1e-9 of the math.
    No point moves out. `rustsheet-eval accuracy` is unchanged: no table point reaches the rule.
  - **R4.4.** Only the error type is scored, so the ulp floor does not arise.
  - Against the rule: two `#DIV/0!` runs peak below the cut (1.3E+215, 3.0E+221). On 11 runs past the cut (6 old,
    5 fresh; roots -4% to -11%) Excel returns the root. There the engine's `#NUM!` becomes `#DIV/0!`, wrong either
    way. So the cut is a proxy, not Excel's mechanism, and a fit, not a structural rule.
  - Not a keep-matched branch: the domain comes from the run's state (a zero step, then a flat secant), not from the
    set of cells that matched before.
- **Decision:** `irr_secant` returns `SecantStop::DivZero` (the call is `#DIV/0!`, with no retry and no fallback)
  when a run from 0.1 goes flat after a zero step and the largest |NPV| it evaluated is above
  `IRR_FLAT_DIV_ZERO_NPV` (3E+226), inside `IRR_FLAT_DIV_ZERO_FLOWS`. It carries one `FITTED(T-3347, ...)` tag;
  `scripts/check_fitted.py --update` records it.
- **Consequences:**
  - eval/baseline/probes.txt loses five known-failure lines (t3406-irr-num-boundary A66, A67, A68, A81, A82). The
    other 14 fixed cells and the new sweep are in probes-numeric, which is not gated.
  - If T-3548 finds Excel's path on the long series (the 11 roots past the cut), that path should replace this cut.
    The proxy then goes, and quality/fitted.json falls by one.
