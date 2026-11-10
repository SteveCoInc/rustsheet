---
id: T-5072
title: "F-family math values for the 122 counted t4857-f-classes cells and 16 t3443 cells that rustsheet-accuracy skips or drops (huge-df tails and quantiles below and above SERIES_SHAPE_CAP)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4776]
---

## Goal
`rustsheet-accuracy math` gives a 50-digit value to the F.DIST, F.DIST.RT, FDIST, F.INV, F.INV.RT and FINV cells of
probes-numeric/t4857-f-classes that it now skips as `not converged` or never writes. Then the 122 of them that count
as visibly off, and the 16 t3443 composite cells, are sorted by the math, not by the lack of it (decision 0325).
Under decision 0188 R1(c), each becomes Phase 1b evidence, a `campaign:` cell for its fit owner, or an R3 candidate.

## Context
- Gap triage 2026-11-08 (`make eval` on origin/main 6f0c3d8e5, excel-graph). 122 counted visibly-off cells of
  t4857-f-classes have no math value. Their owners are T-4857's fit follow-ups: T-4881, T-4883, T-4884 (research),
  T-4885, T-4886 and T-4887. None of those attaches math, and decision 0325 says the fit work must not start on a
  cell before its math is attached. T-4776, which attaches math for the same family at a shape from 1E9, names
  t1538, t1530 and t3862 but not t4857.
- Two root causes. They show in the sidecar's `skipped` list (182 entries: 180 `not converged`, 2 `no single value`)
  and in the cells missing from both `sheets` and `skipped`:
  - **Above SERIES_SHAPE_CAP (1E7)**: 103 skipped cells have max(df1, df2)/2 above 1E7. `_betainc`
    (`tools/src/rustsheet_tools/accuracy/defs/beta.py`) hands those shapes to `mpmath.betainc` alone, and it does not
    converge. This is T-4776's root cause. Once T-4776's method lands, regenerating this sidecar should value them.
    That is why this task depends on it. Examples: A1006 `F.INV(0.5,26,795752330)`, A2054
    `F.DIST.RT(1.05,1000,20000001)`, A2114 `F.DIST.RT(1.05,1000,10000000000)` (Excel 0.13247407018674934, ours
    0.13247407110080875).
  - **At or below the cap**: the other 79 skipped cells, most at a shape from 1E5 to 1E7 (the 2 `no single value` ones at df 1 to 2). With a + b above MPMATH_SHAPE_CAP they go to `_series` /
    `_tail_series`, and the likely failure is its 1E5-term budget (SERIES_MAXTERMS) or the per-cell work budget `spend()` (T-3680), not
    mpmath; confirm that first. T-4776 does not reach these. Examples: A2028 `F.DIST.RT(1.05,1000,1999999)` (Excel 0.13253736480388217,
    ours 0.13253736480364278), the A2658:A2687 sweep `F.DIST.RT(1.3,1000,1E6..3.98E6)` (30 cells, about 1E-11
    relative between Excel and ours), A1214 `F.INV.RT(0.5,7829,268524)`, A1173 `FINV(0.913174975294355,200,12179676)`.
    The 16 composite cells `F.INV(F.DIST(x,d1,d2,TRUE),d1,d2)` of t3443-basym-sweep, -a and -hold (d2 1E5 to 2E5)
    are probably the same failure. T-4776's Context leaves them to T-3597 if T-4776 does not value them. This task
    attaches their math, and T-3597 keeps their fit.
  - **Never written** (in neither `sheets` nor `skipped`): A904, A913, A916, A919, A922, A3112, A3144 and A3162,
    small-df quantiles at a tiny p, for example A904 `F.INV(5.36895450533054E-18,1.8967,1)` (Excel
    7.112449599400453e-35, ours 7.112449599400503e-35). Find out why `eval_math` drops them (a candidate rule, a
    double that cannot be recorded, or a disagreement between dps 60 and 80), and either value them or record the
    skip reason in the sidecar.
- Some cells read their p from a constant cell (`F.INV.RT(B1049,31,4650003532)`, where B1049 = DBL_MIN). `parse_arg`
  already reads those (`cell_refs=True`), so they are not a separate cause.
- What the math is likely to show: T-4885's Context (cells where the math exists) has the engine within 1E-15 of the
  math on most huge-df quantiles, and Excel 1E-13 to 1E-5 off it. Some of these cells will be R3 candidates
  (Excel more than 1e-4 off). Report them. Do not mark them: an R3 mark needs an owner ruling (decision 0188 R3).
