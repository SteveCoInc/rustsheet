# Probe request `undeclared-entity`: what Excel does with an undeclared XML entity in a part of a workbook

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For Taskei P558994790.
The importer refuses a workbook when any part it reads holds an entity reference other than the five predefined
ones (T-5251, T-5275; `docs/excel-semantics/xlsx-storage.md` records it as a resource limit, with no Excel
evidence). A reference such as `&nbsp;` with no declaration makes a part not well-formed XML; Excel for the web
may refuse the file or open it. This request asks which, part by part. Every route here is Excel for the web:
no desktop Excel fills probes (decision 0464).*

## How

- `python3 -I generate.py <dir>` writes eight workbooks: a control, and seven copies of it that each hold one
  `&nbsp;` in one part. The theme and the application properties are those of an Excel-written workbook of the
  repository (`crates/rustsheet-xlsx/tests/calc_tests/DATE_AND_TIME/DATEVALUE.xlsx`), so that the control is a
  package Excel opens.
- For the control, first, and then the shared-strings, styles, theme, application-properties and
  core-properties cases, with `<dir>` an absolute path (the command runs in `tools/`): `cd tools && uv run
  rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err`. The
  tool's canary writer and its other package reads accept these six (checked here, without the network).
  Please return, for each, the JSON output, or the `.err` file if the backend refused the workbook (a refusal
  such as `FileCorruptTryRepair` is an answer, beside an empty JSON), and, for a file that opens, the
  recalculated workbook, which the tool keeps as `eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the file's
  sha256 below: what Excel wrote back for the part says what it kept. If the `.err` file says `the oracle's
  answer cannot be read` or `canary failed`, Excel opened the file and the tool has deleted its copy of the
  answer: that message is then the answer. If Excel refuses the control, stop there: the package is then the
  suspect.
- **The two worksheet cases cannot go through the tool.** Its canary writer reads every worksheet, and refuses
  them itself: "cannot embed the canary: unreadable xl/worksheets/sheet1.xml (undefined entity …)". Please open
  `entity-sheet-text.xlsx` and `entity-sheet-attribute.xlsx` in Excel for the web by hand (upload, then open),
  and note what happens: an error and its text, or the workbook, with A1, C1, D1 and E1 as shown. E1 tells on
  screen what a downloaded copy would: U+00A0 and a space look alike in A1.
- `python3 -I check.py` runs the generator and reads the workbooks back: the control is well formed and holds
  its marker values, each case differs from it in one part and by one `&nbsp;`, that part is no longer
  well-formed XML, the two parts taken from the Excel-written workbook are the ones these files were made from,
  and the sha256 values below are the files'. It needs no Excel.

## The cases

Every workbook holds one sheet. A1 is the text `a b`, written in the sheet, and A2 the text `c d`, held in the
shared strings. B1 is 1 and C1 `=B1+1`. D1 and D2 are the lengths of A1 and A2, and E1 and E2 the code of
their second character, `=UNICODE(MID(A1,2,1))`. Row 1 is 30 points high (`ht="30" customHeight="1"`), the one
font is Courier New 13, the theme's colour scheme is named `Probe` with accent 1 `123456`, and the core
properties hold the title `Probe title`. None of the first three is a value Excel gives a workbook by default,
so what Excel writes back says whether it read the part.

| File (sha256) | Where the entity is | Questions |
|---|---|---|
| `entity-control.xlsx` (sha256 `2991c9d522cb7d7fe439b21935ad5fb3b4ceca646629a9733853cbcba2f421ec`) | nowhere | the control: C1 = 2, D1 = 3, E1 = 32, D2 = 3, E2 = 32 |
| `entity-sheet-text.xlsx` (sha256 `9f564ff08403585df5d08062275a9f185adeecae35c948b8079541b27a8aad45`) | the worksheet, in a cell's text: `<t>a&nbsp;b</t>` | Does Excel open it? What is A1, D1 (3 if the entity became one character, 2 if it was dropped) and E1 (160 if that character is U+00A0, 32 if a space, 98 if the entity was dropped)? |
| `entity-sheet-attribute.xlsx` (sha256 `ec33403bd8f00343ede59ea5c26f884578db3dbd89a466f2fab810f45e6a64e3`) | the worksheet, in a row's attribute: `ht="30&nbsp;"` | Does Excel open it? With which height for row 1 (30 if the value was read without the entity; 15, or whatever Excel gives a row with no height, if the attribute was dropped)? |
| `entity-shared-strings.xlsx` (sha256 `219b7de97b26757cdd697d22873db8ebb0afb7192ef53b8b57eda1acc3cfbb1b`) | the shared strings, where most writers keep cell text: `<t>c&nbsp;d</t>` | Does Excel open it? What is A2, D2 (3 or 2) and E2 (160, 32, or 100 if the entity was dropped)? |
| `entity-styles.xlsx` (sha256 `e0681d70856dd96d31bb8b3655a454084b223a223902343b47870d2691115ac1`) | the styles, a part every workbook needs: `<name val="Courier New&nbsp;"/>` | Does Excel open it? With which font in the styles it writes back (Courier New 13 if the entity alone was dropped; another name or size if the attribute, the font or the part was)? |
| `entity-theme.xlsx` (sha256 `9881de8a6848f0e110418442b9bfe0a375475ad74e1f53a7f3e60c473ebb63b6`) | the theme, a part a reader can do without: `<a:clrScheme name="Probe&nbsp;">` | Does Excel open it? In the theme it writes back, is accent 1 `123456` (this theme) or not (Excel's own)? |
| `entity-app.xlsx` (sha256 `87d83a0a3de7bcc563eaaf56de497648cc5160847de29afba12cca54ff573af7`) | the application properties, which no cell depends on: `<Application>Microsoft&nbsp;Excel` | Does Excel open it? |
| `entity-core.xlsx` (sha256 `46102cb918e8ed606be4b62e04794c627cafc34244a216c27c3860cb4f318edb`) | the core properties, where a title is: `<dc:title>Probe&nbsp;title` | Does Excel open it? With which title in the properties it writes back? |

## What the engine does today

Measured on main cdb95102e by loading each workbook through the importer and calculating it: the control loads
(A1 `a b`, C1 2, D1 3, E1 32, A2 `c d`, D2 3, E2 32), and each of the seven cases is refused with the entity
limit (`Limit { limit: EntityExpansion, max: 0 }`), whichever part holds the reference. That is the entry's
rule: a limit refuses the workbook even in the parts whose unreadability would not be fatal.

## What the answers decide

Each answer goes into `xlsx-storage.md` under the entry "Import resource limits reject a workbook outright", which
holds the entity rule. Where Excel for the web refuses a file, the entry records that refusal as the oracle of
record's (decision 0039), and the importer's refusal matches it. `FileCorruptTryRepair` names a repair that only
desktop Excel offers; what that repair keeps of the file is asked of no oracle (decision 0464), so the entry says so
and does not call the refusal desktop Excel's. Where Excel opens one, the entry gains what Excel shows, and the xlsx
lane decides whether the importer should read that part as Excel does or keep the refusal as a recorded deviation: a
file with a bare `&nbsp;` is usually a sloppy writer's, not an attack, and refusing the whole workbook for its theme
or its properties is a high price.
