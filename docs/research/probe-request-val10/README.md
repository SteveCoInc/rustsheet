# Probe request val10 (2026-11-12): every open probe request, in one change

This change replaces PR #4127 (val9). #4127 failed the `eval (vendored suites)` check because it put four unfilled
accuracy batches under `eval/accuracy/tables/`, where 4 of their 800 points fail until they have Excel values. Here
those batches are under `docs/research/probe-request-val9/t2574-accuracy/`, and every other open request is added.

**Excel for the web only.** Every item below runs through excel-graph or through Office.js (Script Lab) in Excel for
the web, or by hand in the browser. Items that need desktop Excel are left out; they are listed at the end.

The change only adds files. No `.github/` file, no Rust or Python source, and no baseline is touched. Each pending
probe case has no `.oracle.json` sidecar yet, so the gates count it as pending.

## 1. excel-graph fills from `tools/` (automatic)

Two commands fill every probe case below that sits in a gate suite. They also pick up the six older cases on main
that still have no sidecar (legacy-unary-plus-aggregate-argument, t0481-curly-quote-names-c, typed-if-one-argument,
val3-fvs-05, val3-fvs-06, val3-fvs-08):

```sh
cd tools
uv run rustsheet-oracle fill -b excel-graph --suite probes
uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric
```

Wanted back: the new `*.oracle.json` sidecars, committed beside their cases.

| # | Item | Size | Files | Effort |
|---|---|---|---|---|
| 1 | val9, carried over from #4127: LINEST R4.2 zero-head probe and four accuracy batches; F.INV / BETA.INV search fresh sweep and its tails (T-5107); three probe cases new to main; stress typed values (T-5447) | 6 probe cases (61 + 526 + 287 cells in the three sweeps); T-5447's case (60 cells); 4 accuracy batches of 200 points | `docs/research/probe-request-val9/` (README and the per-item notes), `docs/research/t2574/`, `docs/research/t5107-fin-val4/`, `eval/suites/probes/cases/{t2574-zero-head-fresh,t5107-search-fresh,t5107-search-fresh-tails,t-n-at-unary-plus-if-array,at-unary-sign-range,eng-impower-modulus-choice}.case.json` | fill; then the accuracy batches (copied in for the fill, not committed) and T-5447 by path: see `probe-request-val9/LINEST-R42.md` and `stress-t5447/README.md` |
| 2 | M4 structural edits: row and column inserts and deletes (T-5123, T-5358, T-5265). This is the M4 request sent on 2026-11-11 as `research/probe-request-m4`; it is here so one change carries everything | 53 cases, 599 cells | `eval/suites/probes/cases/table-edits/` (48 cases), `docs/research/structural-probes-2026-11-11/` (`PROBE-REQUEST.md`, `predictions.tsv`, and 5 cases in `engine-refused/` to run by path) | fill, plus 5 runs by path; a refusal (Graph 409) is an answer for 9 cases, see `PROBE-REQUEST.md` |
| 3 | M4 shift: Insert / Delete Cells with a shift (T-5465). Sent on 2026-11-11 as `research/probe-request-m4-shift` | 32 cases, 790 cells | `docs/research/shift-probes-2026-11-11/` (README, `cases/`, `tools-case-ops.patch`, `predictions.tsv`, `gen_cases.py`) | 32 runs by path, and one resave for the four `*-records` cases, with the README's local patch applied for the run and reverted after |
| 4 | T-4579: TTEST paired, fresh sweep | 1 case, 384 cells | `eval/suites/probes/cases/t4579-ttest-paired-fresh.case.json`, `docs/research/t4579/`, `probe-request-val10/t4579-paired-fresh/README.md` | fill |
| 5 | T-4994: BETA.INV at shape 1 | 1 case, 253 cells | `eval/suites/probes/cases/t4994-shape-one-fresh.case.json`, `docs/research/t4994/`, `probe-request-val10/t4994-shape-one-fresh/README.md` | fill |
| 6 | T-5524: CHIINV at df 1 (suite probes-numeric) | 1 case, 515 cells | `eval/suites/probes-numeric/cases/t5524-chisq-df1-complement-fresh.case.json`, `docs/research/t5524/`, `probe-request-val10/t5524-chisq-df1-fresh/README.md` | fill (probes-numeric) |
| 7 | T-4894: F.INV.RT in the saturation band | 1 case, 66 cells | `eval/suites/probes/cases/t4894-f-inv-rt-saturation-band.case.json`, `docs/research/t4894/`, `probe-request-val10/t4894-saturation-band/README.md` | fill |
| 8 | T-5422: ECMA.CEILING stored without `_xlfn.` | 1 case, 6 cells | `eval/suites/probes/cases/t5422-ecma-ceiling-stored-bare.case.json`, `probe-request-val10/t5422-ecma-ceiling-bare/README.md` | fill; a refused session (501) is the answer, so please record it |
| 9 | Legacy lookup value intersection, and range corners as right operands | 2 cases, 85 cells | `eval/suites/probes/cases/{legacy-lookup-value-intersection,range-corner-right-operands}.case.json` | fill |
| 10 | 2-D implicit intersection across sheets | 1 case, 27 cells | `eval/suites/probes/cases/implicit-intersection-two-d-range.case.json` | fill |
| 11 | Formula length limits: typed entries and names (P559259288) | 2 cases, 27 cells | `eval/suites/probes/cases/{typed-formula-length-utf16-and-open-call,api-name-body-length-limit}.case.json` | fill; a REFUSED line in the log is the answer for entries Excel refuses |
| 12 | Pivots: missing caption values, one custom subtotal, custom Product grand total (T-5421, T-4925) | 3 cases | `eval/suites/probes/cases/{t4922-pivot-missing-caption-values,t4925-pivot-one-custom-subtotal,t5483-pivot-custom-product-grand}.case.json` | fill |
| 13 | Packages with a missing part: does Excel open them | 5 workbooks | `probe-request-val10/cfix-missing-parts-probe/` (README, the five `.xlsx`, `make_probes.py`) | one excel-graph session per workbook; a 501 is an answer |

