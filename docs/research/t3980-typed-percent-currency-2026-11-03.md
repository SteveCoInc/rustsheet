# Typed percent and currency shapes: ENTERED log (T-3980)

Source: `rustsheet-oracle fill -b excel-graph --suite probes`, 2026-11-03, probe `probes/t3980-typed-percent-currency`
(the values and ISNUMBER are in its oracle sidecar; the number format is only in the fill's ENTERED log, copied here).

| cell | typed | value | numberFormat |
|---|---|---|---|
| A1 | `-%1.5` | -0.015 | `0.00%` |
| A2 | `-%-1.5` | text `-%-1.5` | General |
| A3 | `-% 1.5` | -0.015 | `0.00%` |
| A4 | `1.5$` | text | General |
| A5 | `1.5 $` | text | General |
| A6 | `-200$` | text | General |
