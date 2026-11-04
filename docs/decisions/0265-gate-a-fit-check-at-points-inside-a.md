# 0265 — Gate A fit check at points inside a root's rounding noise

- **Status:** accepted (2026-11-04, owner ruling, T-3916)
- **Context:**
  - Decision 0188 R2 defines Gate A's envelope: within 15 significant digits of Excel, or within ε = 1e-9 relative
    of 50-digit math. Reviewer-excel-fidelity check 7 applies it to fits: "A fit that moves a point out of the
    envelope fails this item."
  - T-3906's out-of-sample probe (probes/t3906-rate-restart-oos, 240 cells) scored T-3698's RATE flat-secant restart
    (financial.md "RATE restarts once on an early flat secant"). Against the hold (no restart), the rule moves A152
    and A194 out. The hold moves A15, A26, A64, A76 and A121 out. T-3850 searched 48 narrowings
    (docs/research/t3850). Every one with 0 regressions still moves a point out, and every one that moves no point
    out regresses 9 to 16 bit-exact cells.
  - A152's root has a rounding-noise width of 7.7e-9 relative, almost 8 times ε. The noise width is
    eps·Σ|terms| / |f'(r)| / |r|, with eps the double's machine epsilon, Σ|terms| the sum of the residual's term
    magnitudes at the root, and f' the residual's slope there. The rule is 1.36e-9 from the math and the hold
    7.9e-10, so which side of ε a value lands on is chance. With roots from the decimal literals, the point that
    leaves is A144 instead, where Excel itself is 1.4e-8 from the math. Across the case, 23 of the 234 numeric points
    have a noise width above ε.
  - So at such points, the math side of the envelope measures rounding noise in the double's evaluation, not an
    accuracy defect. Decision 0188 is an owner decision, so agents could not rule on this themselves (T-3916, "Needs
    owner" in docs/status.md).
- **Options considered:**
  - *Keep the rule and leave R2 as written.* This accepts A152 and A194 as a one-off, and the same question comes
    back on the next fit near a noise floor.
  - *Revert T-3698 under a regression decision record.* That loses 22 bit-exact cells and moves five points out to
    remove two.
  - *Rule how R2 applies at the noise floor* (chosen).
- **Decision:**
  - When a fit is judged on whether it moves a point out of the Gate A envelope (reviewer-excel-fidelity check 7,
    decision 0188 R2/R4), a point whose root's rounding-noise width exceeds ε is judged against Excel only: it is
    inside when the value is within 15 significant digits of Excel's, and the math side is not used. An error against
    a number still fails, as R2 says.
  - The task that claims this treatment reports the point's noise width and how it was computed (formula and terms),
    next to the envelope result.
  - This changes how a fit is judged, not the `make check` gate. SPEC-0029's `envelope` and the Gate A tables are
    unchanged. A table point whose noise width exceeds ε still passes on either side there.
  - Applied to T-3916, A152 is outside under both the rule and the hold, so the rule no longer moves it out. Neither is
    within 15 digits of Excel's 2.9169749198680823: the case file lists the rule at 2.9169749163984258 and the hold at
    2.9169749226737123. A194 remains a real exit: the restarted run is `#NUM!` where Excel returns a number.
    The owner accepts it with the rule. Falling back to the hold on an error fixes A194 but moves A26 out instead
    (T-3850 finding 2), and T-3963 owns both cells. T-3698's restart stays as merged.
- **Consequences:**
  - The reviewer-excel-fidelity check 7 text cites this record.
  - Decision 0188 R2 gets a note pointing here.
  - T-3916 is done with no engine change. T-3850, T-3963 and financial.md's restart entry state the ruling.
  - A future fit that relies on this must show the noise width per point. Measuring the noise width automatically in
    `rustsheet-eval accuracy` is not required by this record.
