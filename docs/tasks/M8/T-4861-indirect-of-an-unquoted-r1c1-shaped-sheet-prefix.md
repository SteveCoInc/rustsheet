---
id: T-4861
title: "INDIRECT of an unquoted R1C1-shaped sheet prefix with a later mark (C1+U+0301!B1) is #REF! (probe the boundary first)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
`INDIRECT("C1́!B1")` (an unquoted sheet prefix C1 + U+0301, R1C1-shaped, naming an existing sheet) is `#REF!` as in
Excel, and the rule's boundary across other R1C1-shaped unquoted prefixes is probed and matched.

## Context
- Evidence: probes/t4757-later-mark-sheet-names Sheet1!A19 (excel-graph, 2026-11-07, T-4757): Excel `#REF!`, ours 23.
  The quoted `INDIRECT("'C1́'!B1")` (A20) reads 23 in both. Every other unquoted later-mark prefix reads its sheet in
  both (A3, A7, A11, A15). See docs/excel-semantics/references.md "Characters of a name" (sheet names bullet) and
  "R1C1-shaped sheet and defined names", which quotes R1C1-shaped names in formula text (`is_r1c1_shaped`,
  `name_needs_quoting` in expressions/utils/mod.rs).
- Code: INDIRECT's A1 path (`fn_indirect` in functions/lookup_and_reference/mod.rs) parses the text with
  `ParsedReference::parse_reference_formula`. The fallbacks (indirect_fallbacks.rs) gate unquoted prefixes with
  `is_indirect_unquoted_sheet`.
- Unknown, so probe first (one excel-graph case, predictions committed before the fill): INDIRECT of the unquoted
  `C1D!B1`, `R1C1!B1`, `RC!B1`, `R1_!B1` and the out-of-range `R0!B1` against sheets so named. Also check whether the
  digit-leading exception (`INDIRECT("2024!B1")` reads sheet `2024`) applies to them. Then implement the rule the probe
  shows. Do not refuse more than it shows.

## Acceptance criteria
- [x] The probe case is committed with its excel-graph oracle, and references.md records its rule.
- [x] probes/t4757-later-mark-sheet-names Sheet1!A19 passes, and the new probe's cells pass (`make eval-baseline`,
      0 regressions).
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns has Excel's value or error (decision 0083).

## Out of scope
- Quoting of these names in formula text (already matched, T-1657/T-4446).

## Outcome
Probe `probes/t4861-indirect-r1c1-shaped-sheet-prefix` (excel-graph): INDIRECT of an unquoted R1C1-shaped prefix with more text (`C1D`, `R1_`) is `#REF!`; bare `R`, `C`, `RC`, `R1`, `C1`, `R1C1`, `R0` read their sheet; quoted forms all read. Rule in references.md, implemented as `is_indirect_r1c1_tail_sheet`. t4757 A19 now passes; the new probe is 16/16.

Round 2 (fidelity review): the probe now covers the tails the rule also refuses (`R1C1D`, `R1CD`, `R1C0D`, `RC1D`, `R1D`, `C1R1`, `R1048576D`, `C16384D`: `#REF!`) and the bare or out-of-range forms (`RC1`, `R1C`, `R0D`, `C16385D`, `RC16385D`: read), predictions committed before the refill; all held, so the rule is unchanged and now evidenced. The prefix test skips text before a `!`, `:` or `'` (two-prefix texts). t4757 A19 leaves the baseline as `make eval-baseline` wrote it. Unprobed, left as a note in references.md: `C1D!Name` (sheet-scoped name behind such a prefix).
