---
id: T-3904
title: "= , COUNTIF and XMATCH(..,0) equate the weightless characters with one another (probes/t3584-match-fold-weightless)"
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
position as T-3662's probe pins down.

## Context
- T-3584 found the set (docs/research/t3584/weightless-bmp.txt, 381 ranges) and the rule for the cells it probed:
  coercion.md "Weightless characters in `=`, COUNTIF and XMATCH (T-3584)". `"a"&UNICHAR(1316)&"b"="ab"` and
  `UNICHAR(1316)&UNICHAR(4295)=UNICHAR(1317)` are TRUE, but `UNICHAR(1316)=""` and `"a"&UNICHAR(1316)="a"` are FALSE.
- Where a weightless character counts (leading, inner, trailing; after a letter or punctuation) is open; T-3662
  probes it for the format characters, so this task waits for it. Add U+0524 to that probe if T-3662 has not.
- Failing cells: probes/t3584-match-fold-weightless Calc!B1:D32, F1:F32, P1:P85.
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
