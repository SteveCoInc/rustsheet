---
id: T-4657
title: "Probe pivot label text: number and date item subtotals, page selections, showAll beyond one field"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's rendered text is evidenced for the pivot labels the T-4149 renderer reads without a probe, so each becomes VERIFIED or is corrected in docs/excel-semantics/pivots.md.

## Context
The renderer (crates/rustsheet-engine/src/pivot/layout/render.rs, T-4149) takes these readings as its own (pivots.md "Rendered pivot cells", UNVERIFIED T-4657):
- the subtotal label of a number item and of an ungrouped date item (the engine writes `number_to_text` of the value and ` Total`, the date as its serial);
- a page field whose hidden items leave exactly one item visible (the engine shows `(Multiple Items)`), and a page field selecting a number, boolean or blank item (the engine shows the item as a row label renders it);
- `showAll="1"` on an axis of two fields, on the outer and on the inner one, and on a field whose items the page filter leaves without records (the engine applies `showAll` only to an axis of one field, `pivot/result.rs`).
Case builder v2 (`tools/src/rustsheet_tools/pivot_definition.py`) can author all of them; read the cells back from the sidecar's `pivots` section.

## Acceptance criteria
- [ ] A filled probe case (`probes/t4657-...`) covers each reading above, and pivots.md records the result with the evidence cells.
- [ ] Each reading the probe contradicts has a follow-up engine task, or the fix if size S.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number formats of labels and values (PV2, T-4169).
