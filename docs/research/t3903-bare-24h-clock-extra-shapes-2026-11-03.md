# Bare typed 24-hour clock, extra shapes, excel-graph 2026-11-03 (T-3903)

The `ENTERED` stderr log of `rustsheet-oracle fill -b excel-graph --suite probes --case
probes/t3903-bare-24h-clock-extra.case.json` (A1:A21; value and number format of the PATCH response).

| Typed | Value | Format |
|---|---|---|
| `25:00:30` | 1.04201388888889 | `[h]:mm:ss` |
| `100:00:00`, `24:00:00`, `25:00:00`, `48:00:00`, `99:59:59`, `24:30:15`, `24:01` | numbers | `[h]:mm:ss` |
| `100:00:00.5`, `30:00:30.5` | numbers | `mm:ss.0` |
| `25:60`, `24:60`, `25:00:60`, `25:61:00` | text | `General` |
| `10:60:00`, `10:61`, `10:99`, `0:00:75`, `10:00:99`, `1:100`, `0:00:100` | numbers (minutes/seconds carry over) | `General` |
