# 0301 — Owner rulings: NPER below 1E-13, XIRR near zero roots and the Gate A T.INV/F.INV points stay on Excel (no new R3 regions)

- **Status:** accepted (2026-11-06, owner rulings; tasks T-4285, T-3484, T-3544).
- **Amends:** decision 0267 item 3 narrows (see below). Decision 0188 R3's regions do not change.
- **Context:** three R3 candidates (decision 0188 R3: where Excel's relative error against 50-digit math exceeds
  G = 1e-4, a new region needs an owner ruling) were blocked on the owner.
  - **NPER, 0 < rate from 1E-14 to 1E-13 (T-4285).** Excel is 7E-4 to 7E-3 off the math at 24 t3367 cells. The engine
    matches Excel to the bit there: no cell of A99 to A125 fails in a probes-numeric run at main c97e0b9c.
  - **XIRR, |root| below about 2E-5 (T-3484).** Excel's bisection stop leaves it 1E-4 to 2E-3 off the math on 9 of 15
    near-zero cells of xirr-two-flow-gate-a-sweep. The engine matches Excel to the bit: all 240 cells of that case pass at main c97e0b9c.
  - **T.INV and F.INV Gate A points outside the R3 region (T-3544).** These are accuracy/T.INV/s1 A91 and A96, and
    accuracy/F.INV/s1 A164, A167 and A169. Excel is more than G off the math there; the engine returns the math and
    passes Gate A through its math side.
- **Decision:** the owner was asked each question with a recommendation and ruled "keep Excel as the target" on all
  three (2026-11-06):
  1. NPER keeps decision 0188's region, 0 < rate < 1E-14. From 1E-14 up the engine matches Excel's own value.
  2. XIRR gets no R3 region. The engine keeps matching Excel's bisection result.
  3. T-3544's five Gate A points stay on Excel.
     - **Removed from decision 0267 item 3:** accuracy/T.INV/s1 A91 and A96. That item lists T.INV and was worded
       for every evidenced tiny-p point where Excel is more than G off. Its scope is now the probes-numeric cells
       T-3386 and T-3337 list (t1531, t1543, t3349, t3336 and the m9 cases).
     - **Settled, not removed:** accuracy/F.INV/s1 A164, A167 and A169. F.INV is not in 0267 item 3. These points were
       unruled under decisions 0188 and 0213: A164 and A169 are outside the region, and A167 is inside 0188's
       p < 1e-150 bound but unmarked. A167, F.INV(4.36982683607437E-189,1,1698856), therefore targets Excel's
       3.85E-145, not the underflowed 0.
     - Decision 0213's marked points are unchanged.
- **Consequences:**
  - T-4285 and T-3484 close with no engine change.
  - T-3544 closes. T-4489 reproduces Excel's values at its five points; until then they pass Gate A through the math
    side as today, and nothing regresses.
  - No new `MATH-OVER-EXCEL` entries.
