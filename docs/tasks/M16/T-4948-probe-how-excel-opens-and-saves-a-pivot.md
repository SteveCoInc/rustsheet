---
id: T-4948
title: "Probe how Excel opens and saves a pivot dataField with an attribute-less or prefixed x14 dataField extension"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for two dataField extension shapes the exporter can write but no Excel-saved file
shows: what Excel for the web renders for each on open, and what it saves back.

## Context
T-4759's exporter (`crates/rustsheet-xlsx/src/export/pivots/show_as.rs`) treats the model's
show-values-as mode as the truth. Two of its outputs have no oracle evidence (pivots.md
"Show-values-as modes as Excel for the web renders them", UNVERIFIED entry):
- Removing a stale x14 mode (the model's mode is ECMA or `normal`) drops `pivotShowAs` from the
  kept `<x14:dataField>`, which can leave an attribute-less `<x14:dataField/>` in its `<ext>`.
- A kept `extLst` in a prefixed main namespace (`<x:extLst>`) gains the extension as `<x:ext>`.
  Excel-saved files write the main namespace unprefixed, so the prefixed form is not seen there.

Use the `excel-probe` skill. Cases, one pivot each (rows Region and Product, Sum of Sales, as in
probes/t4129-pivot-show-as-x14):
- a dataField with `showDataAs="percentOfTotal"` and an attribute-less `<x14:dataField/>` ext;
- a dataField with no `showDataAs` and an attribute-less `<x14:dataField/>` ext;
- a dataField whose `<x:extLst>` holds `<x:ext uri="{E15A36E0-9728-4e99-A89B-3F7291B0FE68}">`
  with `pivotShowAs="rankDescending"`, the part's main namespace prefixed `x`.

## Acceptance criteria
- [ ] Each case is a committed probe with Excel's rendered values, or a recorded refusal (Excel
      for the web saves nothing for a file it refuses).
- [ ] What Excel saves back for each dataField's extension is recorded in pivots.md, and the
      UNVERIFIED entry is replaced by the finding.
- [ ] If Excel refuses a shape or renders it differently, a follow-up task changes the exporter.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The import precedence of `showDataAs` versus `pivotShowAs` (T-4750).
