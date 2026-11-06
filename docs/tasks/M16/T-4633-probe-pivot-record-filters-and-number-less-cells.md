---
id: T-4633
title: "Probe pivot record filters and number-less cells: a malformed pageField@item, hidden items off every axis, cells with no number"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md states, from oracle evidence, three record-filter and aggregate edges that T-4142's result tree (`crates/rustsheet-engine/src/pivot/result.rs`) reads by hypothesis today:
- what a `pageField@item` that names no data item means: an index past the field's `<items>`, one naming a subtotal or blank line, or a value that is no number;
- whether `item@h` on a field that is on no axis (not row, column or page) still filters the records;
- what each aggregate shows in a cell whose records hold no number (only text, blanks or booleans): Sum, Average, Max, Min, Product, the variance family.

## Context
T-4139 reads a malformed `item` leniently as absent, `(All)`, and T-4142 keeps that reading. T-4142 applies hidden items only on axis fields, and gives a number-less cell the worksheet functions' answers (SUM 0, AVERAGE `#DIV/0!`, MAX, MIN and PRODUCT 0, STDEV and VAR `#DIV/0!`). Each is UNVERIFIED in docs/excel-semantics/pivots.md ("Record filters: page selections and hidden items" and "Aggregates over text, blanks, booleans and errors"). Excel for the web may refuse a file with a malformed page item outright (as it does inconsistent caches, pivots.md "Excel for the web renders authored pivot definitions"): if so, record the refusal as a builder observation and keep the lenient reading as the engine's own rule.

Build with case builder v2 (`tools/src/rustsheet_tools/pivot_definition.py`); a malformed page item may need a raw `attrs` override on the page field.

## Acceptance criteria
- [ ] A filled probe covers the three edges, or records why Excel refuses the file; pivots.md's UNVERIFIED parts are closed or narrowed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (a follow-up task implements what the probe finds).