## 2. Office.js in Script Lab (Excel for the web)

| # | Item | Size | Files | Effort |
|---|---|---|---|---|
| 14 | T-5145: Manual calculation mode, precedents and dependents | 18 Manual-mode cases (M1 to M20) and the precedent and dependent cases P1 to P23 (two runs, PREC and DEP) | `docs/research/probe-request-t5145-recalc/` (README, `cases.js`, `check.js`, `generate.py`, three M10 workbooks) | one fresh workbook per case; M9 and M10 partly by hand (see the README) |
| 15 | Small line charts: plot area and automatic value axis | 10 cases | `probe-request-val10/small-chart-layout-probe/` (README, `cases.js`) | one `runAll()` |
| 16 | Workbook locale: regional format and display language (SPEC-0046) | one script, run once per setting row | `docs/research/workbook-locale-probe/` (README, `cases.js`) | one run per row of its settings table (its README: a few minutes a run) |
| 17 | Distinct values in an AutoFilter list and in Remove Duplicates (D-CONTRACTS-24) | 9 one-column cases | `probe-request-val10/d-contracts-24-distinct-dedupe-probe/README.md` | two calls per case; the README's "Office.js" section only |

## 3. By hand in Excel for the web

| # | Item | Size | Files | Effort |
|---|---|---|---|---|
| 18 | Table banding when a data row is hidden by a filter or by hand | 2 screenshots | `probe-request-val10/table-banding-hidden-rows/README.md` | a new workbook and two screenshots |

## Left out: these need desktop Excel

- xls-name-as-cell: it needs desktop Excel for Windows, to make an `.xls` by hand in Compatibility Mode and convert it.
- Outline A1 to A5 (Group, Ungroup, Hide and Show Detail, Clear Outline, SUBTOTAL over collapsed groups): written as
  desktop probes. Excel for the web has Group and Ungroup, so A2, A3 and A5 may be possible there, but nobody has checked
  that the web has Clear Outline (A4) or shows the same refusal text (A1).
- D-MODEL-91, a table's own AutoFilter: its README gives desktop Excel 365 steps, including opening a hand-edited file
  and reading Excel's repair log.
- D-CONTRACTS-24, the "Desktop" section only: the AutoFilter list's order and date tree as shown on the desktop (item 17
  keeps the Office.js part).
- T-5145, the desktop-only parts: M9's "Recalculate workbook before saving" option and M10's application-wide mode
  (item 14's README already keeps these as UNVERIFIED).
