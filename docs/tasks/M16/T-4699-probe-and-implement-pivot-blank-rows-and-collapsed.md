---
id: T-4699
title: "Probe and implement pivot blank rows and collapsed items beyond an outer row field"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4168]
---

## Goal
The engine's readings of `insertBlankRow` and collapsed items (`item@sd="0"`) beyond what probes/t4168-pivot-layout-options shows are evidenced, and corrected where Excel differs.

## Context
Split from T-4168, which implemented the probed shapes (crates/rustsheet-engine/src/pivot/layout/lines.rs `blank_line`, `collapsed`; pivots.md "Layout lines"): a blank line after each item of an outer row field (BR tabular, BR2 compact) and a collapsed outer row item standing alone with its values (SD compact, SD2 tabular). Engine readings left UNVERIFIED (pivots.md, T-4699):
- `insertBlankRow` on the innermost row field (the engine adds no line) and on a column field (none);
- `insertBlankRow` after a collapsed item (the engine adds the blank line);
- a collapsed item on a column field (one column line, the item's tuple) and in a field with `subtotalTop="0"` or custom subtotals (the item's line shows its data field's own aggregate);
- a collapsed item in a middle field of three row fields.
- a collapsed item of a field with the Values field inside it (several data fields on rows): the engine gives it one line with no data field chosen, so the first data field's values;
- custom subtotal lines in plain outline form (`outline` on, `compact` off): the engine puts them after the children, as compact form does (t4168 F2).
Write `probes/t4699-...` with the case builder (`collapsed`, `attrs.insertBlankRow`) and fill it with the excel-graph oracle.

## Acceptance criteria
- [ ] A filled probe case covers each reading above; pivots.md records the result with the evidence cells.
- [ ] The engine's lines and rendered cells pass the probe in unit tests against the sidecar (crates/rustsheet-engine/src/pivot/layout/{lines,render}/tests.rs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Repeat item labels (T-4698).
