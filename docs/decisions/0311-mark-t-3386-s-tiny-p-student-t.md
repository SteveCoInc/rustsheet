# 0311 — Mark T-3386's tiny-p Student-t and beta quantile cells under decision 0267 item 3

- **Status:** accepted (2026-11-06, T-3386)
- **Context:** decision 0267 item 3 (owner ruling, 2026-11-04), as narrowed by decision 0301 (owner ruling,
  2026-11-06), rules the math the target (decision 0188 R3) on the probes-numeric and m9 cells T-3386 lists where
  Excel is more than G = 1e-4 relative off the 50-digit math: T.INV.2T, TINV, T.INV, CONFIDENCE.T and BETA.INV at a
  tiny p. Marking a cell in `eval/accuracy/math-over-excel.tsv` is a protected-path change (SPEC-0029, T-3379). T-3386
  checked each listed cell against its committed math sidecar value (`<case>.math.json`, written by
  `rustsheet-accuracy math`):
  - 51 probes-numeric cells have Excel 0.67 to 2.05 relative off the math and the engine within ε = 1e-9 of it:
    t3349-t-inv-2t-sweep (42 T.INV.2T cells), t1531-t-tiny-p A73, A74, A82, A83, A97, A98, A106, A107 (T.INV, T.INV.2T,
    TINV) and t1543-inverse-cdf-tiny-p A86 (BETA.INV).
  - gen-m9/7980ffb6095a340d A160, `TINV(1E-200,1E+10)`, had no math reference: gen-m9 has no sidecars. T-3386 commits
    that case's sidecar as `rustsheet-accuracy math --suite gen-m9 --function TINV` writes it (SPEC-0029, "Math
    references for eval cells", allows any suite), and no other gen-m9 sidecar. Math 30.2285087720, Excel 58.193, engine
    3.7E-10 relative off.
- **Decision:** these 52 cells go into `math-over-excel.tsv` one by one under the existing statistics.md entry "Tiny-p
  quantiles: Excel is grossly wrong, RustSheet returns the math" (`Status: MATH-OVER-EXCEL`), whose Rule and Region now
  name them. This applies the owner's rulings; it adds no R3 case and widens no region beyond 0267 item 3 as 0301 left
  it, so no new owner ruling is needed. No other cell is marked:
  - t1531 A84 and A108 (CONFIDENCE.T) have no committed math value, since CONFIDENCE.T has no Gate A definition; a
    mark would not count under R1(c). They stay counted.
  - t1543 A89, A91 and A92 are `#NUM!` in the engine against Excel numbers; an error never takes a mark (0188 R3).
  - t1543 A43 and A79 have Excel within G of the math; Excel stays the target (directive 1).
  - The Gate A points T.INV.2T/s1 A98, A99, A103 and TINV/s1 A102 are outside 0267 item 3 after decision 0301.
- **Consequences:** R1(c) classes the 52 cells `r3_pass` (probes-numeric's counted cells drop from 1636 to 1585; the
  gen-m9 sidecar also moves gen-m9/7980ffb6095a340d A84 to Phase 1b). Eval pass/fail does not change: the cells stay F
  lines of eval/baseline, and Gate B and the R1(c) rows read them as passing. A later regeneration of gen-m9 sidecars
  must keep A160's math value.
