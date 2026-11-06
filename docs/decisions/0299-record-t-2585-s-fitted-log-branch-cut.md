# 0299 — Record T-2585's fitted log-branch cut of the t density

- **Status:** accepted (2026-11-06, T-2585)
- **Context:** T-2585 ports Excel's Student t density (Loader's saddle point in Excel's x87 pieces). Its log-branch
  cut `xx > n` (`T_LOG_BRANCH_CUT` in `loader.rs`; R's is 0.2) was chosen by scoring cuts on the in-sample probes
  t2513-r3-tdensity-a1 and -a2 (2,406 cells), so it is a fit of kind (a) under decision 0188 R4. Tagging it raises
  `quality/fitted.json` for `loader.rs` from 1 to 2, a protected path. The Excel-fidelity review also found the cut, and
  the bd0 cut `EXCEL_BD0_CUT` (fitted on BETA.DIST, T-2464) that the density reuses, applied outside their measured
  ranges (R4.3), no fresh sweep (R4.2), and no margin stated before the hold-outs were scored (R4.1).
  Evidence, all in T-2585's Outcome:
  - Hold-outs: hold2 573 exact of 960 (cut 0.2: 480; no log branch: 520), the verifier's 505 of 1,000 (391; 417). The
    shipped model's train rate on a1 + a2 is 879 of 2,406, below both hold-out rates; it beats the correctly rounded
    density on every set and count.
  - Range: over the 19,793 T.DIST(.,FALSE) rows of docs/research/t2464/cells.tsv, outside the fitted df 3..100,
    |t| 0.31..4.3, the cut scores at least as well as 0.2, no log branch and the accurate `student_t_density` on
    every count (exact, within 4 ulp, within 15 digits); the accurate density is the worst on exact cells everywhere.
    Variants of the bd0 cut (0.1 to 0.3) gain at most 18 exact cells of 12,037 in range and lose 4-ulp cells.
  - Fresh sweep: Gate A batch T.DIST/r4-T-2585 (300 points, filled on excel-graph 2026-11-05, after the
    2026-10-24 freeze): every density is inside the envelope except 11 points of T-3554's known error mismatch (t·t
    overflows; Excel #NUM!), listed in known-failures.tsv.
- **Decision:** Keep `T_LOG_BRANCH_CUT = 1` with `FITTED(T-2513, 2406 cells, fitted on ...; measured on df
  3..2.7E297, |t| up to 1.1E153)`, and widen `EXCEL_BD0_CUT`'s tag range to the T.DIST density, rather than guard the
  Loader path to the fitted range. R4.3's fallback to accurate math exists so that a fit cannot be worse than the
  math where it was not measured. Here it was measured: over the wider range the fitted model beats the accurate
  density on every count. The missing pre-declared margin is accepted because the hold-out rates exceed the train
  rate, the case a margin guards against being the opposite.
- **Consequences:** `quality/fitted.json` counts 2 tags for `loader.rs`. A `FITTED` range may name a measured range
  wider than the fitted one when the task records the fit's counts against the accurate math there and a fresh Gate A
  batch covers it. The T.DIST/r4-T-2585 batch stays in Gate A. T-3554 owns the 11 new known-failure lines.
