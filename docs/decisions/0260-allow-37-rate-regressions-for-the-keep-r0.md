# 0260 — Allow 37 RATE regressions for the keep-r0 secant model

- **Status:** accepted (2026-11-03, T-3963). Takes effect when the T-3963 reviewers approve its PR.
- **Context:**
  - T-3963 changes `rate_secant` (financial_util.rs): a secant step whose residual equals the last one keeps the
    older iterate r0 instead of shifting the pair (financial.md "RATE keeps its older iterate when a step leaves the
    residual unchanged"). It replaces T-3698's flat-secant restart and T-3348's hold on two distinct iterates. It is
    a structural model (reviewer check 7, kind (c)); its hold-out and fresh sweep are in the task's Outcome.
  - Gains: probes `fixed 107`; probes/t3963-rate-keep-r0-fresh, drawn after the freeze, 59 of 240 bit-exact where
    the merged rule gets 9. On the 2,400 literal RATE oracle cells: 1,914 -> 1,992 bit-exact, train 1,357 -> 1,416,
    test 557 -> 576. Gate A: 0 failing points before and after.
  - Losses: 37 gated probes cells the merged rule had bit-exact.
    - 19 are 1 to 4 ulp off (under decision 0112's reproducibility floor): t3348-rate-fv-nonzero A183,
      t3698-rate-flat-restart A44, A50, t3850-rate-same-state A30, A104, A130, A131, A132, A168, A179, A203, A211,
      A212, A237, and 5 t3906-rate-restart-oos cells.
    - 13 are more than 4 ulp off: t3698-rate-flat-restart A52, t3698-rate-flat-restart-fresh A29,
      t3850-rate-same-state A43, A44, and t3906-rate-restart-oos A15, A64, A103, A105, A113, A123, A131, A140, A223.
      Most are nper-1 or nper-2 calls whose residual is rounding noise everywhere (A15, A64: pv = -pmt, fv = 0).
      Two are visibly off (A15, A64).
    - 5 are of the wrong type: t3698-rate-flat-restart A1, A2, t3579-rate-residual-nper1 A43 and
      t3698-rate-flat-restart-fresh A21 (a number where Excel is `#NUM!`; the run ends on a zero step that the cap's
      test returns), and t3906-rate-restart-oos A143 (`#NUM!` where Excel returns a number).
  - No refinement scored on the evidence keeps both: a hybrid that restarts like T-3698 inside its steps and |f|
    band is worse on train (-2), test (-3) and the fresh sweep (46 against 59). Keeping the old outcome only where it
    matched would be a keep-matched branch (decisions 0056, 0082).
- **Decision:** take the keep-r0 model and record the 37 cells as known failures with `make eval-baseline
  --allow-regressions` (the only baseline lines removed are these cells' passes; the visible-parity lock rises by 1
  for t3579-rate-residual-nper1 and records the new case). T-3424 owns the wrong-type cells and the zero-step
  question; T-3963's follow-up owns the noise-only restarts.
- **Consequences:**
  - The engine no longer has T-3698's restart constants (`RATE_RESTART_*`), so its FITTED tag goes.
  - A later model that wins these cells back must not lose the keep-r0 gains; it is scored on the same cases.
