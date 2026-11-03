# Typed-input number formats for glued month-name text with separators, excel-graph 2026-11-03 (T-3299)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes --case
probes/t3299-date-text-glued-month-3.case.json`, the run that wrote
`eval/suites/probes/cases/t3299-date-text-glued-month-3.oracle.json`. Each line is the Graph range PATCH
response after writing the text with `values`: the value and the `numberFormat` Excel for the web
picked. The sidecar scores the values (Sheet1!O:R); these formats are the evidence for
docs/excel-semantics/dates.md ("Date text with a month name glued to digits"). A first run with the
year 2026 (the oracle's clock year) gave the same formats (`1-Jul2026`, `1-Jul26`, `1/Jul2026`,
`1-JULY2026`, `1/Jul26` -> 46204 `d-mmm-yy`; `1Jul5` -> 38534 `d-mmm-yy`; the other four stay
text, General); the case was moved to 2024 so no scored cell depends on the clock.

```
ENTERED Sheet1!N1 '1-Jul2024': values=[[45474]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N2 '1-Jul24': values=[[45474]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N3 '1Jul5': values=[[38534]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N4 '1/Jul2024': values=[[45474]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N5 '1.Jul2024': values=[['1.Jul2024']] numberFormat=[['General']]
ENTERED Sheet1!N6 '1,Jul2024': values=[['1,Jul2024']] numberFormat=[['General']]
ENTERED Sheet1!N7 '1Jul1 , 2024': values=[['1Jul1 , 2024']] numberFormat=[['General']]
ENTERED Sheet1!N8 'July1 , 2024 5': values=[['July1 , 2024 5']] numberFormat=[['General']]
ENTERED Sheet1!N9 '1-JULY2024': values=[[45474]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N10 '1/Jul24': values=[[45474]] numberFormat=[['d-mmm-yy']]
```
