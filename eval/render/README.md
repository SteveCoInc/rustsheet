# Render references for T1's render-fidelity gate (bar 3): what to record, in two tiers

The gate compares our grid's layout of each probe sheet with Excel's, starting with geometry and text. It needs one
dump per workbook, in two tiers, because the oracle reads Excel through the **Microsoft Graph workbook API**:

- **Tier 1** is what Graph can read. It runs today, and the layout it reports is Excel for the web's.
- **Tier 2** needs the Excel UI gym (an Office.js add-in plus Playwright, in progress) or Excel desktop. Until a
  tier-2 dump exists, the gate reports those fields as INCOMPLETE and never as passed. A tier-1 PASS on its own
  cannot make the obligation green.

Every claim about Graph below was checked against the Microsoft Graph v1.0 reference pages for `workbookRange`,
`workbookRangeFormat`, `workbookRangeFont`, `workbookRangeFill`, `workbookRangeBorder`, `workbookChart`,
`workbookWorksheet` and `driveItem: Convert to other formats`. Whatever those pages do not state is marked
**unverified**, and the first recording settles it.

## The probes (`probes/`)

Six workbooks with 22 sheets, pinned by sha256 in `probes/SHA256SUMS` and `probes/probes.json`. Upload them as they
are, and do not edit or re-save them: their bytes are the pin. `probes.json` gives each sheet's **probe range**.
- Set 1: `widths-heights`, `number-formats`, `alignment-wrap`, `fonts-fills-borders` and `merges-frozen`. Column A
  of every row says in OOXML terms what that row holds (for example `built-in 14`, `darkGrid fg=FF0000 bg=FFFF00`).
- `charts`: 14 charts, one per P0 chart type, at known anchors. It was written by the charts lane with the product's
  own chart writer.

Do **not** call `autofitColumns` or `autofitRows`. The gate needs the widths and heights the file opens with, and an
autofit would change them.

## Tier 1: the Graph dump, `refs/graph/<probe>.render.json` (schema `rustsheet.excel-render-graph/1`)

```jsonc
{
  "schema": "rustsheet.excel-render-graph/1",
  "probe": "number-formats", "sha256": "<the workbook's sha256, from probes.json>",
  "source": { "api": "graph-v1.0", "recordedAt": "2026-11-10T04:00:00Z" },
  "sheets": [{
    "name": "Codes",
    "columns": [{ "index": 1, "columnWidth": 192.75, "hidden": false }],   // every column of the range, once
    "rows":    [{ "index": 1, "rowHeight": 15, "hidden": false }],         // every row of the range, once
    "cells": [{                                                            // every cell of the range, once
      "a1": "B2",
      "text": "1,234.57", "value": 1234.567, "valueType": "Double", "numberFormat": "#,##0.00",
      "hAlign": "General", "vAlign": "Bottom", "wrap": false,
      "font": { "name": "Calibri", "size": 11, "bold": false, "italic": false, "underline": "None", "color": "#000000" },
      "fill": { "color": null },
      "borders": {                                                         // each side as Graph returns it
        "EdgeLeft":   { "style": "None", "weight": "Thin", "color": "#000000" },
        "EdgeTop": {}, "EdgeRight": {}, "EdgeBottom": {}, "DiagonalUp": {}, "DiagonalDown": {}
      }
    }],
    "charts": [{ "name": "Chart 1", "left": 192, "top": 15, "width": 384, "height": 225 }]   // every chart on the sheet
  }]
}
```

| Field | Graph source | Verified | Note |
|---|---|---|---|
| `columns[].columnWidth` | `range(address='C:C')/format` → `columnWidth`, one column at a time | yes: the property exists | Points (settled 2026-11-10: a column with no `<col>` element, at the default width, reads 48.0, which is 64 px at 96 dpi; Widths!B). A `<col width="8.43" customWidth="1">` reads 44.25 (Widths!H), so the file's width attribute does not map to 48. A hidden column reads 0.0. |
| `columns[].hidden` | the column's range → `columnHidden` | yes | |
| `rows[].rowHeight` | `range(address='5:5')/format` → `rowHeight` | yes: the property exists | Points, as for columns; a hidden row reads 0.0. A row with no custom height reads 15.0 even when it holds larger text that the print PDF shows fitted taller (`fonts-fills-borders` Fonts rows 7:13, 14 to 36 pt): Graph reports the default height, not the web's fitted one (settled 2026-11-10). Fitted heights are tier 2. |
| `rows[].hidden` | the row's range → `rowHidden` | yes | |
| `cells[].text` | `range(address='B2')` → `text[0][0]` | yes | Graph documents that `text` **does not depend on cell width** and is not affected by `#` substitution. So the gate compares it with our engine's display text, not with what we paint. `####` fitting is tier 2. |
| `value`, `valueType`, `numberFormat` | `values`, `valueTypes`, `numberFormat` | yes | Recorded for diagnosis, not compared. |
| `hAlign`, `vAlign`, `wrap` | `…/format` → `horizontalAlignment`, `verticalAlignment`, `wrapText` | yes | Values as Graph names them (`General`, `CenterAcrossSelection`, …). |
| `font.*` | `…/format/font` → `name`, `size`, `bold`, `italic`, `underline`, `color` | yes | `color` is "#RRGGBB". An automatic colour reads "#000000" (settled 2026-11-10). No probe sets an explicit black font, so whether that reads the same is not probed. |
| `fill.color` | `…/format/fill` → `color` | yes | Graph has no fill pattern. Keep the raw value. An unfilled cell reads `""` (empty string), not null (settled 2026-11-10). A solid fill reads its fgColor, but every patterned fill (mediumGray to gray0625, `fonts-fills-borders` Fills rows 3:19, fg FF0000, bg FFFF00) reads its bgColor `#FFFF00`. A `gray125` with automatic colours reads `#FFFFFF` (Fills row 21). |
| `borders.*` | `…/format/borders` → each item's `sideIndex`, `style`, `weight`, `color` | yes, all six sides including both diagonals | Record each side Graph returns, keyed by `sideIndex`. |
| `charts[]` | `worksheet/charts` → `name`, `left`, `top`, `width`, `height` | yes, in points | |
| *(optional)* `refs/graph/<probe>/<chart name>.png` | `chart/image` | yes: the method exists | Not compared in this tier. |
| *(optional)* `refs/graph/<probe>.pdf` | drive item `content?format=pdf` | yes: xlsx is a listed source, under the 2 MB limit | Not compared. Print layout is not the screen. It is the one visual reference Graph gives for tier-2 fields (merges, strikethrough, accounting underlines, `####` fitting), for a person to look at. The download redirects to `*-mediap.svc.ms`, which the network must allow. |

