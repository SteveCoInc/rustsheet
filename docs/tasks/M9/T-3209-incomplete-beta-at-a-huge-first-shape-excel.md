---
id: T-3209
title: "Incomplete beta at a huge first shape: Excel's bgrat is up to 1.5E-14 off the true value (T-2528 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2510]
campaign: visible
defer: phase-1b
---

## Goal
The 22 cells of probes-numeric/t2528-f-dist-huge-df2 that are more than half a unit of the 15th digit off Excel
are within it: Excel's bratio L140 bup + bgrat arithmetic is modelled and matched (decision 0114 `visible`:
reproducing Excel's own algorithm where Excel is the inaccurate one).

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
- T-2528 replaced Cephes' non-converging fraction at a first shape of 1E6 or more and b/a at most 1E-6 with an
  accurate DiDonato-Morris expansion (`large_shape_beta.rs`), within 2E-15 of a 120-digit mpmath reference.
- Excel is within about 1E-15 of the true value for tails above 1E-4 but up to 1.5E-14 off further out
  (`F.DIST.RT(50,5,1E10)` 5.4969308523163854E-52, true 5.496930852316323E-52; `F.DIST.RT(2,5,1E9)`
  0.07523524713822176, true 0.07523524713822164). docs/excel-semantics/statistics.md "Incomplete beta at a huge first
  shape".
- The error grows with z = -nu ln x, which suggests z formed in Excel's own LN/log1p in bgrat (bratio L140 bup + bgrat
  at b0 = 1/2): T-2510's models (`bratio_excel.py` `bgrat`) are the place to test it.

- References: `docs/research/t2528/reference.py` (120-digit mpmath at the double arguments).
- `large_shape_beta.rs` is an accuracy stand-in: when T-2510's bratio bgrat lands and the old kernel goes, it is
  folded into or replaced by `bratio`'s bgrat, so the two series do not stay as parallel definitions.

- Gap triage 2026-11-04: the band's Gate B face, gen-m9 7bd219ec0c0c5e22 A349 `FDIST(7,10,1E+8)` (b = 5), is T-4018's,
  on T-4017's sweep over df1 2 to 79. A model T-4018 lands should be checked on this task's 22 cells.
- probes-numeric/t4080-beta-num-band-edge b = 40 cells A21:B21, A42:B42, B63, A84:B84, A105:B105, A126:B126, B147, A168:B168, A189:B189, A210:B210, A231:B231, A252:B252, A273:B273, A294:B294 (T-4080): ours 1.3E-9 to 4.6E-8 relative off Excel at a = 2E9 to 1E10 (26 cells; A63 and A147 pass within tolerance).
  Gap triage 2026-11-04 (origin/main b450050f8) moved these cells to T-4214 (decision 0267 Tier A, live). Against
  60-digit mpmath it is Excel that is 5E-10 to 4.6E-8 off at b = 40, and ours is within 1E-14. b0 = 40 leaves L140 for
  bfrac, so this task's bgrat is not their path.

## Acceptance criteria
- [ ] The 22 cells are within half a unit of the 15th digit of Excel (a model of Excel's bgrat at these points).
- [ ] No cell of t2528-f-dist-huge-df2, t1530-f-df-cut or t1538-* moves away from Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells below the mean (Cephes' incbcf side), which are T-2510's.

## Evidence (T-4017)
Sweep cases, whose visibly-off cells (the F tails above the mean at df1 other than 5, in the same band) this task owns:
- probes-numeric/t4017-f-huge-df2-above-mean-1e7.case.json (T-3754 moved 46 of its visibly-off cells into 15 digits by
  taking L140 bup+bgrat up to a0 = 1E7; docs/research/t3754)
- probes-numeric/t4017-f-huge-df2-above-mean-3e7.case.json
- probes-numeric/t4017-f-huge-df2-above-mean-1e8.case.json
- probes-numeric/t4017-f-huge-df2-above-mean-3e8.case.json
- probes-numeric/t4017-f-huge-df2-above-mean-1e9.case.json
- probes-numeric/t4017-f-huge-df2-above-mean-1e10.case.json
- probes-numeric/t4045-f-dist-rt-zero-cut.case.json: Calc!E3 (`FDIST(1.5,80,1E10)`, 4E-8 relative off; T-4045)
- probes-numeric/t4045-f-dist-rt-zero-cut.case.json: Calc!A1:A17, B1:B18, C1:C14, E2, E5 (F.DIST.RT and FDIST tails 1E-271 to 1E-291 at df2 1E7 or 1E8, Excel and ours each 1E-14 to 2E-13 off the math, 13th-14th digit apart; residue13, decision 0267; found unowned by the 2026-11-06 gap triage)
- probes/t4045-f-dist-rt-zero-cut-fresh.case.json: Calc!A1:A8, B1:B9, C1:C9, D1:D8, E1:E8, F1:F9 (F.DIST.RT tails near 1E-285, last digits; T-4045)
- probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json (146): Calc!A1:A2, A4, A6, A9:A10, A13, A17:A19, A22:A24, A26:A28, A32, A36:A40, A43, A47, A49, A51, A53:A54, A57, A60, A63, A66:A67, A69, A72, A74, A78:A79, A81:A82, A85:A86, A90, A92, A97:A98, A101:A102, A105:A107, A109, A111, A118, A120:A122, A125:A126, A129, A133, A137, A139:A140, A149:A150, A153, A159, A163, A173:A175, A178:A179, A184, A186, A188, A190, A195:A196, A198, A201, A203, A205:A207, A209:A210, A212:A215, A218:A219, A221:A222, A224:A228, A232:A233, A239, A246, A249, A251:A254, A259, A261, A263:A264, A266, A268, A270:A272, A276, A278, A280, A282, A285:A288, A290:A291, A294:A295, A297:A299, D13, D19, D27, D48, D68, D76, D80, D84, D93, D134, D140, D145 (T-4434: F.DIST.RT at df1 3 to 79 and df2 1E4 to 2E6, and BETA.DIST above the mean at a below 40, both on L140 bup+bgrat. Excel is within 9.5 (F) and 47 (BETA.DIST) units of |ln tail|·2^-53 of the math there, up to 1.3E-13 relative, and the correctly rounded math is not within 15 digits of Excel. So once T-4435 puts ours on the math, what is left is Excel's own digits.)
See (docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape").
