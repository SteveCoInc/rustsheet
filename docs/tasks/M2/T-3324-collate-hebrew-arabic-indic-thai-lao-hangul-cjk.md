---
id: T-3324
title: "Collate Hebrew, Arabic, Indic, Thai, Lao, Hangul, CJK, symbols and astral characters as Excel does (T-3276 probe)"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`cmp_text_excel` orders Hebrew, Arabic, Devanagari, Thai, Lao, Hangul, CJK, the symbols above `€` and
astral characters as Excel does, so `=`/`<`, the criteria and the XMATCH/XLOOKUP `CollationKey` pass
the collation cells of probes/t3276-collation-other-scripts.

## Context
- Rules and cells: docs/excel-semantics/coercion.md "Other scripts, their marks and digits, symbols
  and astral characters (T-3276)". In short:
  - Block order: Latin, Greek, Cyrillic, kana, Hebrew, Arabic, Devanagari, Thai, Lao, the astral
    `𝐀`/`😀`, Hangul, CJK, then `𠀀`. Code-point order inside each script.
  - Hebrew points, Arabic harakat and the Thai tone mark are accents. Devanagari matras and the
    virama are letters.
  - Thai `เ`/`แ` sort before the consonants; Lao `ເ` keeps its code-point place.
  - Other-script digits are their digit plus a tiebreak (ASCII < Arabic-Indic < Devanagari).
  - Symbols `← ☺ ♥ ✓` sort between `>` and `0`, `∞` between `9` and `a`, `™` between `Ab` and `ÿ`.
- Ours today: `element` in crates/rustsheet-engine/src/functions/util/collation.rs weighs these
  `OTHER_BASE + code point` and astral characters from `UNIT_BASE`; `punctuation` stops at `€`.
- Failing cells:
  - probes/t3276-collation-other-scripts.case.json Calc!A8:A96, Calc!A98:A99, Calc!G6, Calc!G16, Calc!G20:G23, Calc!G27:G28, Calc!G31:G32

## Acceptance criteria
- [ ] Every Calc!A, B, D and G cell of probes/t3276-collation-other-scripts passes, and no collation
      probe regresses (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Digits in VALUE and numeric criteria (T-3325).
- Kana and full-width forms (T-3321).

## Outcome
`collation.rs` places the characters after the letters by `script_order`: Latin, Greek and Cyrillic,
then kana, Hebrew, Arabic, Devanagari, Thai (prevowels before the consonants), Lao, astral planes
before the ideographs, Hangul, CJK ideographs, then the astral ideographs. Lone surrogates are units
in the same order, so probes/text-lone-surrogates still passes. Hebrew points, Arabic harakat and
Thai tone marks are accents. Arabic-Indic and Devanagari digits are their digit with a variant
tiebreak. Arrows, symbols and dingbats sort after the punctuation, `∞` between `9` and `a`, and `™`
as `tm` with a variant weight. The primary bases moved up to make room for the symbols.
Unit test `t3276_other_scripts`. The unprobed choices are listed in docs/excel-semantics/coercion.md.
Eval delta: probes known failures 3865 -> 3764 (101 fixed, all
probes/t3276-collation-other-scripts, now 643/643), 0 regressions in every suite.
