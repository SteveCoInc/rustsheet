# cfix-loadvals-value-whitespace (poc-cfix-loadvals-cc, 2026-11-12)

Nine hand-built packages (five first queued, four added 2026-11-12 13h40Z after review of CR-322050446) (`make_probes.py` writes them: `python3 -I make_probes.py OUTDIR`; Application
"RustSheet fixture", every cached formula value a placeholder 0). One sheet, `Probe`. Column A holds a constant
whose `<v>` is written with padding around its value; columns B onward read it with legacy formulas. One question
group per workbook, so a form Excel refuses takes only its own group down: **a refusal (createSession 501
`FileCorruptTryRepair` / `unsupportedWorkbook`) is itself the answer**; record it. excel-graph is enough;
desktop Excel, if anyone has it to hand: does it open silently or offer a repair (M14, decision 0039)?

Why: the 2026-11-12 full corpus run has engine-load constant signatures from files whose writers pad every `<v>`
(`<v> 9</v>`, a trailing space after a number, `<v> #REF!</v>`). The importer read those as 0 and `#ERROR!`; the
fix (branch poc/cfix-loadvals-ws) drops the whitespace around a value, as the importer already did for a
shared-string index (T-4145) and as the harness's snapshot reader does. docs/excel-semantics/xlsx-storage.md
"A cell value's surrounding whitespace is dropped" is UNVERIFIED until this answers. The shared-string workbook
also answers T-4711 (padded index, leading `+`, no-break space).

| File | A1 | A2 | A3 | A4 | A5 | Readers (B, C, D, ...) |
|---|---|---|---|---|---|---|
| value-ws-numbers.xlsx | `<v> 9</v>` | `<v>9 </v>` | `<v>&#9;9&#10;</v>` (tab, LF) | `<v>9</v>` (control) | | `ISNUMBER(A)`, `A+1`, `TYPE(A)`, `ISBLANK(A)`, `A&"\|"` |
| value-ws-number-forms.xlsx | `<v>&#160;9</v>` (no-break space) | `<v>+9</v>` | `<v>9 0</v>` | `<v> </v>` | | same five |
| value-ws-booleans.xlsx (`t="b"`) | `<v> 1 </v>` | `<v>true</v>` | `<v>TRUE</v>` | `<v>1</v>` (control) | | `ISLOGICAL(A)`, `A=TRUE`, `TYPE(A)`, `A&"\|"` |
| value-ws-errors.xlsx (`t="e"`) | `<v> #REF! </v>` | `<v>#REF!</v>` (control) | | | | `ERROR.TYPE(A)`, `ISERROR(A)`, `TYPE(A)` |
| value-ws-shared-index.xlsx (`t="s"`, sst 0 "zero", 1 "one") | `<v> 1</v>` | `<v>1 </v>` | `<v>+1</v>` | `<v>&#160;1</v>` | `<v>1</v>` (control) | `A&"\|"`, `ISTEXT(A)`, `LEN(A)` |
| value-ws-date.xlsx (`t="d"`) | `<v> 2024-01-01 </v>` | `<v>2024-01-01</v>` (control) | | | | `ISNUMBER(A)`, `A+1`, `TYPE(A)` |
| value-ws-only-n.xlsx (no `t`) | `<v> </v>` | | | | | `ISBLANK(A)`, `TYPE(A)`, `ISNUMBER(A)`, `ISERROR(A)`, `A&"\|"` |
| value-ws-only-b.xlsx (`t="b"`) | `<v> </v>` | | | | | same five |
| value-ws-only-e.xlsx (`t="e"`) | `<v> </v>` | | | | | same five |

What the engine gives on the fix branch (every workbook loads):
- numbers: A1:A4 all 9 (B TRUE, C 10, D 1, E FALSE, F `9|`).
- number forms: A1 9 and A2 9 (the code trims Unicode White_Space, the no-break space included, and reads a leading
  `+`); A3 and A4 are 0 (a number cell whose text is no number reads 0: B TRUE, C 1, D 1, E FALSE, F `0|`).
- booleans: A1 TRUE, A2 FALSE, A3 FALSE, A4 TRUE (only `1` is TRUE, xlsx-storage.md "Boolean attributes and boolean
  cells", T-4853); ISLOGICAL TRUE and TYPE 4 for all.
- errors: A1 and A2 `#REF!` (ERROR.TYPE 4, ISERROR TRUE, TYPE 16).
- shared index: A1:A5 all "one" (`one|`, TRUE, 3).
- date: A1 and A2 45292 (ISNUMBER TRUE, A+1 45293, TYPE 1): the date parser trims too.
- whitespace-only: `n` is 0 (ISBLANK FALSE, TYPE 1, ISNUMBER TRUE, `0|`); `b` is FALSE (TYPE 4, `FALSE|`); `e` is
  `#ERROR!` (TYPE 16, ISERROR TRUE). The harness's snapshot reader reads these three as empty, empty and an empty
  error name: Excel's answer decides both readers.