Graph returns `null` for a property that is not uniform across a range. Read one cell, one column or one row at a
time, so null only ever means what the property itself returns. `$batch` (up to 20 requests per call) keeps the
roughly 3,500 cell reads manageable.

The gate refuses a tier-1 dump that lacks a field above or lists more or fewer columns, rows or cells than the probe
range: a dump that skips cells would let the gate compare less than the probe.

## Recording: `rustsheet-oracle render`

```bash
cd tools
uv run rustsheet-oracle render                    # every probe → eval/render/refs/graph/
uv run rustsheet-oracle render --probe charts     # one probe (repeatable)
uv run rustsheet-oracle render --check            # offline: every recorded dump still covers its probe
```

The command refuses a workbook whose sha256 is not its pin, opens it in a session that persists nothing, reads every
column, row and cell of each probe range one at a time through `$batch`, and refuses to write a dump that fails the
completeness rule above. It writes `<probe>.render.json`, `<probe>.pdf` (`--no-pdf` skips it) and
`<probe>/<sheet>_<chart>.png` per chart (`--no-images`). If Excel for the web will not open a probe, the command writes
`<probe>.refused.json` (`rustsheet.excel-render-refused/1`: the Graph error code and message) in place of a dump. That
refusal is the probe's result, and the gate treats the probe as INCOMPLETE. `tools/tests/test_oracle_render.py`
re-checks every committed dump against `probes.json`.

### First recording (2026-11-10)
Five of the six probes recorded: 1,275 cells over 19 sheets, with their print PDFs. **`charts` is refused**:
`createSession` fails with `FileCorruptTryRepair` / `unsupportedWorkbook`, and the PDF conversion refuses it too (406).
Bisected through Graph: the workbook opens with every `<drawing>` removed, and each sheet's drawing alone is enough
to make Excel refuse the file. One chart (`chart1.xml`) alone still makes Excel refuse it. Dropping its
`mc:AlternateContent` makes it open, and so does moving the `xmlns:c14` declaration from `<c14:style>` up to
`<mc:Choice Requires="c14">`. That second change makes all 14 charts open. The chart writer declares the prefix that
`Requires` names on the child element, where it is not in scope for `Requires`. Excel for the web treats that as a
corrupt file. The fix belongs in T1's chart writer, which is not in this repository. T-9000 re-pins and re-records `charts.xlsx`
once T1 supplies a corrected file.

## Tier 2: not available through Graph, so INCOMPLETE until the UI gym (or Excel desktop) records it

None of the following is exposed by the Graph v1.0 resources above, so do not try to record it through Graph:

| Field | Why tier 2 |
|---|---|
| Merged areas | `workbookRange` has `merge`/`unmerge` but nothing that reads merged areas back. |
| Frozen panes, gridlines, headings, zoom, zero display | `workbookWorksheet` has only `name`, `position` and `visibility`. |
| Strikethrough | `workbookRangeFont` has no strikethrough property. |
| Fill pattern and pattern colour | `workbookRangeFill` has only `color`. |
| Indent, shrink to fit, text orientation | `workbookRangeFormat` has none of them in v1.0. |
| Cell geometry (left, top, width, height) and line offsets | `workbookRange` has no position or size. |
| Displayed text after fitting (`####`, shortened General) | `text` does not depend on width, by its own definition. |
| Chart type | `workbookChart` has no chart type property. |
| Desktop's layout (column pixels, fitted row heights) | Graph's layout is Excel for the web's. |

The tier-2 format is `rustsheet.excel-render/1`, the COM geometry dump in the original request. T1 keeps a COM
recorder for it in its repository, as the desktop path. Whether the UI gym can fill every field of that format through
Office.js has not been checked against the Office.js reference here (**unverified**). Its first run settles which
fields it covers.
