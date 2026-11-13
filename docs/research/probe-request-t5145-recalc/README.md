# Probe request `t5145-recalc`: Manual calculation mode, and what Excel calls a precedent and a dependent

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For T-5145, the SPEC-0002
amendments: its designs for calculation modes (T-5114) and for `precedents` and `dependents` (T-5115) will carry rules
marked provisional until these cases are answered. A provisional rule is what Microsoft documents where it documents
anything (the research note beside this directory, sections S1 to S3), and says so. The implementation tasks can
start before the answers: each provisional rule is one hook.*

## Why these cannot be static cases

A case of `eval/suites/probes` is a workbook and its values after one full recalculation. The oracle tool always
forces `fullCalcOnLoad` and sends `calculationType: FullRebuild` (`tools/src/rustsheet_tools/oracle/backends.py`),
and the case schema has sheet edits (`sheet_ops`) but no read between steps and no calculation mode. These questions are about a sequence (set the
mode, edit, read, calculate, read) or about an API that returns ranges, so they need Office.js (Script Lab), in Excel for the web or on the desktop.

## How

- **In Script Lab:** paste `cases.js` into the Script tab and this one line into the HTML tab:
  `<div id="cases"></div><pre id="out"></pre>`. Press Run: the script puts one button a case on the page.
  Click one button only: it runs that case in the workbook that is open, which must be a fresh one, and the
  next case needs a new workbook with Script Lab opened in it again. The case's JSON line is logged, and
  shown under the buttons to copy from. The click turns the buttons off, and the script refuses a workbook
  that holds a value (its line then says `WorkbookNotFresh` and names the cells), so a case cannot run over
  what another left.
- **Office.js:** run `cases.js`, one case per fresh workbook: `run("M1")`, …, `run("PREC")`, `run("DEP")`. Each sets
  Automatic first and logs one JSON line with the build, the platform, the mode it started in (`modeAtStart`) and
  whether iterative calculation was on (`iterationAtStart`; M20 turns it off for its run and puts it back).
  Every read also logs the application's `calculationState`. Please return the lines as `results.jsonl`; it is
  committed beside this README with a `RESULTS.md`, as the two earlier probe directories do
  (`docs/research/chart-selection-probe`, `docs/research/drawing-anchor-probe`).
- The script enters formulas with `Range.formulas` and calculates with `application.calculate(recalculate)`. They
  stand in for typing and for F9. Where a case's answer differs when it is typed, say so in `RESULTS.md`: S3's
  sentence is about entering a formula in the UI.
- `node check.js` runs every M case against a recording fake of Office.js and checks one JSON line per
  successful case, Automatic mode after successful cleanup, restoration of the initial iteration setting on
  successful runs, and M20 running with iteration off. It also injects sync failures into M20: iteration must be
  restored except when its restoration write is dropped; Automatic mode is checked when `run()`'s cleanup sync
  succeeds. And it runs `PREC` and `DEP`: no group of the workbook is refused, and the queries made are exactly
  those of a list in `check.js`, every cell and range of the P table and of "Dependents asked for" (and A1 and
  R10 in `PREC`), so a query removed or added fails it. It runs a case in a workbook that holds a value: the
  case is refused and nothing is set or calculated. And it clicks the first and the last button on a fake of
  Script Lab's page: each runs its own case, and every button is then off. It needs no Excel and says nothing
  about Excel's values.
- **M9 and M10** are done by hand: see below.
- Excel for the web is the set of record: excel-graph is the only oracle (decision 0266 rule 2), and Script Lab runs
  there. Where the web and the desktop differ, the rule follows the web and the entry is CONFLICTING (decision 0039
  rule 4).
- A run on Excel 365 desktop, en-US, is welcome as a second set, labelled as such. It is evidence only from a
  listed oracle operator who signs it, the bar decision 0427 sets for a desktop observation; relayed without that,
  it is a hint and its rules stay UNVERIFIED.
