---
id: T-4354
title: "Probe INDIRECT structured-reference text with an unquoted sheet prefix Excel's A1 text refuses, and fix lookup.md's Implemented-in path"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3394]
---

## Goal
INDIRECT of structured-reference text behind an unquoted sheet prefix that Excel's defined-name path refuses
(`"Co A!Sales[Qty]"`, `"P&L!Sales[Qty]"`, `"O'B!Sales[Qty]"`) and behind a prefix naming no sheet is probed, and the
engine matches Excel.

## Context
- T-3394 (PR #3369, reviewer-excel-fidelity MINOR findings): `indirect_structured`
  (crates/rustsheet-engine/src/functions/lookup_and_reference/indirect_fallbacks.rs) strips any sheet prefix that
  names an existing sheet, without the unquoted-sheet character rule (`is_indirect_unquoted_sheet`) that
  `indirect_defined_name` applies (probes/indirect-sheet-scoped-name: `"Co A!tn"` is `#REF!`). Only `S!`, `'S'!` and
  `P!` are probed (probes/t3394-indirect-structref-shapes P!B9, C8:C11), and a prefix naming no sheet is unprobed
  (the engine gives `#REF!`).
- docs/excel-semantics/lookup.md "INDIRECT of structured-reference text: every qualified shape" names `mod.rs` on
  its Implemented-in line; the function moved to indirect_fallbacks.rs in T-3394.

## Acceptance criteria
- [x] A probe case with excel-graph values covers sheets named `Co A`, `P&L` and `O'B` holding or not holding the
      table, each prefix unquoted and quoted, and a prefix naming no sheet; the engine passes it.
- [x] The lookup.md entry records the result and names indirect_fallbacks.rs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe probes/t4354-indirect-structref-sheet-prefix (excel-graph 2026-11-05, 19 cells; tables Sales on S, TabA on
  `Co A`, TabB on `P&L`, TabC on `O'B`): an unquoted prefix Excel's A1 text refuses is `#REF!` even on the sheet that
  holds the table (`Co A!TabA[Qty]`, `P&L!TabB[Qty]`, `O'B!TabC[Qty]`), the quoted forms resolve, also to another
  sheet's table (`'Co A'!Sales[Qty]`, `S!TabA[Qty]`), and a prefix naming no sheet is `#REF!` quoted or not.
- `indirect_structured` (indirect_fallbacks.rs) applies `is_indirect_unquoted_sheet` to an unquoted prefix, as
  `indirect_defined_name` does; before, the three unquoted shapes resolved (6 cells off). The no-sheet prefix was
  already `#REF!`, now evidenced.
- docs/excel-semantics/lookup.md records the rule and names indirect_fallbacks.rs on its Implemented-in line.
- Eval delta: new case, 19/19 pass; no other change.
