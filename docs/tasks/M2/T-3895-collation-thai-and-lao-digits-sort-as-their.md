---
id: T-3895
title: "Collation: Thai and Lao digits sort as their digits, before the letters, not in their script's code-point run (T-3465 sweep)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`<`, `=`, criteria and XMATCH order Thai (U+0E50..U+0E59) and Lao (U+0ED0..U+0ED9) digits as Excel does:
among the digits, below every letter and script, as the Arabic-Indic and Devanagari digits already are.

## Context
- The T-3465 hold-out sweep (probes/t3465-collation-holdout, excel-graph 2026-11-03) shows Excel putting
  a Thai or Lao digit below Latin, Cyrillic, kana, Hebrew, Arabic, Devanagari and Thai letters, and below
  `æ`, `Ĳ`, `í` and `ÿ`: `"ณ" > "๘"`, `"NPOECV" > "໖66"`, `"рхш" > "໙"`, `"๖ฏ๔ชศ" < "æ"`, `"໓" < "Ĳ"`.
  The engine keeps them in their script's code-point run after the letters: coercion.md "Other scripts ...
  (T-3276)", the UNVERIFIED choice "Thai and other digit blocks keep code-point order", which this sweep
  refutes for Thai and Lao.
- Row 599 starts with U+0E83, an unassigned code point in the Lao block (the generator's Lao span holds it),
  then a Lao digit: Excel orders it below `ÿ`, so either an unassigned point is ignorable or it sorts low.
  Probe that apart before deciding.
- The engine's collation: `element` / `script_order` in crates/rustsheet-engine/src/functions/util/collation.rs
  (Arabic-Indic and Devanagari digits are their digit plus a tiebreak, T-3276).
- Owned cells: probes/t3465-collation-holdout C76, D76, C103, D103, C128, D128, C172, D172, C175, D175, C231, D231, C233, D233, C260, D260, C274, D274, C343, D343, C371, D371, C388, D388, C410, D410, C481, D481, C529, D529, C531, D531, C532, D532, C589, D589, C599, D599

## Acceptance criteria
- [x] A probe (excel-probe skill) settles each Thai and Lao digit against its ASCII digit and its neighbours
      (`"2" < "๒" < "3"`?, equality, COUNTIF `"="&x`), the Bengali, Tibetan, Myanmar and Extended Arabic-Indic
      digits likewise, and U+0E83 alone, against `""` and `"a"`.
- [x] The engine orders them as probed; the owned cells above pass; coercion.md records the rule and drops the
      refuted UNVERIFIED choice.
- [x] `make check` green; reviewers approve.

## Out of scope
- Number-text reading of these digits (T-3446).

## Outcome
- **Probe** probes/t3895-collation-script-digits (191 cells, excel-graph, 2026-11-03): each of the Thai, Lao,
  Bengali, Tibetan, Myanmar and Extended Arabic-Indic blocks at digits 0, 3 and 9 against its ASCII digit, the next
  digit, `"Na"`, `"(N-1)z"`, the Arabic-Indic, Devanagari and full-width digit; the eight blocks pairwise; COUNTIF and
  exact MATCH against the text `"3"`; U+0E83 alone, leading, trailing and against `""` and `"a"`.
- **Finding.** All six blocks are their digit plus a variant tiebreak, like Arabic-Indic and Devanagari; the tiebreak
  orders the blocks by code point, after the full-width digit. U+0E83 is one of T-3584's weightless characters: a
  lone or trailing one counts, a leading one does not (`UNICHAR(3715)&"a"="a"` TRUE).
- **Code.** `element` in functions/util/collation.rs: one `digit_variant` weight per block (Devanagari moves from 4 to
  5 to keep code-point order); U+0E83 is a skipped character, a one-point stand-in for T-3904's weightless rule.
- **coercion.md**: new section "Thai, Lao and the other script digits (T-3895)"; the hold-out paragraph and the
  T-3559 notes point to it instead of the refuted UNVERIFIED choice.
- **Follow-up:** probes/t3895-collation-script-digits A185 (leading U+0E83 equal) goes to T-3904, which owns the
  weightless set; its task file now names the cell and the stand-in.
- **Eval delta.** probes known failures 5139 → 5102: the 38 owned cells of probes/t3465-collation-holdout fixed; the
  new case passes 192 of 193 (A185, T-3904). 0 regressions.
