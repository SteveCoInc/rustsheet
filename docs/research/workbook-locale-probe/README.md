# Probe request `workbook-locale`: what a regional format and a display language change

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For SPEC-0046 (T-4810) and the
probe tasks T-4812 to T-4816. Nothing waits for it: until an item is answered the engine keeps what it does today,
and the item stays UNVERIFIED.*

## Why a Script Lab request

SPEC-0046 lists 57 things a regional setting may change, and six unknowns that decide its design. The oracle
calculates under one setting, en-US, until T-4809 gives it a target per locale, and four of the unknowns are about
what such a target can observe at all. One script, run by a person under each setting, collects exploratory
observations for the inventory and design unknowns now, with what Office.js can read and Graph cannot
(`numberFormatLocal`, `cultureInfo`, the names Excel gives new things). Which unknowns it can answer, and which
it only supports, is said under "What each unknown's answer looks like".

It does not replace the oracle suites of T-4812 to T-4816: those are the cases that gate. It tells them what to
expect, and which shapes Graph can serve.

## How

- In Script Lab: paste `cases.js` into the Script tab and this one line into the HTML tab:
  `<div id="cases"></div><pre id="out"></pre>`. Press Run and click `RUN`, **once per row of the table below,
  each time in a fresh workbook** with Script Lab opened in it again (the click turns the button off).
  It logs eight JSON lines (`ENV`, `V`, `L`, `T`, `D`, `E5`, `D12`, `F6`), shown under the button to copy from.
  Please return them as
  `results-<run>.jsonl`; they are committed beside this README with a `RESULTS.md`, as
  `docs/research/chart-selection-probe` does. In a workbook whose active sheet is not empty it writes nothing and
  logs `ENV` and `REFUSED`.
- A run is identified by what it logs, not by how it was set: every line carries `env`, which holds the culture name,
  the separators Excel uses and the system's, the date patterns, and Office's display and content languages.
