---
id: T-3958
title: "Probe Gurmukhi, Tamil, Telugu and Khmer digits in number text and collation; fix T-3917's rank-cell citations"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3917]
---

## Goal
The Gurmukhi (U+0A66), Tamil (U+0BE6), Telugu (U+0C66) and Khmer (U+17E0) digits have oracle evidence in every
number-text form and in the collation order the engine gives them. The evidence citations of T-3917's rule 7 name
the right cells.

## Context
- T-3917's partial PR (rule 7) added these four blocks to `script_digit_to_ascii` (formatter/parse_formatted.rs)
  and to `element` (functions/util/collation.rs). The only oracle evidence is the COUNTIF criterion
  (probes/t3559-collation-unprobed-blocks E11, E136:E145 = 11) and the rank order `"3" < ... < "៣" < "9"`. The other
  number-text forms (`+0`, VALUE, NUMBERVALUE, TEXT, `%`, `$`, `(12)`, `1,234`, `1e3`, dates, clocks, `1 1/2`) are
  inferred from T-3446's eight blocks (coercion.md "Script digits in number text (T-3446)"). So are `"៣" < "3a" < "4"`
  and the order against Extended Arabic-Indic and Devanagari (`test_collation.rs` `script_digits_tiebreak_by_block`).
- Review finding (T-3917 partial, fidelity MINOR): four places cite "probes/t3559-collation-unprobed-blocks ranks
  A30:A41" (collation/scripts.rs above `ARABIC_INDIC_DIGIT`, the digit comment in collation.rs `element`, the doc
  comment of `script_digits_tiebreak_by_block`, and T-3917's Progress note). The rank cells for the digit 3 of each
  script are Calc!A11 and A136:A145 (rank values 30 to 41). Calc!A30:A41 rank other characters.
- Probe like probes/t3446-script-digits-forms: the eleven forms for each of the four blocks (`+0`, VALUE,
  NUMBERVALUE, TEXT and typed, with the cell's number format), plus pairs that place each digit against `"3a"`,
  `"4"`, `"۳"` and `"३"`.

## Acceptance criteria
- [x] An oracle-filled probe covers the four blocks in the T-3446 forms and in the pairs above. The engine passes it,
      or each off cell has an owner.
- [x] The four citations say Calc!A11, A136:A145 instead of A30:A41.
- [x] coercion.md's T-3446 and T-3559 entries cite the new probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe probes/t3958-gurmukhi-tamil-telugu-khmer-digits (excel-graph, 2026-11-03, 244 checked cells): four blocks x the eleven T-3446 forms (typed, ISNUMBER, +0, VALUE, NUMBERVALUE, TEXT) and 28 collation pairs. The engine passes 200; the 44 off cells are the typed-input ISNUMBER cells Sheet1!B1:B44, owned by T-3891 (amended).
- The four rank-cell citations now name Calc!A11, A136:A145 (collation/scripts.rs x2, test_collation.rs; T-3917's Progress note). coercion.md's T-3446 and T-3559 entries cite the probe.
- Open: the typed-format tags of Gurmukhi, Tamil and Telugu were not logged (T-3891).

## Outcome
- Probe probes/t3958-gurmukhi-tamil-telugu-khmer-digits (excel-graph, 2026-11-03, 244 checked cells): four blocks x the eleven T-3446 forms (typed, ISNUMBER, +0, VALUE, NUMBERVALUE, TEXT) and 28 collation pairs. The engine passes 200; the 44 off cells are the typed-input ISNUMBER cells Sheet1!B1:B44, owned by T-3891 (amended).
- The rank-cell citations now name Calc!A11, A136:A145 (collation/scripts.rs x2, test_collation.rs, T-3917's Progress note). coercion.md's T-3446 and T-3559 entries cite the probe.
- Open: only the Khmer typed-format tags were logged; Gurmukhi, Tamil and Telugu tags are for T-3891.
