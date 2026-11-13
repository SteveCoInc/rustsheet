# import-load-probe (poc-cfix-uiperf-cc, 2026-11-12)

Four packages, written by `make_probes.py` (`python3 -I make_probes.py <dir>`). Each has Application "RustSheet
fixture", one sheet named Data, and no cached values. All four parse with `rustsheet-corpus tag-paths`
(outcome ok). The answers decide specs in `docs/ui/specs/import-load/` (branch poc/cfix-uiperf-freeze-docs). None of
them unblocks a CR today, so they are not urgent: send them with the next batch.

excel-graph is enough for P-1 and P-3; it reads row heights and the shown text. P-2 asks first whether Excel opens
the file at all (`createSession`). A 501 FileCorruptTryRepair is itself an answer. Desktop Excel answers the repair
questions, and the round trips if anyone has it to hand.

| Probe | File | What to read |
|---|---|---|
| P-1a | sizes-default-custom.xlsx | `<sheetFormatPr customHeight="1" defaultRowHeight="30"/>`; A2 holds long text in a wrap style, and its row has no `ht`. Read the row height of rows 1, 2 and 1000 (`range.format.rowHeight`). Then write the same long text into A3 with the wrap style and read row 3's height. Desktop: after Save, what does `sheetFormatPr` hold, and does any `<row>` get `ht`? |
| P-1b | sizes-default-plain.xlsx | The same with `defaultRowHeight="30"` and no `customHeight`: the same readings. Does row 2 grow past 30 pt here and not in P-1a? |
| P-2 | merges-overlap.xlsx | Two merges, A1:B2 and B2:C3, overlap at B2. Does it open? If so, read `getMergedAreas` of A1:D4 and the values of A1 and B2. Desktop: does it offer a repair, and what does the repair log say? |
| P-3 | numfmt-col-row.xlsx | Column B has style 1 (`0.00`). Row 3 is `s="2" customFormat="1"` (`0%`). B5 is a `<c>` of its own with style 0 (General). B6, B3 and D3 have no `<c>`. Write the number 1.5 into B6, B5, B3 and D3, and read each one's `text` and `numberFormat`. B3 is where the row style and the column style cross: which wins? |

What we expect, from recall only (not evidence):
- P-1a: every row 30 pt, and no growth on open or on edit.
- P-1b: row 2 auto-fits on open in desktop.
- P-2: a repair that keeps one of the two merges.
- P-3: B6 shows 1.50, B5 shows 1.5, D3 shows 150%, and B3 shows 150% (the row's style wins).

Engine and UI today:
- **P-1.** The import keeps `defaultRowHeight`. `sheetFormat` carries no custom flag, so the UI treats the default as
  not custom (D-UI-60).
- **P-2.** The model shows the latest of the overlapping merges.
- **P-3.** The engine reports row and column number formats as skipped (`rowColumnNumberFormats`, Taskei P561288130).
