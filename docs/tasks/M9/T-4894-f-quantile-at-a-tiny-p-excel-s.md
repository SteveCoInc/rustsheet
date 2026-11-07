---
id: T-4894
title: "F quantile at a tiny p: Excel's leading-term closed form and deep-tail search (gen-m9 cells left by T-1871)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.INV, F.INV.RT and FINV at a tiny probability (either tail below about 1E-16) give Excel's value to its last digits,
where Excel's own deep-tail arithmetic is not the correctly rounded crossing the engine returns.

## Context
T-1871 fixed the deep-tail failures where Excel is the math (Gate A accuracy/F.INV/s1 A154, A168, F.INV.RT/s1 A155:
TOMS 708 brcomp's far branch in `incomplete_beta.rs` `large_prefactor`, and `fisher.rs` `estimate_solves`, which drops
a statrs estimate that is no root). What is left are cells where Excel is 1E-16 to 3E-14 relative from the 50-digit
math and the engine is the math, on the other side: neither is within half a unit of the 15th digit of the other
(T-1871 Outcome). The error grows with |ln p| over the first shape (about eps |ln p| / a), so it is the rounding of a
log-domain formula, not a search tolerance.
- Lead (T-1871, docs/excel-semantics/statistics.md "F quantile at a tiny p is near the leading-term closed form
  (T-1871)", UNVERIFIED): where the incomplete beta's leading term is exact to 1E-17 (x below about 1E-17 a/(a(b-1)/(a+1)))
  and p (the smaller tail) is below 1E-20, Excel's quantile is within a few ulp of the AS 109 leading-term form
  x = exp((ln q + ln a + ln B(a, b))/a), q the smaller tail and (a, b) its shapes (df1/2, df2/2 for the lower tail,
  swapped for the upper), mapped to F as (df2/df1) x/(1 - x) or (df2/df1) (1 - w)/w. 72 arithmetic variants (libm or
  `excel_ln`/`excel_exp`, `ln_gamma` or `excel_gammaln`, three sum orders, three mappings) were ranked on a fixed
  split of the literal-argument cells (md5 of the cell key, odd = hold-out): the best is bit-exact on 5 of 36 fit
  cells and within 4 ulp on 15 (hold-out: 5 and 17 of 32), where today's crossing is within 4 ulp on none. So the form
  is right and the arithmetic is not one of those 72: the misses are 140 to 250 ulp (one rounding of the sum at
  |ln p| 400 to 700), e.g. `F.INV(1E-170,5,5)`, `F.INV(1E-200,6,1)`, `FINV(1E-100,7,2)`. Try extended-precision sums
  (x87, `x87_exp::excel_exp_extended`) and Excel's LOG10-based ln. The scratch scripts' method: score every
  `=F.INV|F.INV.RT|FINV(literal,literal,literal)` cell of eval/ with an oracle number.
- Outside the lead: at p = 1E-300 with df2 3 to 10 Excel is 1E-12 to 1E-9 off the math and off the closed form
  (`F.INV(1E-300,6,3)`, `F.INV.RT(1E-300,4,10)`, `F.INV.RT(1E-300,3,5)`), and probes/stat-quantile-lift E6, E7
  (`F.INV(1E-50,6000,4)`, `F.INV(1E-50,2000,2)`) are 97% and 46% below the math: Excel's own search, as T-4103's
  points (decisions 0300, 0301: Excel stays the target; no R3 mark). E9 (`BETA.INV(1-2^-53,1.0024017013258346,
  12177.162026553196)`, Excel 0.0691959487211844, true about 0.003) is Excel's erratic extreme-shape BETA.INV
  (statistics.md "Inverse CDF functions at a tiny or near-1 probability (T-1543)"), UNVERIFIED.
- Between p = 1E-20 and 1E-15 statrs' estimate is kept unrefined where it solves the target
  (`F_QUANTILE_REFINE_P_MIN`, FITTED T-0974): with the crossing instead, `F.INV(1E-16,10,6)`, `F.INV(1E-20,6,10)`,
  `FINV(1-1E-16,10,3)` and `F.INV(1E-16,5,7)` come within 15 digits of Excel and `F.INV(1E-20,5,2)`,
  `F.INV(1E-20,4,8)` and E12 (`F.INV(1E-20,5,5)`) leave it, so the cut is noise-level either way until Excel's
  formula is known.
- Saturated cells (`F.INV(1E-200,1,1)` = 2.705579217114786E-151, `F.INV.RT(1E-300,2,1)`): T-4662 and T-4675 own the
  constant (statistics.md "F.INV.RT saturates at 3.696066238512854E+150/df1 in the far tail (T-2435)"); the cells
  below that saturate stay theirs too.
- Shared cells (as they were with T-1871): the composite cells gen-m9/2a18b2f5a4dc43ac A259 and 30e20ba3302fea8b A310
  with T-2221 and T-1374 (the outer function), probes/stat-quantile-lift E6 with T-2500, gen-m9/0a614cfe974b59c3 A268
  and 85f48d5cdfb24555 A10, A12 with T-2695, and gen-m9/4ab2ba98d26c8a5e A110 with T-3485: whichever task lands
  first clears them.
- Owned gen-m9 cells, moved from T-1871 (excel-graph; every one still a known failure in eval/baseline/gen-m9.txt
  after T-1871, which changed none of their bits; `F.INV(1E-20,4,6)` A44 and `F.INV(1E-16,1,2)` A114 moved from
  2.5E-12 and 1E16 relative off to 9 and 53 ulp):

Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A145
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A96, A149
- gen-m9/0a614cfe974b59c3.case.json (4): Calc!A4, A24, A268, A329
- gen-m9/0a9226016e8f5f33.case.json (4): Calc!A45, A235, A314, A321
- gen-m9/146b46e74f2dac3b.case.json (3): Calc!A117, A367, A375
- gen-m9/165db58d1540f014.case.json (3): Calc!A104, A272, A364
- gen-m9/27601819397dd56e.case.json (2): Calc!A56, A284
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A199, A376
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A212, A287
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A64, A394
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A44
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A283
- gen-m9/4ab2ba98d26c8a5e.case.json (4): Calc!A110, A240, A248, A332
- gen-m9/4e55500bb8ad3cd8.case.json (4): Calc!A77, A169, A353, A360
- gen-m9/530c636749a85209.case.json (1): Calc!A83
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A47, A277
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A155, A247, A385
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A319
- gen-m9/623663cacea53bbc.case.json (1): Calc!A148
- gen-m9/7674c89c824aca43.case.json (1): Calc!A329
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A28
- gen-m9/7c20351562a2d71d.case.json (6): Calc!A70, A83, A129, A208, A264, A267
- gen-m9/85f48d5cdfb24555.case.json (3): Calc!A10, A12, A256
- gen-m9/899f9555e67da2d6.case.json (3): Calc!A46, A187, A233
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A127, A245
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A296
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A20, A81
- gen-m9/a54c738c697617c3.case.json (3): Calc!A94, A338, A386
- gen-m9/a612e89321a6df9c.case.json (6): Calc!A206, A252, A256, A259, A298, A348
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A358
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A40, A116, A238
- gen-m9/a7bf0d8d81d33ba6.case.json (2): Calc!A181, A333
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A99
- gen-m9/b4173fe25109a238.case.json (3): Calc!A67, A140, A311
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A34, A334
- gen-m9/bbdfc06cae0e0d71.case.json (3): Calc!A247, A262, A308
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A311
- gen-m9/c18dbf21f11093f3.case.json (2): Calc!A230, A374
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A389
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A58, A362
- gen-m9/c6cfda6bd10cae93.case.json (5): Calc!A53, A61, A114, A175, A297
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A237
- gen-m9/cde551ea787f713a.case.json (2): Calc!A139, A367
- gen-m9/d8266bc15533702a.case.json (1): Calc!A57
- gen-m9/e6cf3b20a014e6a3.case.json (6): Calc!A13, A74, A143, A257, A290, A379
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A203, A335
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A100
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A44, A213, A320
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A295
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A209
- gen-m9/fec39efaba9284c3.case.json (3): Calc!A14, A76, A206
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A259
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A310
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/7c20351562a2d71d.case.json: Calc!A37, A254
  - gen-m9/c18dbf21f11093f3.case.json: Calc!A190
  - gen-m9/589f42e67f4b4b8d.case.json: Calc!A319
  - gen-m9/8dd2312c96effb1b.case.json: Calc!A169
  - gen-m9/c2974c061384fdd9.case.json: Calc!A267


## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): each cell above has Excel's value within half a unit of the 15th significant
      digit, or a follow-up records why not; known-failure lines that become bit-exact leave eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The deep-tail form, if one is adopted, carries decision 0188 R4's hold-out and fresh-sweep results (a structural
      model chosen from ranked candidates) and its rule is recorded in docs/excel-semantics/statistics.md.
- [ ] Gate A (`rustsheet-eval accuracy`) stays at 0 failing; the per-call caps and `QUANTILE_WEIGHT` stay.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The saturation constant (T-4662, T-4675) and T-4568's points (F.INV.RT/s1 A112, A142).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255; moved from T-1871): the gen-m9 cells listed under Context are visibly
off, so the task is in the campaign. Its target is that every visibly-off cell it owns has Excel's type, error and
text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under
Acceptance criteria states it. Bit-level criteria still unmet then move to a follow-up task without the key.
