---
id: T-4965
title: "T-4939 review follow-ups: probe custom equal case folding, fix the and/or wording, reuse Row::hidden_default, eval cases for the patched uploads"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The custom-filter re-apply that T-4939 added rests on evidence for each rule it applies. Its docs describe the code
exactly, and its probes are gated by the eval.

## Context
These are the MINOR findings of PR #3991's review (T-4939):
- `CustomFilter::keeps_text` (crates/rustsheet-engine/src/table_types.rs) ignores case. The only custom upload
  (docs/research/t4895/uploads/custom) has lowercase operands over lowercase cells, so case folding is unprobed. The
  unit test pins `equal "A"` over "a" as kept.
- numbers.md ("The engine follows this") and T-4939's Outcome say the conditions are "combined by `and`". The code
  combines them by `and` or `or`, and the evidenced form is `or`.
- `Worksheet::set_row_style` (worksheet.rs) still builds a default `Row` by hand. It could use
  `..Row::hidden_default(row)` as `set_row_hidden` does.
- The `add_table` doc comment in crates/rustsheet-eval/src/engine.rs has one overlong line.
- The `parse::<f64>()` operand test also reads "inf"/"nan" as numbers. This is conservative, but it needs a comment
  saying why.
- The custom-filter and display-text uploads (docs/research/t4895/uploads, docs/research/t4939/uploads) exist only as
  research sidecars. The spec `"criteria"` form cannot express a custom filter or a display-text values list.

## Acceptance criteria
- [ ] Probe, with predictions committed before the fill: an upper-case custom `equal` operand ("A", "C") over cells a
      and c, deleting the other criterion's column. Patch the workbook as docs/research/t4895/uploads.py does. Then
      implement what Excel does, and record it in numbers.md.
- [ ] numbers.md and the T-4939 Outcome say `and` or `or`.
- [ ] `set_row_style` uses `Row::hidden_default`. The engine.rs comment is re-wrapped, and the numeric-operand
      comment is added.
- [ ] Either the spec form gains a way to write these filters (spec_tables.py and the eval adapter), with the uploads
      as probes cases, or the task records why not.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other custom operators, wildcards and numeric operands (unprobed; they stay `None`).
