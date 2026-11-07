---
id: T-4993
title: "Probe a collapsed middle pivot field with Values deeper and insertBlankRow on a field with Values inside"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4699]
---

## Goal
Excel's layout lines and cells are evidenced for the two pivot shapes T-4699 left UNVERIFIED, and the engine is corrected (or a follow-up filed) where it differs.

## Context
T-4699 (probes/t4699-pivot-blank-collapsed) probed blank rows and collapsed items beyond an outer row field; docs/excel-semantics/pivots.md "Blank rows and collapsed items beyond an outer row field" leaves these readings UNVERIFIED:
- a collapsed item of a middle row field with the Values field deeper still (for example rows Region, Product, Size, Values with Product `A` collapsed). The engine gives `A` its own line, then one line per data field with `A`'s totals, the `Size` level hidden (`x="1048832"`), as t4699 CV3 and CV4 show for an outer field;
- `insertBlankRow` on a row field with the Values field inside it (rows Region, Values with two data fields, and rows Region, Values, Product). The engine adds a `t="blank"` line after each item only when a field other than Values is inside it (lines.rs `blank_line`, `has_subtotals`), so none for rows Region, Values.
Code: crates/rustsheet-engine/src/pivot/layout/lines.rs (`AxisPlan::lines`, `blank_line`, `values_from`). Write the probe with the case builder (`collapsed`, `attrs.insertBlankRow`) and fill it with the excel-graph oracle.

## Acceptance criteria
- [ ] A filled probe case covers both readings; pivots.md records the result with the evidence cells, and the UNVERIFIED items are cleared.
- [ ] The engine's lines and cells pass the probe in the unit tests against the sidecar (crates/rustsheet-engine/src/pivot/layout/{lines,render}/tests.rs), or a follow-up task owns each difference.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
