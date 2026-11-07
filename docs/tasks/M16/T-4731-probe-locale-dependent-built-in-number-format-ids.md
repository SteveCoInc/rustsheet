---
id: T-4731
title: "Probe locale-dependent built-in number format ids and custom id numbering on save"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4169]
---

## Goal
The engine's built-in number format table and its custom-id numbering are backed by Excel evidence beyond id 14.

## Context
T-4169 changed `DEFAULT_NUM_FMTS[14]` to `m/d/yyyy` on evidence (probes/t4169-pivot-number-formats P1!D5:D11) and made the engine number new custom formats from 164 (`get_new_num_fmt_index`, crates/rustsheet-engine/src/number_format.rs). The 164 start is a convention matching XlsxWriter and ECMA-376 §18.8.30 (ids below 164 are built-in or reserved); how Excel itself numbers custom formats when it saves is not probed. Also unprobed: how Excel en-US shows the other built-ins that may depend on the locale (15-17, 22, and the East Asian 27-36 and 50-81). number-formats.md "Built-in id 14 is the short date" and "Custom number format ids start at 164 (convention)" list them.

## Acceptance criteria
- [ ] A probe of plain (non-pivot) cells styled with ids 14, 15-17, 22, 27-36 and 50-81 records Excel's displayed text, filled with excel-graph; the engine matches it or each off cell has an owner.
- [ ] A workbook saved by Excel shows which ids it gives new custom formats; number-formats.md records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US.
