---
id: T-3904
title: "= , COUNTIF and XMATCH(..,0) equate the weightless characters with one another (probes/t3584-match-fold-weightless; classes split to T-4013)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`=`, the COUNTIF-family text criteria and XMATCH's exact mode treat the 2568 BMP characters Excel's comparison
table gives no weight as Excel does: equal to one another, never equal to empty text, and counted or ignored by
position as T-3662's probe pins down. The other T-3584 findings for `=` (Greek Extended's classes, `ϵ` = `ε`,
U+1FBE = `ι`, `Ͷ`/`ϳ`, `ϑ`/`ϴ` and the Deseret case pair) are T-4013's.

## Context
- T-3584 found the set (docs/research/t3584/weightless-bmp.txt, 381 ranges) and the rule for the cells it probed:
  coercion.md "Weightless characters in `=`, COUNTIF and XMATCH (T-3584)". `"a"&UNICHAR(1316)&"b"="ab"` and
  `UNICHAR(1316)&UNICHAR(4295)=UNICHAR(1317)` are TRUE, but `UNICHAR(1316)=""` and `"a"&UNICHAR(1316)="a"` are FALSE.
- Where a weightless character counts is settled (T-3662): coercion.md "Format characters, U+200B and the C1
  controls (T-3662)", the prefix rule. Walk both texts from the start while both units are below U+007B and equal
  ignoring ASCII case; if one text ends during that walk, it is the smaller; otherwise compare by the collation,
  where weightless characters weigh nothing. The format characters U+00AD, U+200C:U+200F, U+2060, U+FEFF and
  U+FE0F are weightless like U+0524 (already in docs/research/t3584/weightless-bmp.txt).
- Design note: `CollationKey` must keep texts the pairwise test calls equal under equal keys, so the key drops
  weightless characters and the prefix rule only ever turns an equal-key pair unequal (XMATCH's index then
  confirms with `cmp_text_pairwise`).
- Failing cells: probes/t3584-match-fold-weightless Calc!B1:D32, F1:F32, P1:P85, all weightless pairs, except B21,
  D21, F21, B25, D25, F25, P7 and P9, which T-4013 owns (split 2026-11-04; the classes need no position rule).
- Also owned: probes/t3584-match-fold-weightless (54): Calc!R1, R4, R6-R17, R19-R20, R23-R30, R32-R34, R36-R37,
  R39-R40, R44-R51, R78, R160, R165, R167-R172, R216, R251-R252, R254-R256 (the `=` digits 4 and 8 of the BMP sweep;
  every MATCH digit already matches after T-3445 and T-3921).
- Also owned (T-3929): probes/t3929-wildcard-fold-criteria Calc!B3:D3, B7:D7 and
  probes/t3929-wildcard-fold-xmatch Calc!E3, E7: COUNTIF, SUMIF, COUNTIFS and XMATCH(..,0) without a wildcard
  equate a&U+0524&b with ab and U+0524 with U+10C7. Rows 29-30 (`Ͷ`/`ϳ`, `ϑ`/`ϴ`) are T-4013's.
- Also owned (T-3662, the format characters and the prefix rule):
  - probes/t3662-collation-ignorables (293): Calc!B1, B3:B9, B13:B17, C1, C3:C9, C13:C17, E1, E3:E9, E13:E17, F1, F3:F9, F13:F17, I1, I3:I9, I13:I17, M1, M3:M9, M13:M17, N1, N3:N9, N14:N17, O1, O3:O9, O13:O17, P1, P3:P9, P13:P17, Q1, Q3:Q9, Q14:Q17, S1, S3:S9, S13:S17, U1, U3:U9, U13:U16, V1, V3:V9, V13:V16, X1, X3:X9, X13:X17, Y1, Y3:Y9, Y13:Y17, Z1, Z3:Z9, Z13:Z17, AA1, AA3:AA9, AA13:AA16, AB3:AB9, AB13:AB17, AC1, AC3:AC9, AC13:AC17, AD1, AD3:AD9, AD13:AD17, AE1, AE3:AE9, AE13:AE17, AJ1, AJ3:AJ9, AJ13:AJ17, AL1, AL3:AL9, AL13:AL17
  - probes/t3662-collation-ignorables-prefix (44): Calc!B1:B4, G1:G4, H1:H4, J1:J4, K1:K4, M1:M4, N1:N4, O1:O4, P1:P4, T1:T4, U1:U4
  - probes/t3662-collation-ignorables-sweep (26): Calc!A123:A127, B123:B127, C123:C127, E1:E4, E6:E11, F2
  - probes/t3465-collation-holdout (18, from T-3662; an inserted soft hyphen): C18, E18, D69, E69, D117, E117, C132, E132, C153, E153, C222, E222, C234, E234, C579, E579, C597, E597
  Rows 2 and 10:12 of -ignorables and rows 10:13 of -prefix (U+200B and the C1 controls) are T-4039's.
- Also owned: probes/t3895-collation-script-digits A185 (`UNICHAR(3715)&"a"="a"` TRUE). T-3895 maps U+0E83, a
  weightless character, to a skipped one (collation.rs `UNASSIGNED`) for the hold-out's row 599; replace that
  stand-in with the weightless rule and keep A179:A188 of that case passing.
- Code: `cmp_text_excel` and `element` (crates/rustsheet-engine/src/functions/util/collation.rs) give these
  characters a code-point primary weight today. Exact MATCH, VLOOKUP and HLOOKUP are T-3445's (they count each
  weightless character one for one).

## Acceptance criteria
- [ ] The cells above pass and leave eval/baseline/probes.txt; 0 regressions (the collation hold-out
      probes/t3465-collation-holdout and probes/text-collation-order keep passing).
- [ ] The weightless table is one definition shared with T-3445's fold (no second copy of the ranges).
- [ ] coercion.md's section says "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Exact MATCH, VLOOKUP and HLOOKUP (T-3445). Supplementary characters (none is weightless).
- U+200B and the C1 controls, which have weights (T-4039).
- The class and Deseret cells (T-4013).
