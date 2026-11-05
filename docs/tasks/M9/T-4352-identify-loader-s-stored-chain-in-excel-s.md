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
- Visibly-off cells of T-2146's fresh sweep, owned here: probes-numeric/t2146-binom-negbinom-pmf-fresh Calc!A5:A6,
  A10, A13, A15, A19:A22, A24, A27, A29:A30, A33:A35, A38, A45, A47, A50:A51, A55:A56, A60, A62, A65, A69:A70, A76,
  A86:A87, A93, A104, A110, A117:A118, A120:A121, A124:A128, A130:A132, A135:A136, A139, A144, A148:A149,
  A151:A152, A155, A157:A158, A160:A163, A165, A173:A175, A177:A178, A180, A186, A195, A197, A199:A200,
  A202:A203, A206, A209:A212, A215, A218, A220:A221, A224, A226:A229, A233, A236:A237, A239:A240, A245, A251,
  A254:A256, A258, A264, A267:A269, A272, A274, A279, A288, A294, A297:A299, A301, A304, A306, A311,
  A314:A315, A317:A318, A320, A361:A363, A373, A376, A378, A382, A385, A393, A398 (131 cells).
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