- Method is the design question, hence opus: a term budget that scales with the shape but stays deterministic,
  Temme's uniform expansion, or the gamma limit with its 1/b correction. It has to agree between dps 60 and 80, and it
  has to leave every Gate A table value unchanged. Reuse T-4776's method where it reaches. Do not add a second large-
  shape path beside it.
- Scope against the other open math tasks: this task changes `defs/beta.py` (and `fisher.py` only if a search needs
  it) after T-4776. T-4775 owns `defs/discrete.py` (BINOM.DIST, NEGBINOM.DIST). probes-numeric is frozen for new
  cells (decision 0267 item 5), but a regenerated sidecar adds no case and no cell.
- Not in scope, filed nowhere yet: the 6 probes/t4857-f-kinds cells (`F.INV(0.3,5,TRUE)`, `(1=1)`, `Data!A5`, a lifted
  `{1.5;"2.5";TRUE}`). They have no math value because `eval_math` does not coerce a boolean, comparison or lifted
  df. That is an argument-kind rule in `eval_math`, not a convergence problem.

Cells this task owns for their math value (the fit work stays with T-4881, T-4883, T-4884, T-4885, T-4886, T-4887 and,
for t3443, T-3597):
- probes-numeric/t4857-f-classes.case.json (122): Calc!A335, A375, A903, A904, A907, A913, A916, A919, A921, A922, A923, A1006, A1049, A1069, A1088, A1134, A1155, A1173, A1174, A1195, A1214, A1215, A1222, A1236, A1238, A1252, A1255, A1256, A1257, A1262, A1265, A1269, A1274, A1275, A1280, A1289, A1290, A1293, A1294, A1303, A1304, A1310, A1314, A1317, A1318, A1412, A1413, A1431, A1449, A1966, A1982, A1985, A1991, A1994, A1998, A2007, A2028, A2029, A2030, A2031, A2052, A2053, A2054, A2055, A2062, A2066, A2074, A2078, A2082, A2086, A2094, A2097, A2098, A2100, A2101, A2103, A2104, A2109, A2110, A2111, A2113, A2114, A2116, A2117, A2118, A2119, A2123, A2124, A2125, A2658, A2659, A2660, A2661, A2662, A2663, A2664, A2665, A2666, A2667, A2668, A2669, A2670, A2671, A2672, A2673, A2674, A2675, A2676, A2677, A2678, A2679, A2680, A2681, A2682, A2683, A2684, A2685, A2686, A2687, A3112, A3144, A3162
- probes-numeric/t3443-basym-a.case.json (6): Calc!A184, A195, A198, A213, A215, A218
- probes-numeric/t3443-basym-hold.case.json (3): Calc!A184, A185, A206
- probes-numeric/t3443-basym-sweep.case.json (7): Calc!A184, A191, A193, A195, A201, A207, A211

## Acceptance criteria
- [ ] `_betainc` (or the method T-4776 introduces) returns I_x(a, b) at every shape these cells reach, with dps 60
      and 80 in agreement and a deterministic budget (no clock, T-3680). A unit test pins at least three cells
      against an independent 50-digit reference (mpmath quadrature of the beta density, or the gamma limit with
      enough correction terms): one above the cap (t4857 A2114), one below it (A2028), and one t3443 composite.
- [ ] `cd tools && uv run rustsheet-accuracy math --suite probes-numeric --function F.DIST.RT` (or the narrowest
      `--function` set that reaches the four cases) regenerates their sidecars. None of the 138 cells above is under
      `skipped`, and each one still without a value has a named reason in the Outcome. `rustsheet-accuracy math
      --check` passes, and `rustsheet-accuracy verify` shows no change for BETA.DIST, BETA.INV, F.DIST, F.DIST.RT,
      F.INV, F.INV.RT and FINV.
- [ ] After `make eval`, the Outcome gives per case the count that became Phase 1b evidence (`r1c_left_out`) and the
      count still counted, with Excel-minus-math and ours-minus-math for each counted cell. It names the cells where
      Excel is more than 1e-4 off the math (R3 candidates, for an owner ruling via their fit owner). `eval/baseline/`
      gains no new line: this task changes no engine value.
- [ ] `make check` green; reviewers approve (excel-fidelity included).

## Out of scope
- Any engine change to the F distribution or quantiles (T-4881, T-4883, T-4885, T-4886, T-4887; T-3597 for t3443).
- R3 marks or decision records for the cells where Excel is more than 1e-4 off.
- The t1538, t1530 and t3862 cells (T-4776), and the t4857-f-kinds argument-kind cells (see Context).
