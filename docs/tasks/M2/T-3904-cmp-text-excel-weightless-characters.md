---
id: T-3904
title: "= , COUNTIF and XMATCH(..,0) equate the weightless characters with one another (probes/t3584-match-fold-weightless; classes split to T-4013)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3662]
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
- Where a weightless character counts (leading, inner, trailing; after a letter or punctuation) is open; T-3662
  probes it for the format characters, so this task waits for it. Add U+0524 to that probe if T-3662 has not.
- Failing cells: probes/t3584-match-fold-weightless Calc!B1:D32, F1:F32, P1:P85, all weightless pairs, except B21,
  D21, F21, B25, D25, F25, P7 and P9, which T-4013 owns (split 2026-11-04; the classes need no position rule).
- Also owned: probes/t3584-match-fold-weightless (54): Calc!R1, R4, R6-R17, R19-R20, R23-R30, R32-R34, R36-R37,
  R39-R40, R44-R51, R78, R160, R165, R167-R172, R216, R251-R252, R254-R256 (the `=` digits 4 and 8 of the BMP sweep;
  every MATCH digit already matches after T-3445 and T-3921).
- Also owned (T-3929): probes/t3929-wildcard-fold-criteria Calc!B3:D3, B7:D7 and
  probes/t3929-wildcard-fold-xmatch Calc!E3, E7: COUNTIF, SUMIF, COUNTIFS and XMATCH(..,0) without a wildcard
  equate a&U+0524&b with ab and U+0524 with U+10C7. Rows 29-30 (`Ͷ`/`ϳ`, `ϑ`/`ϴ`) are T-4013's.
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
- The class and Deseret cells (T-4013).
