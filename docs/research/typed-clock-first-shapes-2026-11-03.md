# Typed clock-first date-times, excel-graph 2026-11-03 (T-3852)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes` for
`eval/suites/probes/cases/date-text-typed-clock-first-shapes.case.json`. Formats are the evidence for docs/excel-semantics/dates.md.

```
ENTERED S!A1 '10 AM 7/1/2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A2 '10:00 AM 7/1/2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A3 '10:00 PM 7/1/2025': values=[[45839.9166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A4 '9:00 7/1/2025': values=[[45839.375]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A5 '10:00:30 7/1/2025': values=[[45839.4170138889]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A6 '10:00:30.5 7/1/2025': values=[[45839.4170196759]] numberFormat=[['mm:ss.0']]
ENTERED S!A7 '10:00.5 7/1/2025': values=[[45839.0069502315]] numberFormat=[['mm:ss.0']]
ENTERED S!A8 '24:00 7/1/2025': values=[[45840]] numberFormat=[['General']]
ENTERED S!A9 '10:60 7/1/2025': values=[[45839.4583333333]] numberFormat=[['General']]
ENTERED S!A10 '10:00  7/1/2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A11 '10:00 July 1, 2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A12 '10:00 1-Jul-2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A13 '10:00 2025-07-01': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A14 '10:00 7/1': values=[[46204.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A15 '10:00 7/1/25': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A16 '23:59 7/1-2025': values=[[45839.9993055556]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A17 '10:00 7-1-2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A18 '10:00 7/1/2025 ': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A19 '2025/7-1': values=[[45839]] numberFormat=[['m/d/yyyy']]
ENTERED S!A20 '1-Jul/25': values=[[45839]] numberFormat=[['d-mmm-yy']]
ENTERED S!A21 '10:00 1/Jul/2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!A22 '10:00 AM 7-1-2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
FAIL /home/user/rustsheet/eval/suites/probes/cases/t0481-curly-quote-names-c.case.json: POST https://graph.microsoft.com/v1.0/sites/musicalretreat.sharepoint.com%2C7553f6ba-abeb-414f-8efe-5d0fc479e66f%2Cad1bbda5-6807-40b8-981b-2c888e7cbe8e/drive/items/01VPIWYNK7OJFSOTH425FITKX24EC7V3I2/workbook/createSession -> 501: {"error":{"code":"FileCorruptTryRepair","message":"We're sorry, but something went wrong with this file.","innerError":{"code":"unsupportedWorkbook","message":"The request failed. The workbook contains unsupported features or exceeds the size limit.","innerError":{"code":"FileCorruptTryRepair","message":"We're sorry, but something went wrong with this file."},"date":"2026-11-03T04:27:23","request-id":"c525f5f5-ff09-4731-8ae4-adf65d1608ce","client-request-id":"c525f5f5-ff09-4731-8ae4-adf65d1608ce
FAIL /home/user/rustsheet/eval/suites/probes/cases/typed-if-one-argument.case.json: POST https://graph.microsoft.com/v1.0/sites/musicalretreat.sharepoint.com%2C7553f6ba-abeb-414f-8efe-5d0fc479e66f%2Cad1bbda5-6807-40b8-981b-2c888e7cbe8e/drive/items/01VPIWYNNHEYM3SO2REZDJUIICXE6O33PO/workbook/createSession -> 501: {"error":{"code":"FileCorruptTryRepair","message":"We're sorry, but something went wrong with this file.","innerError":{"code":"unsupportedWorkbook","message":"The request failed. The workbook contains unsupported features or exceeds the size limit.","innerError":{"code":"FileCorruptTryRepair","message":"We're sorry, but something went wrong with this file."},"date":"2026-11-03T04:27:27","request-id":"1714a932-53c5-45fa-a20a-6ffd92550849","client-request-id":"1714a932-53c5-45fa-a20a-6ffd92550849
```
