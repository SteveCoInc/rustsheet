# Typed currency before parentheses, excel-graph 2026-11-04 (T-4117)

Raw `ENTERED` stderr lines of the `rustsheet-oracle fill -b excel-graph --suite probes` run for
probes/t4117-typed-currency-paren: each text typed into Sheet1!A by a range `values` PATCH; the value and number
format are those of the PATCH response. Not scored.

```
ENTERED Sheet1!A1 '$(12)': values=[[-12]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A2 '$(1,234.5)': values=[[-1234.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A3 '$ (12)': values=[[-12]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A4 '($12)': values=[[-12]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A5 '(12$)': values=[['(12$)']] numberFormat=[['General']]
ENTERED Sheet1!A6 '$(12': values=[['$(12']] numberFormat=[['General']]
ENTERED Sheet1!A7 '$12)': values=[['$12)']] numberFormat=[['General']]
ENTERED Sheet1!A8 '$(-12)': values=[['$(-12)']] numberFormat=[['General']]
ENTERED Sheet1!A9 '-$(12)': values=[['-$(12)']] numberFormat=[['General']]
ENTERED Sheet1!A10 '$(12)%': values=[['$(12)%']] numberFormat=[['General']]
ENTERED Sheet1!A11 '$(0)': values=[[0]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A12 '$((12))': values=[['$((12))']] numberFormat=[['General']]
ENTERED Sheet1!A13 '(12)': values=[[-12]] numberFormat=[['General']]
ENTERED Sheet1!A14 '$(1,23)': values=[['$(1,23)']] numberFormat=[['General']]
ENTERED Sheet1!A15 '$(12.5)': values=[[-12.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A16 '$( 12 )': values=[[-12]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A17 '$(1e3)': values=[[-1000]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A18 '€(12)': values=[['€(12)']] numberFormat=[['General']]
ENTERED Sheet1!A19 '$12': values=[[12]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A20 '$(12.0)': values=[[-12]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A21 '$(1,234)': values=[[-1234]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A22 '$(1.5)': values=[[-1.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A23 '$(.5)': values=[[-0.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A24 '$(12.)': values=[[-12]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
```
