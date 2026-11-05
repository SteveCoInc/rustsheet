# 0297 — Composite cells' math is the outer node at the inner doubles

- **Status:** proposed (2026-11-05, T-3993). It amends SPEC-0029 "Math references for eval cells" on which cells get a
  math value. It becomes accepted once the per-option counts below are filled in from `docs/research/t3993/score.py`
  and reviewers approve.
- **Context:**
  - SPEC-0029 gives a 50-digit math value only to a cell that is one call `=FUNC(args)` over literal or exact-input
    arguments. A nested call (`F.INV(F.DIST(x,d1,d2,TRUE),d1,d2)`) or arithmetic between calls
    (`BINOM.DIST(k1,n,p,TRUE)-BINOM.DIST(k2,n,p,TRUE)`, `1-BINOM.DIST(...)`, `EXP(LN(x)/3)`) gets none, so decision
    0188 R1(c) counts every such visibly-off cell as outside the envelope, even where Excel and the engine both sit
    within 1e-9 of the math (T-3993's Context lists the cases).
  - Which value is "the math" of a composite is a choice:
    - (a) the exact composition: no inner rounding, so `F.INV(F.DIST(x))` is x;
    - (b) the outermost node's math at the doubles its inner nodes round to: each inner call's 50-digit value
      rounded to the nearest double (a subnormal call result is 0, numbers.md "Subnormal results become 0"), each
      inner `+ - * /` applied to those doubles with Excel's operator model (the fit lab's `arithmetic`, x87 double
      rounding);
    - (c) the outer math at Excel's own inner double, which tools cannot use (they never read oracle values as math
      inputs).
  - Under (a) an ill-conditioned outer function amplifies the inner rounding: `F.INV` at d1 = 24451, d2 = 3106 maps
    one ulp of its probability to many ulps of x, so even a pipeline that rounds every step correctly is far from
    (a). Under (b) the inner rounding is part of the reference, which is what R1(c) asks: is the outer function
    accurate on the input it was actually given?
  - Scores (`docs/research/t3993/score.py`; Excel within 1e-9 relative / engine within 1e-9 / both): TODO(T-3993):
    fill in from the score run.
- **Decision:** a composite cell whose functions all have a Gate A definition gets option (b) as its math value
  (`tools/src/rustsheet_tools/accuracy/composite.py`). A composite that calls a function without a definition gets
  none and is counted as skipped (`no definition for ...`). `^` with a call result as an operand is not read
  (Excel's `^` is not correctly rounded, numbers.md), nor is an inner non-integer result for an integer parameter
  (listed under the sidecar's `skipped`).
- **Consequences:**
  - `rustsheet-accuracy math` writes composite values; `math --check` requires them like one-call values.
  - A composite whose inner function gains a definition later (GAMMA, F.DIST.RT; T-3798, T-3783) gets its value at
    the next regeneration with no further work.
  - Option (a) stays available (`composite.compute(rounded=False)`) for scoring only.
