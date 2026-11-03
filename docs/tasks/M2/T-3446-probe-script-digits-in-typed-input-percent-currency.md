---
id: T-3446
title: "Probe script digits in typed input, percent, currency, parentheses, groups and exponents (T-3325 follow-up)"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's reading of Arabic-Indic, Devanagari and Thai digits is probed and recorded for typed cell input, percent, currency, accounting parentheses, group separators, exponents, dates and times, and other digit blocks (Bengali, Lao, Tibetan, Myanmar).

## Context
T-3325 maps the three blocks in `cast::number_text_in` (coercion, VALUE, criteria) from VALUE/`+0`/COUNTIF evidence only. Typed input is deliberately not mapped; the other forms follow the mapping by inference. See docs/excel-semantics/coercion.md "Other scripts ... (T-3276)". Use the excel-probe skill.

## Acceptance criteria
- [x] A probe case with oracle values covers each form above; coercion.md records results; engine changed (or follow-up filed) where the probe disagrees.
- [x] `make check` green; reviewers approve.

## Out of scope
- Collation of these digits (T-3324).

## Outcome
- Probe probes/t3446-script-digits-forms (excel-graph, 2026-11-03, 516 checked cells): nine scripts
  (ASCII and eight digit blocks: Arabic-Indic, Extended Arabic-Indic, Devanagari, Bengali, Thai, Lao,
  Tibetan, Myanmar) x eleven forms (plain, sign, decimal, percent, `$`, parentheses, group commas,
  exponent, ISO date, clock, mixed fraction), typed and through `+0`, VALUE, NUMBERVALUE and TEXT,
  plus mixed scripts and the Arabic separators U+066B/U+066C/U+066A. Excel reads all eight blocks in
  every form; the Arabic separators are not read; typed text is a number with a locale-tagged format
  (ENTERED log: docs/research/t3446-script-digits-forms-2026-11-03.md). Recorded in
  docs/excel-semantics/coercion.md "Script digits in number text (T-3446)".
- Engine: `script_digit_to_ascii` maps the five new blocks; `ascii_script_digits` is applied in
  `cast::number_text_in` before the clock reader too (`"١٢:٣٠"+0` was `#VALUE!`) and before both
  mixed-fraction readers (`"١ ١/٢"+0`). Every coercion, VALUE, NUMBERVALUE and TEXT cell of the case
  passes (424 of 516).
- Follow-ups: T-3891 (opus, typed script digits with Excel's tagged formats; owns B12:B102) and
  T-3894 (sonnet, bare typed 24-hour clock `12:30` stays text in the engine; owns B10).
- Eval delta: probes +1 case, 424 new passing cells, 92 new known failures (all owned typed-input
  cells), 0 regressions, 0 fixed elsewhere.
