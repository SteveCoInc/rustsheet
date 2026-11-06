# 0289 — Mark the huge-shape gamma quantile batch's closed-form points under decision 0188 R3 case 3

- **Status:** accepted (2026-11-05, T-3863)
- **Context:** T-3863 adds a Gate A batch `e-T-3863` for GAMMA.INV, CHISQ.INV and CHISQ.INV.RT at huge shapes, its math
  from Temme's uniform asymptotic expansion of the incomplete gamma (tools/src/rustsheet_tools/accuracy/defs/gamma.py), filled from excel-graph
  on 2026-11-05. On 131 of its points (103 GAMMA.INV, 28 CHISQ.INV.RT; shapes 3.9E9 to 7E293) Excel returns the
  Wilson-Hilferty closed form, 1.000266 times the 50-digit quantile (relative 2.66e-4 on every one of them, against
  G = 1e-4): exactly decision 0188 R3's case 3, "GAMMA.INV at a huge shape, the Wilson-Hilferty closed-form branch
  only", which already lists GAMMA.INV, CHISQ.INV.RT and the branch's other functions. No other point of the batch is
  more than 1e-12 off the math, except four GAMMA.INV points whose quantile is below the smallest double, where Excel
  is 0 and the envelope's absolute floor applies (not R3). Marking a point in `eval/accuracy/math-over-excel.tsv` is a
  protected-path change (SPEC-0029, T-3379).
- **Decision:** the 131 points go into `math-over-excel.tsv` under the existing entry statistics.md "GAMMA.INV at a
  huge shape, closed-form branch" (`Status: MATH-OVER-EXCEL`). This adds no R3 case and widens none, so no new owner
  ruling is needed (decision 0188 R3 asks one only for a new case). The engine still returns Excel's closed form there,
  so each marked point is a known failure owned by T-3364, which moves the branch to the math.
- **Consequences:** Gate A checks these points against the math only. The 131 known-failure lines are removed when
  T-3364 lands (a listed point that passes is a stale line and fails the check). The batch's CHISQ.INV.RT `#NUM!`
  cells (26) and every true-quantile cell stay Excel's targets, as decision 0188 R3 says.
