# Typed clock-first date-times with an `A.`/`P.` word, excel-graph 2026-11-03 (T-3907)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes` for
`eval/suites/probes/cases/t3907-typed-clock-a-period-shapes.case.json` (written first, used to decode the rule; rows 11
and 22 refilled with year-independent formulas, values unchanged) and
`eval/suites/probes/cases/t3907-typed-clock-a-period-edges.case.json` (written after the rule was decoded, as a fresh
sweep). Formats are the evidence for docs/excel-semantics/dates.md "Typed clock-first date-times".

## Shapes

```
ENTERED S!A1 '10 A. 7/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A2 '1 A. 7/1/2025': values=[[95368.2700775463]] numberFormat=[['mm:ss.0']]
ENTERED S!A3 '10 P. 7/1/2025': values=[[95368.2763391204]] numberFormat=[['mm:ss.0']]
ENTERED S!A4 '10 A. 1/1/1900': values=[[49711.2763206019]] numberFormat=[['mm:ss.0']]
ENTERED S!A5 '10 a.m. 7/1/2025': values=[['10 a.m. 7/1/2025']] numberFormat=[['General']]
ENTERED S!A6 '10 A.': values=[[0.416666666666667]] numberFormat=[['h:mm AM/PM']]
ENTERED S!A7 'A. 7/1/2025': values=[['A. 7/1/2025']] numberFormat=[['General']]
ENTERED S!A8 '10 A. 8/1/2025': values=[[95368.2763287037]] numberFormat=[['mm:ss.0']]
ENTERED S!A9 '10 A. 7/2/2025': values=[[95399.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A10 '10 A. 7/1/2026': values=[[95733.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A12 '10 A. 7/1': values=[['10 A. 7/1']] numberFormat=[['General']]
ENTERED S!A13 '10 A.7/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A14 '10 B. 7/1/2025': values=[['10 B. 7/1/2025']] numberFormat=[['General']]
ENTERED S!A15 '10 A. 1/1/2000': values=[[86236.2763206019]] numberFormat=[['mm:ss.0']]
ENTERED S!A16 '0 A. 7/1/2025': values=[[95368.2693831018]] numberFormat=[['mm:ss.0']]
ENTERED S!A17 '10 A. 5': values=[[49710.2763252315]] numberFormat=[['mm:ss.0']]
ENTERED S!A18 '10 A. 5/1': values=[['10 A. 5/1']] numberFormat=[['General']]
ENTERED S!A19 '20 A. 7/1/2025': values=[[95368.2832719907]] numberFormat=[['mm:ss.0']]
ENTERED S!A20 '10:30 A. 7/1/2025': values=[[95368.7068831018]] numberFormat=[['mm:ss.0']]
ENTERED S!A21 '10 A. 7-1-2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A23 '10 A. July 1, 2025': values=[['10 A. July 1, 2025']] numberFormat=[['General']]
ENTERED S!A24 '10 A 7/1/2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A25 '10 A. 17/1/2025': values=[[95368.276321412]] numberFormat=[['mm:ss.0']]
ENTERED S!A26 '10:00:30 A. 7/1/2025': values=[['10:00:30 A. 7/1/2025']] numberFormat=[['General']]
ENTERED S!A11 '10 A. 7/1/25': values=[[95757.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A22 '10 A. 2025-07-01': values=[[95914.276321794]] numberFormat=[['mm:ss.0']]
```

## Edges

```
ENTERED S!A1 '10 a. 7/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A2 '10 p. 7/1/2025': values=[[95368.2763391204]] numberFormat=[['mm:ss.0']]
ENTERED S!A3 '10:00 P. 7/1/2025': values=[[95368.6860613426]] numberFormat=[['mm:ss.0']]
ENTERED S!A4 '10 A. 70/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A5 '10 A. 07/1/2025': values=[[95368.2763202546]] numberFormat=[['mm:ss.0']]
ENTERED S!A6 '10 A. 7/13/2025': values=[['10 A. 7/13/2025']] numberFormat=[['General']]
ENTERED S!A7 '10 A.  7/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A8 '10  A. 7/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A9 '10 A. 7/1/2025 ': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A10 '99 A. 7/1/2025': values=[['99 A. 7/1/2025']] numberFormat=[['mmm-yy']]
ENTERED S!A11 '10 A.. 7/1/2025': values=[['10 A.. 7/1/2025']] numberFormat=[['General']]
ENTERED S!A12 '10 A. 7': values=[[49710.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A13 '10 A. 123456789/1/2025': values=[[95368.2763208681]] numberFormat=[['mm:ss.0']]
ENTERED S!A14 '10 A. 1234567890/1/2025': values=[[95368.2763208681]] numberFormat=[['mm:ss.0']]
ENTERED S!A15 '10 A. 7 1/1/2025': values=[[95368.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A16 '10 A. 7-2025': values=[['10 A. 7-2025']] numberFormat=[['General']]
ENTERED S!A17 '10 A. 7/Jul/2025': values=[[95549.2763275463]] numberFormat=[['mm:ss.0']]
ENTERED S!A18 '10 A. 7/1/2025 10:00': values=[['10 A. 7/1/2025 10:00']] numberFormat=[['General']]
ENTERED S!A19 '24 A. 7/1/2025': values=[[95368.2860497685]] numberFormat=[['mm:ss.0']]
ENTERED S!A20 '10 PM. 7/1/2025': values=[[95368.2763391204]] numberFormat=[['mm:ss.0']]
ENTERED S!A21 '10 A. 7/0/2025': values=[['10 A. 7/0/2025']] numberFormat=[['General']]
ENTERED S!A22 '10 A. 7/1/1899': values=[['10 A. 7/1/1899']] numberFormat=[['General']]
ENTERED S!A23 '60 A. 7/1/2025': values=[['60 A. 7/1/2025']] numberFormat=[['mmm-yy']]
ENTERED S!A24 '1:2 A. 7/1/2025': values=[[95368.3124386574]] numberFormat=[['mm:ss.0']]
```
