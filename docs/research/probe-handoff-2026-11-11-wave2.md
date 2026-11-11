# Probe hand-off, wave 2: everything still open, in one request (2026-11-11)

**For:** the Excel oracle owner / the Excel-for-the-web oracle agent.
**Why:** one request in place of the open ones. It carries what #4123 left unfilled after #4124, and the probe
cases written since #4123 merged (2026-11-10 22:17Z).

## PROBE-REQUEST

**35 pending case files** (`rustsheet-case/v1`, no expected values, no oracle sidecars) and **one hand recording**.
The engine, evaluator, tests, baselines and gates are untouched.

| Part | Cases | Checked cells | Lane | What to return |
|---|---:|---:|---|---|
| A. #4123's two large HYPGEOM.DIST cases, split | 23 | 1,207 | statistics (r8hyp) | 23 `.oracle.json` sidecars, or the parts refused |
| B. New statistics probes | 2 | 489 | statistics (r8hyp) | 2 `.oracle.json` sidecars |
| C. New formula-semantics probes | 10 | 280 formula cells | engine, engine-b, xo-onto | 10 `.oracle.json` sidecars |
| D. Chart Styles presets (#4123's last commit, not merged) | — | — | charts | `chart-styles.golden.xlsx` |

Still open from earlier requests, with **no new files here**: the 24 AutoFill probes in `eval/fill/` (#4119) need
Office.js, since Graph has no fill method. Their results go to `eval/fill/refs/officejs/<probe-id>.fill.json`, as
`eval/fill/README.md` describes. The tier-2 render recorder `eval/render/record-tier2.js` (#4119) needs the Excel UI gym
or Excel desktop. Both wait for your gym, as before.

### A. `t1896-hypgeom-cdf-wide` and `t5109-fresh`, split (statistics lane, T-1896, T-5109, T-2481)

#4124 records that both workbooks failed twice on excel-graph (Graph's `calculate` or `createSession`, retries
exhausted) and asks for them to be split. Here they are as 23 smaller workbooks in
`eval/suites/probes-numeric/cases/`, made by `python3 docs/research/probe-wave2/split_cases.py <case> <out-dir>`
(stdlib only; it regenerates these files byte-identically). The splitter works like this:

- Each sheet becomes its own workbook.
- A sheet that holds a HYPGEOM.DIST cdf summing more than 2^17 terms (x - max(0, n + K - N) + 1) is split by row
  instead. Its other rows go in one `-light` part, and the heavy rows go in groups of at most 6. This follows
  statistics.md: the largest single cdf on record that Excel answered is 108,560 terms, and the smallest refused alone
  is 153,801.
- Every part keeps its sheet's header row 1 and the case's `Precision` sheet.
- Sheet names, cell addresses, formulas and the order of `check` are the original's. No formula reads another row or
  another sheet, and the splitter refuses any that does.

So the frozen predictions apply cell for cell: `docs/research/t5109-fresh-predictions.json` (on main) and
`docs/research/t1896-cdf-wide-predictions.json` (added here).

| Part | Sheet and rows | Checked cells | cdf cells | Size |
|---|---|---:|---:|---|
| `t5109-fresh-calc` | Calc | 606 | 0 | light |
| `t5109-fresh-poisson` | Poisson | 86 | 0 | light |
| `t5109-fresh-closed` | Closed | 41 | 0 | light |
| `t5109-fresh-cdf` | Cdf | 126 | 60 | light |
| `t1896-hypgeom-cdf-wide-rare-light` | Rare rows 9, 12 | 12 | 2 | light |
| `t1896-hypgeom-cdf-wide-p-01`, `-p-02` | P rows 2-7, 8-13 | 24 each | 6 each | heavy |
| `t1896-hypgeom-cdf-wide-rare-01`, `-rare-02` | Rare rows 2-7; 8, 10-11, 13 | 24, 18 | 6, 4 | heavy |
| `t5109-fresh-cdfwide-01` .. `-11` | CdfWide rows 2-7, 8-13, .., 62-67 | 18 each | 6 each | heavy |
| `t5109-fresh-cdftie-01` .. `-03` | CdfTie rows 2-7, 8-13, 14-16 | 18, 18, 12 | 6, 6, 3 | heavy |

Checked cells include each part's 6 `Precision` cells. Light parts first: they hold 871 of the original cells and should
fill as the other probes do. The heavy parts may be refused, like the single 153,801-term cell. **If a part fails twice,
do not retry it.** Name it in the PR as refused: the cases ask which rows Excel cannot answer, and a refusal is that
answer. To narrow a refused part down to its rows, run the splitter again with `--max-rows 1`. Please do not fill the two
original cases whole, because that is what failed. Once the parts are in, the originals hold no cell the parts lack. You
can drop them in the same PR or leave them pending.

### B. New statistics probes (statistics lane)

| Case | Checked cells | What it pins | Frozen predictions |
|---|---:|---|---|
| `probes-numeric/t2586-negbinom-edge` | 79 | NEGBINOM.DIST pmf: where its zero stops (the quotient p / (f + 1) on both sides of 2.2251E-308), r + f from 1E10 to 2E15, one near-tie (T-2586) | `docs/research/t2586-negbinom-edge-predictions.json` |
| `probes-numeric/t3247-poisson-closed-fresh` | 410 | a fresh hold-out for a candidate closed form of POISSON.DIST's pmf off the saddle point, EXP(S)(1 + err), which fits all 48 cached cells (T-3247) | `docs/research/t3247-poisson-closed-fresh-predictions.json` |

Both have the `Precision` sheet. The predictions are research metadata, **not oracle or math sidecars**. Never copy them
into expected values.

### C. New formula-semantics probes (engine lanes)

| Case | Formula cells | Lane | Question |
|---|---:|---|---|
| `probes/address-empty-args` | 25 | engine | ADDRESS with each optional argument written empty, alone and through IF/CHOOSE and variables |
| `probes/range-to-a-call-cells` | 6 | engine | a range whose right operand is a call returning a reference, in cells, and its FORMULATEXT |
| `probes/legacy-array-parameters-linest` | 93 | engine | scalar functions and IF nested in LINEST/LOGEST/TREND/GROWTH/SLOPE/.. array parameters in a legacy formula, on and off the data rows |
| `probes/legacy-unary-plus-aggregate-argument` | 63 | engine | `SUM(+B1:B6)` and kin in a legacy formula, on and off the range's rows |
| `probes/aggregate-array-form-3d-span-hidden` | 8 | engine-b | AGGREGATE's array form over a 3-D span whose sheets hide different rows (T-5292) |
| `probes/date-text-spaced-hyphens` | 32 | engine-b | a date text with a space after each hyphen, read by arithmetic, VALUE and DATEVALUE, with variants |
| `probes/sheet-span-cell-like-names-typed` | 6 | engine-b | typed 3-D references over sheets named like cells (T2, T3), quoted and not, and their FORMULATEXT |
| `probes/defined-name-body-scope-first-sheet` | 19 | engine-b | which names a defined name's body reads when the sheet holding sheet-local names is the first sheet |
| `probes/legacy-index-match-one-row` | 18 | engine-b | a legacy INDEX over a one-row or one-column range with a MATCH index: does FORMULATEXT show `=@INDEX(...)`? |
| `probes/structured-ref-digit-column` | 10 | xo-onto | a table column headed by digits: `SUM(Years[2020])`, its bracket and range forms, and their FORMULATEXT |

Each case's `description` says what it pins and why. Six of them carry no `check` list (the older layout): fill every
formula cell.

### D. Chart Styles presets (charts lane)

`docs/research/chart-styles-probe/` (README.md, cases.js) asks what Excel writes into the package for each Chart Styles
preset of each chart type. It joined the combined branch at 22:26Z, after #4123 had merged, so it is not on main.
Graph cannot apply a preset, so this needs Excel's UI (desktop or web, by hand) or Office.js. The README gives the
steps, the sheet and chart names, and an order if the full set is too much. Return `chart-styles.golden.xlsx`
in that directory, with a RESULTS.md as for chart-selection and drawing-anchor in #4124, naming the app and API used.
It is low priority.

## How to run

From `tools/`, one workbook per case. List the cases rather than filling the suites whole: a suite-wide fill (or the
`oracle` workflow) would also retry the two unsplit originals.

```bash
p=(); for c in address-empty-args range-to-a-call-cells legacy-array-parameters-linest \
  legacy-unary-plus-aggregate-argument aggregate-array-form-3d-span-hidden date-text-spaced-hyphens \
  sheet-span-cell-like-names-typed defined-name-body-scope-first-sheet legacy-index-match-one-row \
  structured-ref-digit-column; do p+=(--case "probes/$c.case.json"); done
uv run rustsheet-oracle fill -b excel-graph --suite probes "${p[@]}"

n=(); for c in t2586-negbinom-edge t3247-poisson-closed-fresh t5109-fresh-calc t5109-fresh-poisson \
  t5109-fresh-closed t5109-fresh-cdf t1896-hypgeom-cdf-wide-rare-light; do n+=(--case "probes-numeric/$c.case.json"); done
uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric "${n[@]}"

h=(); for f in ../eval/suites/probes-numeric/cases/t1896-hypgeom-cdf-wide-{p,rare}-0*.case.json \
  ../eval/suites/probes-numeric/cases/t5109-fresh-cdf{wide,tie}-*.case.json; do h+=(--case "probes-numeric/${f##*/}"); done
uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric "${h[@]}"   # the 18 heavy parts, last
```

In every workbook with a `Precision` sheet, `Precision!B2:B4` must be **0.9999999999999998, 1, 1.0000000000000002**
and `Precision!C2:C4` **-1, 0, 1**. A capture failure means stop. Return the sidecars with full binary64 values, and do
not transcribe displayed numbers. Excel for the web is sufficient for parts A to C. Leave every case file as it is.

## How it will be scored

Each lane scores its cases against the predictions frozen before the fill (the files named above). The engine lanes'
cases have no frozen predictions: the visibly-off cells get owners when they land, as `scripts/new_case_owners.py`
requires. A refused heavy part is recorded as an oracle limit, not as an Excel value, as statistics.md does today.
