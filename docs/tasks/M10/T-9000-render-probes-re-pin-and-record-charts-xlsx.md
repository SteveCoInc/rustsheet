---
id: T-9000
title: "Render probes: re-pin and record charts.xlsx once T1's chart writer declares c14 on mc:Choice"
milestone: M10
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The `charts` render probe has a tier-1 reference: Excel for the web opens a re-pinned eval/render/probes/charts.xlsx and
`rustsheet-oracle render --probe charts` records its dump, chart images and print PDF in place of
eval/render/refs/graph/charts.refused.json.

## Context
- The current charts.xlsx (sha256 f254966247195b1a8ec1631f5024cd7ccca8e4e4625576d0d32df86e088c7114) is refused:
  createSession `FileCorruptTryRepair`, and the PDF conversion refuses it too. Cause: each chart writes
  `<mc:Choice Requires="c14"><c14:style xmlns:c14="…"/>`, with the prefix declared out of `Requires`' scope
  (docs/excel-semantics/xlsx-storage.md, "A chart's `mc:Choice` must have its `Requires` prefix in scope";
  eval/render/README.md, "First recording").
- The file comes from T1's chart writer, which is not in this repository. The writer fix (declare `xmlns:c14` on
  `mc:Choice`, as the Excel Online-saved eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx does) is T1's. This task waits for a corrected file, then pins and records it. Do not patch
  the bytes here: the pinned file must be the writer's own output.
- Blocked until T1 supplies the corrected workbook.

## Blocked because
T1 has not supplied a charts.xlsx whose chart parts declare `c14` on `mc:Choice`. The need is met when such a file
exists: `unzip -p charts.xlsx xl/charts/chart1.xml | grep -c '<mc:Choice Requires="c14" xmlns:c14='` prints 1, and
`rustsheet-oracle render --probe charts` records a dump.

## Acceptance criteria
- [ ] probes.json and SHA256SUMS pin the corrected charts.xlsx, and every chart part declares `c14` on `mc:Choice`.
- [ ] `rustsheet-oracle render --probe charts` writes charts.render.json (14 charts with placement), chart images and
      charts.pdf, and charts.refused.json is removed; `render --check` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The chart writer itself (T1's repository).
- Tier-2 chart fields (chart type), which Graph cannot read.
