# 0282 — R3 marks for the T-3367 NPER tiny-rate eval cells

- **Status:** accepted (2026-11-05, T-3367). Applies decision 0188 R3, case 2, to eval cells inside the region that
  decision 0220 already applies to Gate A points.
- **Context:**
  - Decision 0188 R3 lists NPER at 0 < rate < 1e-14 as an approved `MATH-OVER-EXCEL` case. Rates where 1 + rate is 1
    are excluded and stay `#DIV/0!`. The case is recorded in financial.md, "NPER at a tiny positive rate".
  - T-3367 moves the engine to the math in that region. Its probe case, probes-numeric/t3367-nper-tiny-rate
    (excel-graph, 2026-11-05, with a 50-digit math sidecar), holds 66 numeric cells inside the region (A22 to A97, at
    rates 1.2E-16 to 9.9E-15). In each, Excel is more than G = 1e-4 off the math, from 6.5E-3 to 4.7E-1, and the
    engine returns the math to within 1e-14.
  - Without a mark, the R1(c) and new-case owner gates would count these 66 cells as visibly off with no owner.
    `eval/accuracy/math-over-excel.tsv` is a protected path (T-3379), so marking them needs this record.
- **Decision:**
  - The 66 cells are R3-marked against the financial.md entry. The region is unchanged: positive rates below 1e-14
    where 1 + rate is not 1.
  - The probe's cells at 1E-14 to 1E-13, where Excel is still 7.0E-4 to 6.9E-3 off, are outside the region. They are
    not marked and the engine keeps Excel's value there. Whether to widen the region is T-4285's owner ruling.
- **Consequences:**
  - The marked cells pass R1(c) and Gate B on the math side (`R1c::R3Pass`). The check rejects a mark where Excel is
    not more than G off the math or the engine is not within ε of it.
  - Every error cell of the probe keeps Excel's error, so no mark covers an error.