- M1 to M4, M6 and M12 could also be recorded through Graph, which writes values and formulas, inserts a range,
  reads values and calculates in a session, from an uploaded `calcMode="manual"` file (its `workbookApplication`
  has no call that sets the mode, so M11, which switches it, could not), as
  `docs/research/chart-selection-probe/record_graph.py` recorded that probe. No such recorder is written here. Graph
  has no call for precedents, so the P cases stay Office.js.
- M5, M7 and P16 were dropped while the request was written; ids are not reused.
- A case whose requirement set the host lacks should say so instead of failing silently (the script logs the
  error code).

## M: Manual mode (the design for T-5114)

| Case | Steps | Question | Our provisional rule |
|---|---|---|---|
| M1 | A1 = 1, B1 `=A1+1`; Manual; A1 = 5; read; F9; read | Does B1 read 2 until F9, then 6? | yes: a read returns the stored value |
| M2 | as M1, then enter C1 `=B1*10` before F9 | Is C1 20 (B1's stale value), 60 (B1 calculated first), or empty? Does B1 change? | C1 is calculated at entry, as Microsoft documents (research note, S3); from which value of B1 is the open part |
| M3 | A1 = 7; Manual; enter D1 `=A1*2` | Is D1 14 at once? | yes, as Microsoft documents (research note, S3) |
| M4 | B1 `=A1+1`, E1 `=B1`; Manual; edit B1 to `=A1+2` | Is B1 3 at once? Is E1 still 2? | B1 is 3 at once, as Microsoft documents (research note, S3); E1 stays 2 until F9 |
| M6 | F1 `=RAND()`, G1 `=F1`; Manual; edit A9, which nothing reads; F9; F9 | Does F1 change at the edit? At the first F9? At the second F9, with nothing dirty? | no; yes; yes |
| M8 | as M1, C1 `=B1*10`; Manual; A1 = 5; `Range("C1").calculate()` | Is C1 20 or 60? Is B1 calculated? | no rule yet: `compute` is ours, not `Range.Calculate`. Microsoft documents that it "calculates the range resolving all dependencies within the range" (S3) and recalculates "just the cells in the given range regardless of whether they are dirty or not" (S1), which, read literally, gives 20 with B1 left at 2 |
| M9 | as M1 after A1 = 5, no F9; save; unzip | What cached value does B1 hold in the file, and what does `calcPr` say (`calcMode`, `calcOnSave`, `fullCalcOnLoad`)? Repeat with "Recalculate workbook before saving" off | Microsoft documents that a save in Manual mode recalculates first while "Recalculate workbook before saving" is on, which choosing Manual selects (research note, S3 and 3). Ours gives the same file by another route: a save cannot calculate (it takes `&self`), so it is refused while a cell is owed and the caller calculates first. Please also report the `calcId` the saved file carries |
| M10 | three runs, each in a new Excel session with no other workbook open: (a) open `m10-manual-stale.xlsx`; (b) open `m10-auto-stale.xlsx`; (c) open the manual one, then the auto one. Both hold B1 `=A1+1` cached 999 and C1 `=B1*10` cached 9990. In each run, for each file, before any edit: read B1, C1 and the calculation mode; F9, read; Ctrl+Alt+F9, read. A fourth run, (d), opens `m10-manual-fullcalc.xlsx` alone, the manual file with `fullCalcOnLoad="1"` | What do B1 and C1 show on open? After F9? After Ctrl+Alt+F9? Which mode is Excel in in (a), (b) and (c)? In (d), does Manual win over `fullCalcOnLoad`, as ECMA-376 says (research note, 7)? | the cached values show until a recalculation, in (a), (c) and (d); in (b) Microsoft documents that only formulas that depend on changed cells are recalculated on open (research note, 3), so 999 shows there too |
| M11 | as M1 after A1 = 5; switch to Automatic | Is B1 6 at once? | yes |
| M12 | as M1 after A1 = 5, B3 `=A3+ROW()`; insert a row above row 2 | Is B1 calculated (6)? Is the moved B4 14, with ROW() 4? | a structural edit owes everything; nothing is calculated until F9. Microsoft's pages pull both ways here. S1 and S3 both list the insert of a row among the actions that trigger a recalculation, without the words "in automatic mode" that other items of the same lists carry. But S1 says of its list "When calculation is set to automatic, recalculation occurs after every data input and after certain events such as the examples given in the previous section", S3 says "Manual calculation mode means that Excel recalculates all open workbooks only when you request it by pressing F9 or Ctrl+Alt+F9, or when you save a workbook", and the two lists disagree on which items carry the words ("Renaming a worksheet" has none in S1; S3 has "Renaming worksheets or changing worksheet position when in automatic mode"). We take the two sentences over the missing words; this case asks Excel which |
| M13 | B1 `=A1+1`; Manual; copy B1's formula to B2 (A2 = 100) | Is B2 101, 2 (the source's value) or 0? | calculated at the paste, as an entered formula is (research note, S3, which does not name a paste) |
| M14 | S!A1 = 1, S!B1 `=A1+1`, T!A1 `=S!B1*10`, T!B1 `=A1+1`; Manual; S!A1 = 5; Calculate Sheet (Shift+F9) on T | Is T!A1 20 (S!B1's stale value) or 60? Is T!B1 calculated from it? Is S!B1 still 2? After F9, are all four right? | the sheet's dirty cells run from stored values elsewhere, and stay dirty while a precedent is (Microsoft documents that Shift+F9 "resets all previously uncalculated (dirty) cells as calculated", research note, S3: the F9 after it tests which) |
| M15 | as M1 after A1 = 5; add a defined name `n` = `=S!$A$1*2`; read | Is B1 calculated (6) when a name is added? ("Adding, changing, or deleting defined names" is on S3's list with no "in automatic mode") | nothing is calculated until F9, for M12's reason |
| M16 | as M1 after A1 = 5; hide row 3; read | Is B1 calculated (6) when a row is hidden? ("Filtering, hiding, or un-hiding rows" is on S3's list likewise) | nothing is calculated until F9, for M12's reason |
| M17 | A1 = 5; Manual; enter B1 `=B2+1` and B2 `=A1*2` in one write, and C1 `=A1+1` and C2 `=C1+1` in another; read; F9; read | Is B1 11 at once (the pair calculated together) or 1 (B2 read before it was calculated)? Are C1 and C2 6 and 7? | formulas written together are calculated together: 11, and 6 and 7. Not documented; Microsoft documents that `Range.Calculate` resolves "all dependencies within the range" (S3), and Excel also has `CalculateRowMajorOrder`, which ignores them |
| M18 | A1 = 3; Manual; enter D1 `=SEQUENCE(A1)`; read; A1 = 2; read; F9; read | Does D1:D3 hold 1, 2, 3 at once? After A1 = 2, does D3 keep 3 until F9? | the formula spills at entry, as an entered formula is calculated (S3, which does not name a spill); the spill stays as it is until F9 |
| M19 | S!A1 = 1, S!B1 `=A1+1`, S!C1 `=RAND()`, S!D1 `=C1`, T!A1 `=S!C1`; Manual; Calculate Sheet (Shift+F9) on S with nothing dirty; read; F9; read | Does S!C1 draw at Shift+F9? Does S!D1 follow it? Does T!A1 keep the old draw until F9? | S!C1 draws and S!D1 follows; T!A1 keeps its value until F9. Microsoft documents that "a volatile function is always recalculated at each recalculation" (S3) and that Shift+F9 recalculates "in the active worksheet only" (S1); the two together are not stated |
| M20 | Disable iterative calculation and record the effective setting; A1 `=B1+1` (1); Manual; enter B1 `=A1`; read; F9; read | What does B1 hold at entry: 1 (A1's stored value), 0, or an error? Does A1 change at entry? What do both hold after F9? | B1 is calculated at entry from A1's stored value, 1, and A1 does not change; F9 then finds the loop, and both keep those values (recalc.md, "A circular reference without iteration keeps the cells' values"). Not documented. The warning Excel shows cannot be read by the script: please note in `RESULTS.md` whether it appears at the entry or at F9 |
| V1 | Automatic; F1 `=RAND()`; edit A9, which nothing reads | Does F1 change? | yes (S1 says so; this is the control) |

`generate.py` writes the three M10 workbooks (`python3 -I generate.py <dir>`), the same bytes at every run, and prints
their sha256. The files are deflated, so the sha256s below are those of a Python built on zlib; one built on
zlib-ng writes the same parts under other sha256s. `python3 -I check.py` holds the files to what was sent: the
five parts, in order, their content, and on zlib these sha256s. They are not committed; the request sent to
the oracle owner carries them. Their `calcId` is 191029;
if the probing build writes a higher one (M9 reports it), say so, because Excel then has a version reason to
recalculate the Automatic file on open.

| File | sha256 |
|---|---|
| `m10-manual-stale.xlsx` | `e1e4db729bd2bbf2a6f1074b8b6a201d6b1692d743096ec33c99861a18cb9415` |
| `m10-auto-stale.xlsx` | `298c5aa525fff5257c1a29bf27429f03ba8ce7feebb8b0f54b3a95d9de6851ec` |
| `m10-manual-fullcalc.xlsx` | `5b91946f2abfc733b3b318075f9315bd42a79579948b6bfb71fe508747cc02e8` |

## P: precedents and dependents (the design for T-5115)

`PREC` builds one workbook (sheets S, T, U; formulas in S!H1:H23, a spill at J1, a table `Tb`) and logs
`getDirectPrecedents()` and `getPrecedents()` of every formula cell. `DEP` logs `getDirectDependents()` and
`getDependents()` of the cells they read. The workbook is built in groups; a formula or a name the host refuses is
listed in the line's `buildErrors` and the others still answer. What each formula asks (the ids are the ones that design cites;
P7 is P7a and P7b, P11 the spill rows, P11a to P11g):

| Id | Cell | Formula | Question | Our provisional rule |
|---|---|---|---|---|
| P1 | H1 | `=A1+B1` | Two areas or one merged `A1:B1`? | the areas as written, not merged |
| P2 | H2 | `=SUM(A1:A5)` | one area | `A1:A5` |
| P3 | H3 | `=SUM(A:A)` | The whole column, or clipped to the used range? | `A:A` as written |
| P4 | H4 | `=T!A1+1` | Is the other sheet's cell listed? | yes |
| P5 | H5 | `=SUM(rng)`, `rng` = `S!$C$1:$C$3` | Does a name resolve to its range? | yes: `C1:C3` |
| P6 | H6 | `=calc+1`, `calc` = `SUM(S!$C$1:$C$3)*2` | Does a formula-bodied name give its references? | yes: `C1:C3` |
| P5b | H16 | `=konst+A2`, `konst` = `42` | Only A2? | yes |
| P7a | H7 | `=INDIRECT("C1")` | Is C1 a precedent? | no: none found |
| P7b | H8 | `=INDIRECT(D1)`, D1 holds `C1` | D1 only, or C1 too? | D1 only |
| P8 | H9 | `=SUM(OFFSET(C1,1,0,2,1))` | C1 (the argument), C2:C3 (the target), or both? | C1 only |
| P9 | H10 | `=SUMIF(A1:A3,">0",B1)` | `B1`, or the resized `B1:B3`? | `A1:A3` and `B1` |
| P10 | H11 | `=IF(TRUE,A1,B1)` | Is B1, the branch not taken, listed? | yes |
| P12 | H12 | `=SUM(S:U!A1)` | One area per sheet? | yes |
| P20 | H13 | `=@A1:A5` (H13 is off rows 1 to 5, so `#VALUE!`) | `A1:A5`? | yes |
| P20b | V3 | `=@A1:A5` on a row of the range | The range `A1:A5`, or the intersected cell `A3`? | `A1:A5` |
| P21 | H14 | `=LET(x,A1,x+B1)` | A1 and B1? | yes |
| P17 | H15 | `=1+2` | `ItemNotFound`? | an empty result, where Office.js throws ItemNotFound (S2): the same fact in our API's shape |
| P14 | H17 | `=H1+1` | `getPrecedents`: H1, then A1 and B1? | yes: every level |
| P18 | H18 | `=SUM((A1:A2,B4:B5))` | Two areas? | yes |
| P19 | H19 | `=SUM(A1:B3 B2:C5)` | Both operands, or the intersection `B2:B3`? | both operands |
| P13 | H20 | `=SUM(Tb[Qty])` | `N2:N3`? | yes |
| P11a | J1 | `=SEQUENCE(3)+A1` | A1 | yes |
| P11b | J2 | a spill cell of J1 | The anchor's precedents, the anchor, or none? | the anchor's |
| P11c | K1 | `=SUM(J1#)` | `J1:J3` or `J1`? | `J1:J3` |
| P11d | K2 | `=J2*2` | `J2`; and does `getPrecedents` go on to J1 and A1? | yes |
| P11g | K3 | `=J1*2`, a reader of the anchor cell | `J1`, then A1 | yes |
| P15 | H1:H2 | a two-cell range | The union of both cells' precedents? | yes |
| P22 | H21 | `=MyFn(A1)`, `MyFn` = `LAMBDA(x,x*S!$C$1)` | A1 only, or C1 too, through the named LAMBDA's body? | A1 and C1 |
| P23 | H22 | `=SUM(P1:Q2:R3)` | The bounding rectangle `P1:R3`, or the two operands? (`DEP` asks R1, which only the rectangle holds) | `P1:R3` |
| P9b | H23 | `=SUMIF(A1:A3,">0",B1:B5)` | `B1:B5` as written, or the `B1:B3` SUMIF reads? (`DEP` asks B5) | `A1:A3` and `B1:B5` |
| P11f | S10 | `=SUM(R10#)`, R10 `=SEQUENCE(3)` blocked by R11 | R10 only? | R10 |
| P13b | W2, W5 | `=Tb[@Qty]*2` on a data row of the table and off its rows (`#VALUE!`) | `N2` for W2? Anything for W5? | `N2`; no area for W5 |
| P11h | Y1, Y2 | `=SUM(X1#)`, X1 `=1+1`; `=SUM(X2#)`, X2 = 5 | `X1` for Y1? Anything for Y2, whose `X2#` is `#REF!`? | `X1`; no area for Y2 |
| P15b | X4:X5 | a two-cell range whose X5 `=X4+1` reads its X4 `=A1+1` | Is X4, a cell of the target, among the precedents of the range? Is X5 among its dependents (`DEP`)? | yes to both: a target cell is listed when another target cell reaches it |

Dependents asked for: S!A1, A2, A3 (is V3, whose `@A1:A5` picks A3, a dependent of A3 only or of every cell of
the range? P20b), A5, A9 (nothing but `SUM(A:A)` reads A9), B1 to B3 (is H10 a dependent of B2 or
B3, which only SUMIF's resized range reads?), C1 to C3 (through the names, INDIRECT and OFFSET), D1, J1, J2, J3
(a spill cell: is K1 a dependent of J3?), N2 (through the table), H1, T!A1, and the two-cell range A1:A2. For A1
the question is also whether its dependents hold the anchor J1 only or the whole spill J1:J3, whether
`getDependents` reaches K2, which reads the spill cell J2 (P11e), and whether the conditional format on T1 and the
data validation on U1, whose formulas read A1, are listed (P24). B5, R1, R10 and R11 are asked for P9b, P23 and P11f,
X1 and X2 for P11h (is Y1 a dependent of X1, and Y2 of nothing?), and the range X4:X5 for P15b.

## What we do with the answers

Each answer becomes a rule of `docs/excel-semantics/recalc.md` (the M cases) or `references.md` (the P cases) with
the case id as evidence, and the provisional mark in the two designs comes off. An answer that
contradicts a provisional rule changes the rule, not the answer.

A case that only desktop Excel can answer (M10's keys and its application-wide mode, M9's "Recalculate workbook
before saving") keeps its provisional rule, entered as UNVERIFIED (decision 0266 rule 2); nothing waits for it.
