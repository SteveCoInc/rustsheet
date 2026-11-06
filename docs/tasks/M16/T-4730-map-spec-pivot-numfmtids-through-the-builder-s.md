---
id: T-4730
title: "Map spec pivot numFmtIds through the builder's styles for several custom formats"
milestone: M16
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4169]
---

## Goal
A spec pivot's data field `numFmtId` names the same format code in the engine adapter as in the built workbook Excel opens, whatever the number and order of the case's custom formats.

## Context
probes/t4169-pivot-number-formats passes its custom-format cells (P1!E5:E11, `numFmtId="164"`) only because the case has one custom format: the builder (XlsxWriter, `tools/src/rustsheet_tools/spec2xlsx.py`) numbers it 164 in its own order, and the engine adapter (`set_number_formats` in crates/rustsheet-eval/src/engine.rs, by sheet then BTreeMap address order, through `get_new_num_fmt_index`) also numbers its first custom format 164. With two or more custom formats the two orders can differ, so a definition's `numFmtId` could name another code in the engine than in Excel. The engine unit test `value_cells_show_their_data_fields_number_format` (crates/rustsheet-engine/src/pivot/grid/tests.rs) hand-sets Q!B1 for the same reason. Fix by mapping the definition's numFmtIds through the builder's styles (e.g. a definition key naming the format code, resolved by the builder to its styles.xml id and by the adapter to the engine's), or by loading the builder's `numFmts` into the spec adapter. UNVERIFIED (eval-harness) in pivots.md "Value cells show their data field's number format" and number-formats.md "Built-in id 14 is the short date".

## Acceptance criteria
- [ ] A probe with two or more custom formats, in an order where XlsxWriter's numbering and the adapter's BTreeMap order differ, filled with excel-graph, passes its value and text cells.
- [ ] The engine unit test no longer depends on the hand-set Q!B1 numbering.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number formats outside pivots.
