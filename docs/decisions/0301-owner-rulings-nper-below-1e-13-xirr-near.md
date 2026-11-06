# 0301 — Owner ruling: T-3544's Gate A T.INV and F.INV points stay on Excel

- **Status:** accepted (2026-11-06, owner ruling; task T-3544).
- **Amends:** decision 0267 item 3 (its tiny-p scope narrows; see below). Decision 0188 R3's regions do not change.
- **Related:** decision 0300 (NPER T-4285 and XIRR T-3484: Excel kept as the target).
- **Context:**
  - Decision 0300 (the same day, another session) ruled "keep Excel" on nine R3 candidates, NPER's T-4285 and XIRR's
    T-3484 among them. It left T-3544 open for the owner.
  - In this session the owner was asked the same three questions and gave the same answer on NPER and XIRR, which
    0300 already records. This record covers only the third question.
  - T-3544's points are accuracy/T.INV/s1 A91 and A96, and accuracy/F.INV/s1 A164, A167 and A169. Excel is more than
    G = 1e-4 off the math there; the engine returns the math and passes Gate A through its math side.
- **Decision (owner, 2026-11-06):** asked whether to extend the tiny-p R3 ruling to these points with the math as the
  target (recommended), or to keep Excel, the owner chose "Keep Excel as target".
  - **Removed from decision 0267 item 3:** accuracy/T.INV/s1 A91 and A96. That item lists T.INV and was worded for
    every evidenced tiny-p point where Excel is more than G off. Its scope is now the probes-numeric cells T-3386 and
    T-3337 list (t1531, t1543, t3349, t3336 and the m9 cases).
  - **Settled, not removed:** accuracy/F.INV/s1 A164, A167 and A169. F.INV is not in 0267 item 3. These points were
    unruled under decisions 0188 and 0213: A164 and A169 are outside the region, and A167 is inside 0188's
    p < 1e-150 bound but unmarked. A167, F.INV(4.36982683607437E-189,1,1698856), therefore targets Excel's
    3.85E-145, not the underflowed 0.
  - Decision 0213's marked points are unchanged.
- **Consequences:**
  - T-3544 closes. T-4489 reproduces Excel's values at its five points; until then they pass Gate A through the math
    side as today, and nothing regresses.
  - No new `MATH-OVER-EXCEL` entries.
