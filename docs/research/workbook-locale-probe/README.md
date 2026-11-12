# Probe request `workbook-locale`: what a regional format and a display language change

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For SPEC-0046 (T-4810) and the
probe tasks T-4812 to T-4816. Nothing waits for it: until an item is answered the engine keeps what it does today,
and the item stays UNVERIFIED.*

## Why a Script Lab request

SPEC-0046 lists 52 things a regional setting may change, and six unknowns that decide its design. The oracle
calculates under one setting, en-US, until T-4809 gives it a target per locale, and four of the unknowns are about
what such a target can observe at all. One script, run by a person under each setting, answers the unknowns and
most of the inventory now, with what Office.js can read and Graph cannot (`numberFormatLocal`, `cultureInfo`, the
names Excel gives new things).

It does not replace the oracle suites of T-4812 to T-4816: those are the cases that gate. It tells them what to
expect, and which shapes Graph can serve.

## How

- Run `cases.js` in Script Lab, `await run()`, **once per row of the table below, each time in a fresh workbook**.
  It logs seven JSON lines (`ENV`, `V`, `L`, `T`, `D`, `E5`, `F6`). Please return them as `results-<run>.jsonl`; they
  are committed beside this README with a `RESULTS.md`, as `docs/research/chart-selection-probe` does.
- A run is identified by what it logs, not by how it was set: every line carries `env`, which holds the culture name,
  the separators Excel uses and the system's, the date patterns, and Office's display and content languages.
- Excel for the web is the set of record: excel-graph is the only oracle (decision 0266 rule 2), and Script Lab runs
  there. A run on Excel 365 desktop is welcome as a second set, labelled as such; it is evidence only from a listed
  oracle operator who signs it (decision 0427), and otherwise a hint.
- The script writes about 370 cells one at a time, so that a write Excel refuses is logged as that cell's answer
  and the others still answer. A run takes a few minutes.
- If Excel's own separators are overridden (`env.excelSeparators` differs from `env.systemSeparators`), say so:
  the request assumes they are the system's (SPEC-0046, non-goals, and unknown U6).

| Run | Regional format | Display language | What it is for |
|---|---|---|---|
| R1 | English (United States) | English | the control: its `V` and `D` answers must equal the en-US oracle's |
| R2 | German (Germany) | German | de-DE |
| R3 | French (France) | French | fr-FR |
| R4 | Spanish (Spain) | Spanish | es-ES |
| R5 | Italian (Italy) | Italian | it-IT |
| R6 | English (United Kingdom) | English | en-GB |
| R7 | German (Germany) | English | separates the two settings: which one decides each item (U3, U4, U5) |
| R8 | English (United States) | German | the other cross |

R1, R2, R7 and R8 answer the unknowns; R3 to R6 fill the other locales' rows and can come later.

How the two are set differs by host (the web's regional format setting and the account's language; on the desktop,
Windows's region and Office's display language). We have not verified the steps here, which is why the script
records what Excel reports. If a host cannot set them apart, R7 and R8 are skipped, and that is an answer to U2.

## What each group asks

The ids are SPEC-0046's inventory ids, so `V.B1a` evidences item B1.

| Group | What the script does | What is read | Inventory | Unknowns |
|---|---|---|---|---|
| `ENV` | nothing | culture name, Excel's and the system's separators, date patterns, Office's languages | | U6 |
| `V` | writes 103 formulas in their en-US form (`Range.formulas`) | value, type, displayed text and its code points, `formulasLocal` | B1 to B7, C5, C9, C10, D8 (the three functions), D9, D10, E1 to E4, F1 to F5, F7, G1, G2, G4, G5 | U3, U4 |
| `L` | writes 25 formulas in their en-US form | `formulasLocal`: function, boolean and error names, the argument separator, the decimal comma, array-constant separators | D1 to D6 | U5 |
| `T` | writes each of 69 strings three ways: `formulasLocal`, `values`, `formulas` | value, type, the stored formula, the number format Excel chose | A1 to A9, D2, D5 | U1 |
| `D` | puts 34 stored format codes on stored values | displayed text, its code points, and `numberFormatLocal` | C1 to C8, C11, C12, G2 | U4 |
| `E5` | writes 12 format codes through `numberFormatLocal` | the stored code (`numberFormat`) and the displayed text | E5, E4 | U3 |
| `D12` | sets a list validation from `a,b,c` and from `a;b;c` | the list's source as read back | D12 | |
| `F6` | adds a sheet and a table | their default names, the header names, a structured reference's `formulasLocal` | F6, D7 | |

What each unknown's answer looks like:
- **U1** (is `formulasLocal` typing, and is `values` not?): in R2, `T["1,5"].formulasLocal` is the number 1.5 and
  `T["1,5"].values` is text, or not.
- **U3** (do TEXT's letters follow the regional format or the language?): `V.E1b` (`TT.MM.JJJJ`) in R2, R7 and R8.
- **U4** (untagged month and day names): `V.C5a` and `D.C5a` in R7 and R8.
- **U5** (is the argument separator the regional list separator?): `L.D1a`'s `formulaLocal` in R7 and R8, and
  `L.D5a` for the array separators in every run.
- **U6** (the separator override): `env.excelSeparators` against `env.systemSeparators`.
- **U2** (what decides `formulasLocal`'s language for the oracle's Graph session) is not answered here: it is about
  an app-only session, and T-4809's canary asks it.

What this request leaves to the oracle suites: the full list of function names (508 names in four languages;
shape L of SPEC-0046 through Graph), and R1C1 letters, which no API reads (a person reads them in the UI).

## What we do with the answers

- Each answered item becomes a rule of `docs/excel-semantics/` (a new `locales.md`, by T-4812) with the run and the
  id as evidence, and SPEC-0046's "Decides" column is corrected where a cross run says otherwise.
- U1 decides how T-4811 sends typed entries; U3 to U5 decide which table of SPEC-0046 Design 4 holds which field.
- An answer that contradicts the spec changes the spec, not the answer. The engine changes only in the task that
  owns the item, with an oracle case of T-4812 to T-4816 behind it.
