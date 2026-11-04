---
id: T-3891
title: "Typed cell input reads script digits as numbers with Excel's locale-tagged formats (T-3446 follow-up)"
milestone: M5
lane: engine
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
Text of Arabic-Indic, Extended Arabic-Indic, Devanagari, Bengali, Gurmukhi, Tamil, Telugu, Thai, Lao, Tibetan, Myanmar or Khmer digits
typed into a cell becomes the number Excel makes of it, with the number format Excel gives it,
instead of staying text.

## Context
probes/t3446-script-digits-forms (excel-graph, 2026-11-03): every typed form of the eight blocks
(plain, signed, decimal, percent, `$`, parentheses, group commas, exponent, ISO date, clock, mixed
fraction) is a number in Excel (B12:B99 TRUE, B100:B102 TRUE for mixed scripts; B103:B105 stay text
with U+066B, U+066C and U+066A). The engine keeps them text: `set_user_input` (model.rs) calls
`parse_typed_number` on the raw text; coercion maps the digits through `ascii_script_digits`
(formatter/parse_formatted.rs) since T-3325/T-3446.
The formats Excel gives (oracle ENTERED log, docs/research/t3446-script-digits-forms-2026-11-03.md)
carry a locale and numeral-system tag per block (`ar-SA,200`, `ar-SA,300`, `hi-IN,400`, `bn-IN,500`,
`th-TH,D00`, `lo-LA,E00`, `bo-CN,F00`, `my-MM,1000`): an integer is `[$-ar-SA,200]0` (not General),
`١.٥` is `[$-ar-SA,200]0.#`, `١٢%` is `[$-ar-SA,200]0.00%` (ASCII `12%` is `0%`), `$١٢` is
`[$$-ar-SA,200] #,##0.00_);[Red]([$$-ar-SA,200]#,##0.00)`, `١,٢٣٤` `[$-ar-SA,200]#,##0`, a date
`[$-,200]mm/dd/yyyy`, a clock `[$-,200]h:mm`; the exponent form keeps `0.00E+00` and the mixed
fraction `# ?/?` with no tag. Mixed scripts: `1٢3` takes `ar-SA,200`, `١२` takes `hi-IN,400`.
docs/excel-semantics/coercion.md "Script digits in number text (T-3446)".

## Acceptance criteria
- [x] Probe the format rule's open shapes first (excel-probe): two or more decimals (`١.٢٥`), decimal
      percent (`١٢.٥%`), grouped decimals, `-$١٢`, `(١,٢٣٤)`, slash dates, clock with seconds and AM/PM,
      which block tags a three-script mix, and how the tagged formats render through TEXT/displayed text.
- [x] Typed script-digit text is a number with Excel's format for every probed shape; probes/t3446-script-digits-forms
      B12:B102 and probes/t3958-gurmukhi-tamil-telugu-khmer-digits Sheet1!B1:B44 (Gurmukhi, Tamil, Telugu, Khmer typed text) pass; the engine's number formatter accepts the `[$-xx-YY,N00]` tags it now stores.
- [x] Probe the typed-format tags of the Gurmukhi, Tamil and Telugu blocks and the Khmer shapes T-3958 did not log (a fresh fill of a new case prints the ENTERED lines; commit them under docs/research/).
- [x] `make check` green; reviewers approve.

## Out of scope
- The bare ASCII clock `12:30` typed (T-3894).
- Collation of these digits (T-3324).

## Outcome
- Probed (excel-graph 2026-11-04): probes/t3891-typed-script-digit-shapes (86 typed texts: the open
  Arabic-Indic shapes, three-script mixes, and the eleven forms in Gurmukhi, Tamil, Telugu and Khmer; ENTERED
  log in docs/research/t3891-typed-script-digit-shapes-2026-11-04.md) and probes/t3891-numeral-tag-render (30
  TEXT cells over the `locale,N00` tags). Rules in coercion.md "Script digits in number text" and
  number-formats.md "Numeral-system digits": the last script digit picks the tag, a plain number counts its
  typed decimals (`0.##`), every percent is `0.00%`, every currency the long form, dates take
  `mm/dd/yyyy`/`dd-mmm-yyyy`, exponent and fraction stay untagged; `,N00` is the numeral byte above a low byte.
- `parse_typed_number` (formatter/parse_formatted.rs) maps script digits, parses the ASCII text and gives
  Excel's format (`script_digit_format`, tags from `SCRIPT_DIGITS`, which `script_digit_to_ascii` now reads
  too); `locale_tag::numerals` reads the `,N00` suffix, so the stored formats print the script's digits.
- Cells: t3891-typed-script-digit-shapes 170/172, t3891-numeral-tag-render 28/30, and the typed cells of
  t3446-script-digits-forms B12:B102 and t3958-gurmukhi-tamil-telugu-khmer-digits B1:B44 now pass.
  Eval delta: probes known failures 135 fixed (t3446 B12:B102, t3958 B1:B44), 0 regressions; the two new cases
  add 4 owned known failures (below).
- Follow-ups (owners of the four visibly-off cells): T-4117 (typed `$(12)`, B29:C29), T-4107 (`,2000`
  prints fullwidth, render A28), T-4108 (a currency tag after a literal paren, render A6).
