---
id: T-4352
title: "Identify Loader's stored chain in Excel's BINOM.DIST and NEGBINOM.DIST pmfs (decision 0293's 15 regressed cells)"
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
BINOM.DIST's interior pmf and NEGBINOM.DIST's pmf (f >= 1) give Excel's doubles. That means identifying the
stored chain of Excel's Loader kernel (lf/2, the stirlerr sum, the order of the bd0 terms and the final p·kernel
step), which T-2513 left open.

## Context
- T-2146 (decision 0293) moved both pmfs from the exact walk onto `loader.rs` `excel_dbinom_exponent` everywhere,
  because that model beats the walk on train, hold-out and a fresh sweep. It is still bit-exact on only about a third of
  cells. statistics.md "BINOM.DIST and NEGBINOM.DIST pmfs take Loader's kernel everywhere" and "BINOM.DIST pmf: closed
  forms at k = 0 and k = n, and one kernel with BETA.DIST" record what was tried.
- Decision 0293's 15 gated cells that matched under the walk and miss under the model:
  - gen-m9: 17c110a53924beb8 Calc!A197, 1db18890916ee643 Calc!A10 (visibly off), 329b177b5d5165ad Calc!A391,
    3a25a03eb94fcfbd Calc!A73, 941c409532603d32 Calc!A223, b9a32ce9a1fd0d71 Calc!A309, d37313406b9f6e55 Calc!A141,
    ede0323c389fc703 Calc!A249 and Calc!A97;
  - ironcalc-calc: STATISTICAL/BINOM.xlsx BINOM.DIST!E51, E53, E54;
  - probes: t3352-binom-negbinom-large-n Calc!A15, A157, A169.
  - Three of the gen-m9 hashes above are shared with probes copies (probes/t2429, probes/t2436), so
    `scripts/exit_owners.py` credits them only with the suite prefix (gap triage 2026-11-05, T-4365):
    gen-m9/1db18890916ee643 Calc!A10, gen-m9/3a25a03eb94fcfbd Calc!A73, gen-m9/941c409532603d32 Calc!A223.
- Visibly-off cells of T-2146's fresh sweep, owned here: probes-numeric/t2146-binom-negbinom-pmf-fresh Calc!A5:A6,
  A10, A13, A15, A19:A22, A24, A27, A29:A30, A33:A35, A38, A45, A47, A50:A51, A55:A56, A60, A62, A65, A69:A70, A76,
  A86:A87, A93, A104, A110, A117:A118, A120:A121, A124:A128, A130:A132, A135:A136, A139, A144, A148:A149,
  A151:A152, A155, A157:A158, A160:A163, A165, A173:A175, A177:A178, A180, A186, A195, A197, A199:A200,
  A202:A203, A206, A209:A212, A215, A218, A220:A221, A224, A226:A229, A233, A236:A237, A239:A240, A245, A251,
  A254:A256, A258, A264, A267:A269, A272, A274, A279, A288, A294, A297:A299, A301, A304, A306, A311,
  A314:A315, A317:A318, A320, A361:A363, A373, A376, A378, A382, A385, A393, A398 (131 cells).
- Visibly-off gen-m9 pmf cells no task named (`scripts/exit_owners.py --unattributed` on c5fdf6bd3, Excel then engine),
  owned here: 2138b909d8f810ea Calc!A45 `NEGBINOMDIST(10,DAY(42697),0.999999)` 6.451075625335091E-53 vs ...129E-53;
  27601819397dd56e Calc!A280 `BINOMDIST(3,6,1E-16,...)` 2.0000000000000043E-47 vs 1.9999999999999905E-47;
  gen-m9/44a8841352dc48e9 Calc!A157 `STANDARDIZE(...,NEGBINOMDIST(7,9,0.025))` -2042715136323.223 vs ...228;
  583ca2baea0bb30a Calc!A123 `NEGBINOM.DIST(7,2.5,1-1E-10,...)` 8.000004631861894E-70 vs ...933E-70;
  607e85cfee83a689 Calc!A258 `NEGBINOM.DIST(SUM(...),9,0.5,...)` 4.0054898597929714E-07 vs ...978E-07;
  6e18550e9fc9c1b9 Calc!A5 `NEGBINOM.DIST(6,Data!B8,2.5%,...)` 5.4696773655614074E-61 vs ...428E-61;
  9bc247733a1ca9fa Calc!A340 `NEGBINOMDIST(CEILING.PRECISE(...),8,...)` 1.3262698904326607E-23 vs ...504E-23;
  b9a32ce9a1fd0d71 Calc!A43 `NEGBINOMDIST(...,100,...)` 1.3139181131891553E-265 vs ...209E-265;
  c2684066d8df263e Calc!A149 `NEGBINOMDIST(CODE(Data!C1),8,1E-10)` 1.3378455939797276E-72 vs ...6992E-72.
  They are attributed to no function only because each formula calls a second non-glue function.
- T-2146's remaining gated pmf cells (gen-m9 NEGBINOMDIST tails at p 1E-20 to 1E-10 and 1 - 1E-10 to 1 - 1E-16)
  and the T-2513 probes are the train set. Keep t2513-r3-binom-pmf-hold and t2513-verify-r3-binom-pmf-h as
  the hold-out, and draw a fresh sweep after the freeze (decision 0188 R4).

## Acceptance criteria
- [ ] The 15 cells above pass again, and gen-m9's remaining BINOM-family pmf lines shrink, with no new known-failure
      lines (or a decision record for any).
- [ ] The fresh-sweep cells listed above are within 15 significant digits of Excel's value, or the Outcome says which
      remain and why.
- [ ] statistics.md records the chain and the candidates that lost.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdfs (bratio port, T-2847 and its follow-ups) and BINOM.DIST.RANGE's multi-point ranges.
