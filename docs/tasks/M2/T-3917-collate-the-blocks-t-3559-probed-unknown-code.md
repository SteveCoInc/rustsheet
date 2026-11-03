---
id: T-3917
title: "Collate the blocks T-3559 probed: unknown code points, symbol order, more scripts, Lao and Gurmukhi exceptions, equal forms, other-script digits"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`cmp_text_excel` orders the characters probes/t3559-collation-unprobed-blocks ranks as Excel does, so
that case passes with no known failures.

## Context
- Evidence and rules: docs/excel-semantics/coercion.md, "Scripts and symbol blocks beyond T-3276
  (T-3559)" (excel-graph, 2026-11-03). Probe: eval/suites/probes/cases/t3559-collation-unprobed-blocks.
- Code: `element`, `script_order`, `symbol` and `digit_variant` in
  crates/rustsheet-engine/src/functions/util/collation.rs and collation/scripts.rs (T-3324 placed the
  T-3276 blocks there). T-3324 is done; this task follows it in `element`.
- What the engine gets wrong (rules from coercion.md):
  1. `➿` U+27BF, `ᡸ` U+1878 and `₿` U+20BF weigh the same, just above `""` (B35, B62, B126 = 3).
  2. Symbols sort among the punctuation in Excel's order, not by block: `! # $ % 、 。 ~ 「 ₩ € ₹ + ≠ ∑
     ∫ ≤ ■ ● ← ⇿ ─ █ ☀ ✓ •` then digits; `•` after the arrows (G16, G18).
  3. `™` sorts after `"tn"` (G5); `∞` between `"9"` and `"a"`; `℃ №` between `"a"` and `"tm"`.
  4. Block order: Armenian and Georgian between Cyrillic and kana; Bengali, Gurmukhi, Tamil, Telugu
     after Devanagari; Myanmar, Mongolian, Tibetan between Thai and Lao; Khmer, Ethiopic after Lao;
     plane 3 (`𰀀`) before Hangul, plane 2 after the ideographs.
  5. Exceptions to code-point order: Gurmukhi `ਹ` before `ਕ`; Georgian Mkhedruli before `Ⴀ`; Lao
     `ໄ` before `ໃ`.
  6. Equal forms: `"Ա" = "ա"` (G15), `"ץ" = "צ"` (G14), `"ﻻ" = "لا"` (G13).
  7. Other-script digits (Bengali, Gurmukhi, Tamil, Telugu, Tibetan, Myanmar, Khmer) are their digit
     plus a tiebreak by code point; Thai and Lao digits are T-3895's. COUNTIF `"=3"` counts all eleven
     spellings of 3 (E11, E136:E145 = 11), so `cast::number_text_in` must also read Gurmukhi U+0A66,
     Tamil U+0BE6, Telugu U+0C66 and Khmer U+17E0 digits (T-3446 covered eight other blocks).
- Failing cells (eval/baseline/probes.txt, one case):
  - probes/t3559-collation-unprobed-blocks: Calc!A2:A27, A30:A145, A154, A155; B35, B62, B97, B98,
    B109, B110, B126; C62, C98, C126; D62, D98, D110, D126; E35, E62, E97, E98, E109, E110, E126; G5, G13, G14, G16, G18.
- If this is larger than size M, split it by the numbered rules above (planner), keeping 7 with T-3895.

## Acceptance criteria
- [ ] Every cell of probes/t3559-collation-unprobed-blocks passes, its lines leave
      eval/baseline/probes.txt, and probes/t3276-collation-other-scripts and
      probes/t3465-collation-holdout regress no cell.
- [ ] Unit tests in crates/rustsheet-engine/src/test/test_collation.rs pin each rule 1 to 7.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The Thai and Lao digit order (T-3895) and the middle dot (T-3896).
- Collation of scripts neither T-3276 nor T-3559 probed.

## Progress
- Rule 7 done (2026-11-03, PR "T-3917 (partial)"): `element` gives the Gurmukhi U+0A66, Tamil U+0BE6, Telugu
  U+0C66 and Khmer U+17E0 digits a `digit_variant` weight, all twelve blocks in code-point order (Thai and Lao
  from T-3895), and `script_digit_to_ascii` reads them as digits. Unit test `script_digits_tiebreak_by_block`.
  Fixed E11 and E136:E145 (probes 5102 → 5091, 0 regressions). The rank cells A30:A41 still fail on rules 1 to 6
  (other characters ranked wrong around them), so they stay in the list above.
