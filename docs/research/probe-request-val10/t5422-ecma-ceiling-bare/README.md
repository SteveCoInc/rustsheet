# Probe request: does Excel for the web calculate ECMA.CEILING stored without `_xlfn.`? (T-5422, poc-gateb-cc)

One case, 6 checked cells (the Precision canary; the two formula cells are read from the sidecar, not scored), excel-graph:
`eval/suites/probes/cases/t5422-ecma-ceiling-stored-bare.case.json`, on branch `gateb/ecma-ceiling-bare`
(CR-321689585) and copied here at its suite path.

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t5422-ecma-ceiling-stored-bare.case.json
```

Sheet S holds `=ECMA.CEILING(2.5,2)` in A1 and `=IFERROR(ERROR.TYPE(ECMA.CEILING(2.5,2)),0)` in A2. Both are
listed in the sheet's `bare_function_names`, so the builder writes the name **without** its `_xlfn.` prefix, as
`probes/catalog/function-name-forms` A1 does for PERCENTOF.

## What it decides

Desktop Excel 16 saves ECMA.CEILING bare. T-1301 recorded that Graph refused a workbook holding the bare name
(`createSession` 501 `unsupportedWorkbook`), but that workbook may also have held the one-argument call, which is
refused on its own, and it was not kept.

- **A1 = 4, A2 = 0:** the bare name calculates on the web too. ECMA.CEILING is ISO.CEILING's case (T-0417): the
  engine should store it bare, and the importer exemption and the round-trip harness exemption of CR-321689585 go.
- **A1 = `#NAME?`, A2 = 5:** the web does not know the bare name. The engine's prefixed export stays.
- **The session is refused (501):** that refusal is the answer. Please record it (the fill writes no sidecar then):
  a line in the fill's log is enough, and I will write it into docs/excel-semantics/numbers.md and oracle.md.

Not urgent: it unblocks no CR. It decides a design question a reviewer raised.
