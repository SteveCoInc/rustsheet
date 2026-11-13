# This workbook's names over a linked workbook's cells (Taskei P560248216; engine lane)

One fixture, `external-link-own-names.xlsx`, 38 checked cells, excel-graph. It is `external-link-cached-ranges.xlsx`
(of the `fixtures` suite) with names of *this* workbook over references into the linked one: `Start` =
`[1]Uses!$B$2` (the cached text "7"), `Num` = `[1]Uses!$B$1` (10), `Area` = `[1]Uses!$A$1:$B$2`, `Blank` =
`[1]Uses!$Z$9` (not cached) and `Nope` = `[1]Nope!$A$1` (no such sheet). Sheet `Refs`: one legacy formula per row
through the name in column B, and the same formula over the written reference in column C (placeholder cached 0):
SUM, COUNT, COUNTA, MAX, `&""` and ISREF of `Start`; SUM, COUNT, COUNTA, ROWS and ISREF of `Area`; SUM of `Num`;
COUNTA of `Blank` alone and through IF; ISREF, SUM and ROWS through IF and CHOOSE; ERROR.TYPE of `Nope`.

The question: does Excel read such a name as the reference it stands for, so that `SUM(Start)` is 0 as
`SUM([1]Uses!$B$2)` is, or as its value? The engine reads it as its value (`SUM(Start)` 7, `COUNT(Start)` 1,
`ISREF(IF(TRUE,Start))` FALSE). The engine change was withdrawn until this answers.

It is here, under `docs/`, and not in `eval/suites/fixtures/cases/`, because a fixture without Excel's values would
be scored against its placeholder cached values. Run it by path, from `tools/`:

    uv run rustsheet-oracle recalc -b excel-graph ../docs/research/probe-request-val11/external-link-own-names/external-link-own-names.xlsx > external-link-own-names.graph.json 2> external-link-own-names.graph.err

Wanted back: the JSON (or the error), and the recalculated workbook the tool keeps.
