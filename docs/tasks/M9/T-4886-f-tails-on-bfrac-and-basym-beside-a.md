---
id: T-4886
title: "F tails on bfrac and basym beside a huge shape outside T-4434's df1 3..400 band (df1 1000 or df2 200..201 with the other df 1E6..1E10): engine 1E-12 to 9E-8 off Excel"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
F.DIST's cdf, F.DIST.RT and FDIST match Excel on bfrac and basym where one shape is huge (df 1E6 to 1E10) and the
other is 100 or more, the region next to T-4434's sweep that it does not cover.

## Context
- T-4434 sweeps df2 1E4..1E10 with df1 3..400 for T-4435 (Cephes' fraction at a huge shape). T-4857's probe
  (probes-numeric/t4857-f-classes, excel-graph 2026-11-07) found the same engine error outside that band:
  - df1 1000 against df2 1E6 to 1E10 (bfrac): `F.DIST.RT(1.3,1000,d2)` for d2 1E6 to 4E6 is 1E-12 to 1.9E-11 off
    Excel (sweep-df2-2e6 A2658:A2687, smooth in d2: no jump at shape 1E6), `F.DIST.RT(1.05,1000,d2)` 1.8E-12 at
    2E6, 1.8E-11 at 2E7, 6.9E-9 at 1E10 (A2028:A2031, A2052:A2055, A2111:A2123). At df1 1 and 2 the same sweep
    matches Excel to 15 digits.
  - The mirrored orientation, a huge df1 against df2 200 or 201: `F.DIST(0.17003311650339,1E10,200,TRUE)` is 9.0E-8
    off Excel and 8.9E-8 off the math, Excel 6.6E-10 off it (A434); `FDIST(1.55868846894177,201,1E10)` 5.1E-8 off
    Excel, Excel 2.9E-9 off the math (A315).
  - Basym at (201, 7.2E8): `F.DIST(1.01014876035667,201,724863290,TRUE)` is the math (2.8E-16); Excel is 1.3E-9 off
    it (A313), the T-4214 pattern of an inaccurate Excel at a huge shape.
  - Bfrac at (10814, 4E9) in the deep lower tail: 1.8E-13 off (A335).
- Decision 0267 item 2's 13-digit bar applies (these are family cells first wrong at the 8th to 13th digit).
- Code: `bratio/bfrac.rs`, `bratio/basym.rs`, `incomplete_beta.rs` `beta_tails` (the fallback), `large_shape_beta.rs`.

Cells (probes-numeric/t4857-f-classes): A313, A315, A335, A434, A2028:A2031, A2052:A2055, A2111, A2114, A2117, A2123, A2658:A2687

## Acceptance criteria
- [ ] Each cell within 13 digits of Excel, or the residue measured against the math with Excel's side documented in
      statistics.md; T-4434's and T-4435's regions not regressed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4434's df1 3..400 band (T-4435 owns it).
