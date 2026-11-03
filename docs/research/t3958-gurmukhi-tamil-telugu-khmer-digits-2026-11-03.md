# Typed Khmer digit texts, excel-graph 2026-11-03 (T-3958)

Raw `ENTERED` stderr lines of the first `rustsheet-oracle fill -b excel-graph --suite probes` run that filled
probes/t3958-gurmukhi-tamil-telugu-khmer-digits. Only the last six lines (Sheet1!A39:A44, Khmer) were still on
screen when the run ended; the other 38 lines (Gurmukhi, Tamil, Telugu and Khmer A34:A38) were not kept, and a
re-fill does not print them again. Those formats are UNVERIFIED (re-probe: T-3891).

```
ENTERED Sheet1!A39 '(១២)': values=[[-12]] numberFormat=[['[$-km-KH,1200]0']]
ENTERED Sheet1!A40 '១,២៣៤': values=[[1234]] numberFormat=[['[$-km-KH,1200]#,##0']]
ENTERED Sheet1!A41 '១e៣': values=[[1000]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A42 '២០២០-១-១': values=[[43831]] numberFormat=[['[$-,1200]mm/dd/yyyy']]
ENTERED Sheet1!A43 '១២:៣០': values=[[0.520833333333333]] numberFormat=[['[$-,1200]h:mm']]
ENTERED Sheet1!A44 '១ ១/២': values=[[1.5]] numberFormat=[['# ?/?']]
```