- Excel for the web is the set of record: excel-graph is the only oracle (decision 0266 rule 2), and Script Lab runs
  there. No desktop Excel fills probes (the project lead's ruling of 2026-11-12).
- The script writes about 390 cells one at a time, so that a write Excel refuses is logged as that cell's answer
  and the others still answer. A run takes a few minutes.
- Every item has a block of its own, 4 rows by 4 columns with a gap around it, and each of `T`'s three routes has
  one. A formula that returns an array (an array constant of up to four elements, however the setting reads its
  separators, or `SEQUENCE(2)`) spills inside its block, where nothing else is written. An answer's `spill` is
  the range the array filled, or `null` for one cell. A `#SPILL!` value with no `spill` would mean that an array
  found no room: that is a fault of the script, not an answer about the setting, and not how a refused write
  reads (`error`). Please report it.
- If `env.excelSeparators[2]` is false, or either of its first two entries differs from the corresponding
  `env.systemSeparators` entry, report it. The request assumes system separators are enabled (SPEC-0046
  non-goals and U6).

| Run | Regional format | Display language | What it is for |
|---|---|---|---|
| R1 | English (United States) | English | the control: its `V` and `D` answers must equal the en-US oracle's |
| R2 | German (Germany) | German | de-DE |
| R3 | French (France) | French | fr-FR |
| R4 | Spanish (Spain) | Spanish | es-ES |
| R5 | Italian (Italy) | Italian | it-IT |
| R6 | English (United Kingdom) | English | en-GB |
| R7 | German (Germany) | English | separates the two settings: which one decides each item (U3, U4) |
| R8 | English (United States) | German | the other cross |
| R9 | German (Switzerland) | German | U5, if the setting is what we believe it is (below) |

R1, R2, R7 and R8 come first: they are the control, a locale and the two crosses. R3 to R6 fill the other locales'
rows and can come later. R9 is one more run for one question, U5: see "U5 support only" below.

In Excel for the web the two are the regional format setting and the account's display language. We have not
verified the steps here, which is why the script records what Excel reports. If the web cannot set them apart, R7
and R8 are skipped: please say so with the results. U3 and U4 then stay open, and it is a fact for T-4809, whose
canary asks U2 for the oracle's own session.

## What each group asks

The ids are SPEC-0046's inventory ids, so `V.B1a` evidences item B1.

| Group | What the script does | What is read | Inventory | Unknowns |
|---|---|---|---|---|
| `ENV` | nothing | culture name, Excel's and the system's separators, date patterns, Office's languages | | U6 (support only) |
| `V` | writes 103 formulas in their en-US form (`Range.formulas`) | value, type, displayed text and its code points, `formulasLocal` | B1 to B7, C5, C9, C10, D8 (the three functions), D9, D10, E1 to E4, F1 to F5, F7, G1, G2, G4, G5 | U3, U4 |
| `L` | writes 25 formulas in their en-US form | `formulasLocal`: function, boolean and error names, the argument separator, the decimal comma, array-constant separators | D1 to D6 | U5 (support only) |
| `T` | writes each of 69 strings three ways: `formulasLocal`, `values`, `formulas` | value, type, the stored formula, the number format Excel chose, the range an array filled | A1 to A9, D2, D5 | U1 (support only) |
| `D` | puts 34 stored format codes on stored values | displayed text, its code points, and `numberFormatLocal` | C1, C2, C4, C5, C7, C8, C10, C11, G2; built-in-ID coverage for C3, C6 and C12 is deferred | U4 |
| `E5` | writes 12 format codes through `numberFormatLocal` | the stored code (`numberFormat`) and the displayed text | E5, E4 | U3 |
| `D12` | sets a list validation from `a,b,c` and from `a;b;c`, each on a cell that holds `a` and on one that holds the whole source | the list's source as read back, and `valid` for each cell | D12 | |
| `F6` | adds a sheet and a table | their default names, the header names, a structured reference's `formulasLocal` | F6, D7 | |

T's answers are what Office.js makes of a string by each of its three routes. They are a first reading of A1 to
A9, not their evidence: an A item is about what typing does, and whether a route parses as typing does is U1's
question, which this request does not answer. No one types here, so these results must not be used to mark an A
item verified.

D writes explicit format codes. Its existing C3, C6 and C12 labels do not establish built-in-format behavior: those
items require a fixture with known built-in number-format IDs. These results must not be used to mark C3, C6 or
C12 verified. (The labels say which built-in format a code is the en-US code of. Whether Excel shows a code written
through the API as it shows the id in a file is not asked here.)

D12 records the source as stored and, for two cells, the API's `valid`. An unchanged source does not establish how
many list items Excel recognizes, which is why each list is put on a cell that holds `a` and on one that holds the
whole source: `valid` on the first and not on the second means three items, the reverse means one. This is the
API's reading of a source written through the API. What the dialog makes of a typed list is not asked here.

What each unknown's answer looks like:
- **U3** (do TEXT's letters follow the regional format or the language?): `V.E1b` (`TT.MM.JJJJ`) in R2, R7 and R8.
  Successful crossed runs can resolve it.
- **U4** (untagged month and day names): `V.C5a` and `D.C5a` in R7 and R8. Successful crossed runs can resolve it.
- **U1 support only:** T compares Office.js `formulasLocal`, `values` and `formulas`. It does not test Graph PATCH
  or compare against manual typing. U1 remains unverified until T-4809's target canary establishes the required
  equivalence.
- **U5 support only:** `L.D1a` records the argument separator and `L.D5a` records array separators. These runs do
  not distinguish a regional-list separator from one derived from the decimal separator; U5 needs the
  discriminating setting described in SPEC-0046 or stays UNVERIFIED. R9 is our candidate for that setting. We
  believe, and have not verified, that German (Switzerland) has `.` as its decimal separator and `;` as its list
  separator. If R9's `env.systemSeparators[0]` is `.` and its `L.D1a` reads `=SUMME(1.5;2)`, the argument separator
  is not derived from the decimal separator, and that half of U5 is answered. With any other pair R9 says nothing
  about U5.
- **U6 support only:** ENV records whether system separators are currently enabled and the separators in use. If
  all runs use system separators, availability of an override remains UNVERIFIED. If you know where Excel for
  the web has such a setting, or that it has none, a sentence with the results answers U6.
- **U2** (what decides `formulasLocal`'s language for the oracle's Graph session) is not answered here: it is about
  an app-only session, and T-4809's canary asks it.

What this request leaves to the oracle suites: the full list of function names (508 names in four languages;
shape L of SPEC-0046 through Graph), the built-in number formats a file stores as an id with no code (C3, C6 and
C12: an oracle case is a workbook, and can carry the ids), and R1C1 letters, which no API reads (a person reads
them in the UI).

## Checks, without Excel

- `node check.js` runs the script against a fake of Office.js that knows no locale: the eight lines in order,
  the number of items in each group as the table above gives it, every item written to a cell of its own by its
  group's property, a refused write as that item's answer, a workbook that is not fresh left untouched, and any
  one sync failing without an exception.
- The fake fills, for every formula, the largest array any reading of its separators gives. The check holds that
  no write lands in another item's array, that no array lacks room and that none is larger than its block; that
  the earlier layout (a row an item, `T`'s routes side by side) fails this; and that every function the script
  calls is one whose result the check can bound.
- `cases.js` type-checks against `@types/office-js` 1.0.617 (`tsc --checkJs`).

## What we do with the answers

- Each answered item becomes a rule of `docs/excel-semantics/` (a new `locales.md`, by T-4812) with the run and the
  id as evidence, and SPEC-0046's "Decides" column is corrected where a cross run says otherwise.
- U3 and U4 decide which table of SPEC-0046 Design 4 holds which field. For U1, U5 and U6 the answers are support:
  U1 stays with T-4809's canary, and U5 and U6 stay UNVERIFIED unless R9, or a sentence from the person who ran
  it, answers them as said above.
- An answer that contradicts the spec changes the spec, not the answer. The engine changes only in the task that
  owns the item, with an oracle case of T-4812 to T-4816 behind it.
