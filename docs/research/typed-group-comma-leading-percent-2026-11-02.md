# Typed group-comma and leading-percent texts, excel-graph 2026-11-02 (T-3835)

Formats read from the styles of the recalculated workbook the oracle cached for
`probes/typed-group-comma-leading-percent` (the `ENTERED` stderr log of the fill run was cut off by
`tail`; the cell styles are the same data). Value from the oracle sidecar; format from the cell style.
Text left as text keeps General.

| Typed | Value | Number format |
|---|---|---|
| `1,0000` | 10000 | `#,##0` |
| `%1` | 0.01 | `0%` |
| `%-1` | -0.01 | `0%` |
| `1,` | text | General |
| `0,000` | text | General |
| `% 1` | 0.01 | `0%` |
| `%1.5` | 0.015 | `0.00%` |
| `%1,0000` | 100 | `0%` |
| `1,0000%` | 100 | `0%` |
| `%1%` | text | General |
| `1,000,0000` | 10000000 | `#,##0` |
| `0,0000` | text | General |
| `01,0000` | 10000 | `#,##0` |
| `1,00` | text | General |
| `-1,0000` | -10000 | `#,##0` |
| `1,0000.5` | 10000.5 | `#,##0.00` |
