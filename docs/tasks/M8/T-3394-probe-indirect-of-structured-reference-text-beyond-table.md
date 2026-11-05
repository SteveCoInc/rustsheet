---
id: T-3394
title: "Probe INDIRECT of structured-reference text beyond Table[Column]: specifiers, @, unqualified, case, spacing, bare name, a1 FALSE"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's result for INDIRECT of structured-reference text other than `Table[Column]` is probed, and the engine matches it.

## Context
T-2911 resolves only `INDIRECT("Table[Column]")` (`indirect_structured` in functions/lookup_and_reference/mod.rs); every other shape stays `#REF!` until probed: `#All`/`#Headers`/`#Data`/`#Totals`, `[@Col]`/`#This Row`, unqualified `[Col]` inside a table, case and spacing of the text, a bare table name, and `a1` FALSE. Record results in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [x] Probe cases with Excel values exist for each shape listed in Context, and the engine passes them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probe probes/t3394-indirect-structref-shapes (65 cells, excel-graph 2026-11-05): each cell reports the first
  row, first column, rows and columns of `INDIRECT(text)` for one shape, over a table `Sales` with a totals row.
  Excel resolves every structured reference that names its table as in a formula: the specifiers and their
  combinations, column ranges, a bare name (`Sales`, `Sales[]` = the data area), `@` / `#This Row` at the INDIRECT
  cell's row (also from another sheet; `#REF!` off the data rows), any case, `a1` FALSE, and with a sheet prefix,
  which is ignored even when it names a sheet without the table. An unqualified `[Qty]`, `[@Qty]`, `[#Data]` is
  `#REF!` even inside the table; whitespace follows the A1 rule (leading `#REF!`, trailing spaces and line feeds
  ignored, trailing tab `#REF!`), and spaces around a column name or before the bracket are `#REF!`.
- `indirect_structured` now takes every qualified shape (resolving `#This Row` with `Node::this_row_at`), strips a
  sheet prefix that names an existing sheet, applies the whitespace rule, and runs for `a1` FALSE too. The two
  INDIRECT fallbacks (defined name, structured reference) moved to the new
  `functions/lookup_and_reference/indirect_fallbacks.rs` (mod.rs was over the 1,000-line cap).
- docs/excel-semantics/lookup.md: new entry "INDIRECT of structured-reference text: every qualified shape"; the
  T-2911 entry and the whitespace entry point to it. Unprobed and kept `#REF!`: a sheet prefix naming no sheet.
- Eval delta: probes +1 case, 65/65 cells pass (no known failure added or fixed elsewhere; every suite unchanged).

