---
id: T-3559
title: "Probe where Excel collates the scripts and symbol blocks T-3276 left unprobed (Bengali, Tamil, Georgian, Armenian, Ethiopic, math and box symbols, emoji sequences, other planes)"
milestone: M2
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
An oracle-filled probe case, and a coercion.md entry, that places each unprobed script and symbol block in Excel's
collation order relative to the blocks T-3276 placed. With it, `cmp_text_excel` can order the rest of Unicode from
evidence. Today the rest of Unicode keeps an inferred "code-point order after the letters".

## Context
- docs/excel-semantics/coercion.md, "Text comparison uses Excel's collation, not code-point order", says:
  "Every other character (other scripts, such as Bengali, Tamil, Georgian, Armenian and Ethiopic, and other symbol
  blocks) keeps code-point order after the letters, UNVERIFIED." The T-3276 section ends with: "UNVERIFIED: whether
  `™` is an expansion (`"tm"`) or a symbol placed between `a` and `y` ...; where `∞` sits against the punctuation
  other than digits; where other astral characters (other planes, emoji with modifiers) sit; whether Thai vowels
  other than `เ แ า` and other Lao vowels follow the same split; Hebrew and Arabic letters other than those probed."
- T-3276 (probes/t3276-collation-other-scripts, excel-graph 2026-10-31) found that block order is not code-point
  order: kana come before Hebrew, and `𝐀`/`😀` sit between Lao and Hangul. Inferring the place of the unprobed
  blocks from code points is therefore unsafe.
- Engine today: `element` in crates/rustsheet-engine/src/functions/util/collation.rs weighs every unlisted character
  `OTHER_BASE + code point`, and astral characters from `UNIT_BASE`. T-3324 implements the probed blocks only. This
  probe gives the follow-up its rules.
- Method: reuse T-3276's layout (Data!A strings, Calc!A count-less rank, B count-equal, C exact MATCH, D XMATCH,
  E COUNTIF `"="&x`, G pairwise `(x<y)+2*(x=y)+4*(x>y)`), built with `rustsheet-gen` or a spec case and filled with
  `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. Include anchor strings from T-3276
  (`a`, `z`, `ω`, `я`, `あ`, `א`, `ا`, `क`, `ก`, `ກ`, `𝐀`, `😀`, `가`, `一`, `𠀀`, `0`, `9`, `~`, `€`, `←`, `✓`) so
  each new block is bracketed.
- Do not reprobe what other open tasks own: kana and width forms (T-3381), U+0300-U+036F marks and the Latin, Greek
  and Cyrillic accent weights (T-3203, T-3305), `İ` (T-3180), script digits in VALUE and typed input (T-3446), and
  the case-fold sweep (T-3298).

- T-3324 (PR #2875) extended the probed points to whole ranges with no evidence. Each is UNVERIFIED in coercion.md
  "Other scripts, their marks and digits, symbols and astral characters", and this probe should settle it:
  - every arrow (U+2190 to U+21FF), miscellaneous symbol and dingbat (U+2600 to U+27BF) sorts after `•` and before
    the digits (from `← ☺ ♥ ✓`);
  - all Hebrew points and cantillation marks, Arabic harakat (U+0610 to U+061A, U+064B to U+065F, U+0670) and Thai
    tone marks U+0E48 to U+0E4B are accents weighing their code point (from U+05B7, U+05BC, U+064E, U+0651, U+0E48);
  - the Thai prevowels `โ ใ ไ` sort before the consonants, as `เ แ` do;
  - `™` is not equal to `"tm"` (the `assert_ne!` in `t3276_other_scripts`, crates/rustsheet-engine/src/test/test_collation.rs);
  - Thai and other digit blocks keep code-point order, unlike the Arabic-Indic and Devanagari digits.
  The engine now orders the rest of Unicode with `script_order` (crates/rustsheet-engine/src/functions/util/collation/scripts.rs),
  not by `OTHER_BASE + code point`.

## Acceptance criteria
- [x] `eval/suites/probes/cases/t3559-collation-unprobed-blocks.case.json` (with its `.oracle.json` sidecar) is committed and
      filled on excel-graph. It ranks at least two letters per block (first and last code point, plus one with a
      combining mark where the script has them) for Bengali, Gurmukhi, Tamil, Telugu, Georgian, Armenian, Ethiopic,
      Mongolian or Tibetan, Khmer and Myanmar. It also ranks: the Thai vowels `ใ ไ โ ะ ำ` and the Lao vowels
      `ແ ໂ ໃ ໄ ະ າ`; Hebrew `ש ץ` and Arabic `ء ى ﻻ`; symbols from the math operators (`∑ ≤ ≠ ∫`), box drawing (`─ █`),
      currency (`₹ ₩ ₿`), CJK punctuation (`、 。 「`), letterlike (`℃ №`) and geometric (`■ ●`) blocks; `™` against
      `tm` and `TM`, and `∞` against `!`, `#`, `%`, `+` and `~`; emoji with a skin-tone modifier, a ZWJ sequence and a
      regional-indicator flag, and characters from planes 1, 2 and 3 other than the probed ones (`𐀀`, `🀄`, `𪜀`, `𰀀`).
- [x] coercion.md replaces the "keeps code-point order after the letters, UNVERIFIED" sentence and the T-3276
      UNVERIFIED list with the observed rules, citing cells. Every remaining guess stays marked UNVERIFIED.
- [x] The cells the engine fails are known failures in eval/baseline/probes.txt (`make eval-baseline`, 0
      regressions), and an engine task to fix them is filed with `make new-task` (M2, engine). It lists the cells
      one line per case, and names T-3324, and T-3324 names it, if both touch `element`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's collation (the filed follow-up and T-3324).
- Non-en-US locales (criteria.md: out of scope).
- Desktop Excel (M14).

## Outcome
- New probe `probes/t3559-collation-unprobed-blocks` (excel-graph, 2026-11-03): 155 strings in
  T-3276's layout (rank, count equal, MATCH, XMATCH, COUNTIF) plus 25 pairwise cells, 955 cells.
- docs/excel-semantics/coercion.md gains "Scripts and symbol blocks beyond T-3276 (T-3559)" and
  replaces the "code-point order after the letters" sentence and the T-3276 UNVERIFIED list. Main
  findings: three late-Unicode characters (`➿ ᡸ ₿`) weigh the same, just above `""`; symbols sort
  among the punctuation in an order that is neither code point nor block, `•` after the arrows and
  dingbats (refutes T-3324's choice); `™` sorts after `"tn"` (refutes "`tm` with a variant
  weight"); Armenian and Georgian sit between Cyrillic and kana, Myanmar, Mongolian and Tibetan
  between Thai and Lao, Khmer and Ethiopic after Lao, plane 3 before Hangul (refutes "units from
  U+D840 after the ideographs" for plane 3); Gurmukhi, Georgian and Lao have exceptions to
  code-point order; `"Ա" = "ա"`, `"ץ" = "צ"`, `"ﻻ" = "لا"`; Thai `โ ใ ไ` go with `เ แ` (confirms
  T-3324); marks are tiebreaks by code point (confirms); every other-script digit is its digit plus
  a tiebreak, and COUNTIF `"=3"` counts eleven spellings of 3.
- Engine gap filed as T-3917 (M2, engine, opus); T-3324 names it. 181 of 955 cells fail and are
  known failures.
- Eval delta: probes +1 case, 774/955 cells pass, 181 new known failures, 0 regressions.
