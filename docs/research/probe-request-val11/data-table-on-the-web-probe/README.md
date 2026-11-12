# Probe request `data-table-on-the-web`: what a person sees of a what-if data table in Excel for the web

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). For SPEC-0051 (what-if
data tables in the collaborative sheet; T-5640, Taskei P561294192; in review as CR-322164526 when this was
written), its Design 8 and one rule of its Design 5. Nothing here has been run in Excel. It is five minutes by
hand, with one workbook.*

## What is asked

The collaborative sheet will keep an imported data table a data table, and refuse the edits Excel refuses. What
Excel for the web computes and which edits it refuses are known: `docs/excel-semantics/recalc.md` has them from
excel-graph (the Graph API), which answers a refused edit with a 500 `GeneralException`. Three things a person
sees are not known, because Graph does not show them:

1. what the formula bar shows for a result cell;
2. what Excel for the web tells a person whose edit it refuses (the words, or that it says nothing);
3. whether undo brings a table back after its results were cleared.

Microsoft's [Excel for the web service description, “What if analysis tools”](https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online#what-if-analysis-tools)
says that the web app can display what-if results, but using Data Tables and Goal Seek requires the desktop app.
Step 6 checks whether the current web UI offers such tools.

## The workbook

`data-tables.xlsx`: the workbook `rustsheet-spec2xlsx` builds from
`eval/suites/probes/cases/data-tables.case.json`, the case excel-graph has already calculated (20 of 20 cells).
From the repository's `tools` directory:

    uv run rustsheet-spec2xlsx -o data-tables.xlsx ../eval/suites/probes/cases/data-tables.case.json

The builder fixes the package creation time at 2000-01-01; time alone does not change the bytes. When sending a
generated copy, include a `SHA256SUMS` file identifying it. The build of 2026-11-12 has sha256
`88bd92dfbb11c74e4d2ccebc646b98222c5583134d63062eca4e8cc2efcfad63`, which is the input excel-graph calculated
(`input_sha256` in `eval/suites/probes/cases/data-tables.oracle.json`). Its one sheet, `S`, holds four tables.
The steps use two of them:

| Table | Results | Its formula | Its values | Its input cell |
|---|---|---|---|---|
| a column table | B4:B6 | B3, `=A1*10+B1` | A4:A6 = 1, 2, 3 | A1 (1); B1 is 2 |
| a two-variable table | B11:D12 | A10, `=A1*10+B1` | B10:D10 = 100, 200, 300 and A11:A12 = 1, 2 | row input B1, column input A1 |

The file stores 0 in every result cell and asks Excel to recalculate everything on load (`fullCalcOnLoad`), so
what the cells show after opening is Excel's own calculation.

## How

In Excel for the web, opened for editing. Please note the date, the browser and the build Excel reports
(File > About), and write a sentence for each step; a screenshot where a message appears is the best evidence.

| Step | Do | Write down |
|---|---|---|
| 1 | Open the workbook. | What B4:B6 show (we expect 12, 22, 32) and H1 (66). If they show 0, say so and stop: the table was not calculated, and that is the answer. |
| 2 | Click B5. Then click C12. | The formula bar's content for each, character for character, or that it is empty. (`FORMULATEXT` of B5 is `{=TABLE(,A1)}` on excel-graph; by the same rule C12's is `{=TABLE(B1,A1)}`. The bar itself is not known.) |
| 3 | With B5 selected, type 99 and press Enter. | What happens: the exact words of any message, where it appears (a dialog, a bar, a tip), or that nothing is shown. What B5 holds afterwards. |
| 4 | Select B5 alone and press Delete. Then right-click the header of row 5 and insert a row. | The same for each of the two. If the row was inserted, say what B4:B7 show, and undo it. |
| 5 | Select B4:B6 and press Delete. Then press Ctrl+Z once. Then type 5 into B1 and press Enter. | After Delete: are the three cells empty, and what does H1 show? After Ctrl+Z: what do B4:B6 show, and what does the formula bar show on B5? After B1 = 5: do B4:B6 become 15, 25, 35 (the table is back) or stay as they were (only its values came back)? |
| 6 | Look in the Data tab and in the search box for "data table", "what-if" and "goal seek". | Whether any command to make a data table, Goal Seek or a scenario is offered. |

**Return** the notes as `RESULTS.md` beside this file, with the screenshots. If the workbook is saved by Excel
after step 5 (it saves by itself), a downloaded copy of it is welcome too: it shows what Excel stored for the
table that undo brought back.

## What the answers decide

| # | Question | Read from | What it decides in SPEC-0051 |
|---|---|---|---|
| 1 | What does the formula bar show for a result cell? | step 2 | Design 8: the bar shows what Excel shows. If Excel shows nothing, so does the sheet, and the worker need not hand the text out. |
| 2 | What is a person told when an edit is refused? | steps 3 and 4 | the sentence the sheet shows for a refused edit (Design 5), and whether a refusal is a message or a silent no-op. Until answered, the sentence starts from desktop Excel's "Cannot change part of a data table." |
| 3 | Is an insert inside the table refused for a person as it is through Graph? | step 4 | nothing new if it is. If the web's own UI allows it, recalc.md's rule (VERIFIED through Graph) is CONFLICTING for that edit and Design 5's row waits. |
| 4 | Does undo bring back the table, or only its values? | step 5 | Design 5: match the observed undo behavior. If Excel restores only values, revise the proposed record-restoration rule; any existing mismatch is a known gap with a follow-up task, not an accepted product choice (decision 0052). |
| 5 | Is there a tool to make one? | step 6 | the non-goal. Microsoft documents that there is none; a tool found here would reopen it. |

Work on the already-evidenced parts of SPEC-0051 need not wait for this request. The formula-bar presentation,
refusal feedback and undo behavior remain unverified until these steps are recorded.
