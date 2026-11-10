# Probe request: val4-fvs-01..06 (FVSCHEDULE's carried overflow state), 2026-11-10

For the Excel oracle agent. Nothing here needs a question back to the author.

## What to run
Six unfilled cases, `eval/suites/probes/cases/val4-fvs-0{1..6}.case.json`. Same format as val3 (`rustsheet-case/v1`, every
formula cell in `check`, no expected values, no sidecars). 185 + 46 + 12 + 12 + 12 + 18 = 285 formula cells.

```bash
cp docs/research/probe-cases-r8fvs-2026-11-10/val4-fvs-0*.case.json eval/suites/probes/cases/
cd tools
for n in 1 2 3 4 5 6; do
  uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/val4-fvs-0$n.case.json
done
```

One workbook per case, each its own session, as val3 was run. Do not merge cases, add sheets, add observers that
read the probe cells, or reorder rows: the position of every formula is the experiment (Excel calculates the last
sheet first, the bottom row first, each row right to left).

## Why these will not be refused like val3-fvs-05, -06, -08
Those three ended their calculation with the overflow's state still pending, and the oracle's canary `SQRT(2)` took
it. Every val4 sheet ends, in Excel's order, with `=2^1023` cells that take the state, so the canary is clean.

## If a fill is refused anyway
That is a result; capture it instead of retrying blind. `rustsheet-oracle fill` deletes the recalculated workbook
when the canary fails, so fetch it directly (from `tools/`):

```bash
uv run python - <<'EOF'
import json, pathlib
from rustsheet_tools.spec2xlsx import build_xlsx
from rustsheet_tools.oracle.canary import embed_canary
from rustsheet_tools.oracle.backends import BACKENDS
case = "val4-fvs-03"   # the refused case
spec = json.loads(pathlib.Path(f"../eval/suites/probes/cases/{case}.case.json").read_text())
data, info = BACKENDS["excel-graph"]().recalc(embed_canary(build_xlsx(spec)))
pathlib.Path(f"/tmp/{case}.refused.xlsx").write_bytes(data)
EOF
cargo run --release -p rustsheet_eval -- snapshot /tmp/val4-fvs-03.refused.xlsx > /tmp/val4-fvs-03.refused.json
```

Return the snapshot JSON (all sheets, the `RustSheetCanary` sheet included) and the refusal message.

Cheap and valuable extra: the same capture for one unchanged `val3-fvs-05` run. Prediction: Probe!B1:B5 = 0, 1, TRUE,
0, 0; Precision!B2:B4 = 0.9999999999999998, 1, 1.0000000000000002 and C2:C4 = -1, 0, 1; canary A4 `#NUM!`, the other
five canary cells right.

## Frozen predictions (written 2026-11-10, before any fill)
"2^1023" is 8.98846567431158E+307.

| Case | If the carried-state reading is right | If it is wrong |
|---|---|---|
| val4-fvs-03 (`{1,0}`), -04 (`{1,1E-300}`), -05 (`{1,-0.5}`) | Probe!A1 `#NUM!`; B1:B5 = 0, 1, TRUE, 0, 0; not refused | A1 2^1023. B1 `#NUM!` with A1 2^1023 would be a per-call classifier after all. 05 and 06 differing would mean stored factors are not enough |
| val4-fvs-06 (no overflow) | every detector 2^1023, every A(4i+4) 1E+300 | a `#NUM!` detector: the detector reacts to something else |
| val4-fvs-01 P!A(4i+4) | 0 in every block | - |
| val4-fvs-01 P!A(4i+1) (second detector) | 2^1023 in every block (one state, taken once) | `#NUM!`: the state is not single |
| val4-fvs-01 blocks `=2^1023`, `=(2^1023)`, `=0*2^1023` | candidate `#NUM!`, first detector 2^1023 | candidate natural, first detector `#NUM!` |
| blocks `=2^-52`, `=2^-1023` | candidate 0, first detector 2^1023 | - |
| blocks `=2^K1`, `=K4^1023`, `=K4^K1`, `=IFERROR(2^K1,"err")` | "literal operands": natural value, first detector `#NUM!`. "every `^`": `#NUM!` (or "err"), first detector 2^1023 | - |
| blocks `=2^K2`, `=2^K3`, `=0*2^K3`, `=IFERROR(0*2^K3,"err")` | "literal operands": natural value, first detector `#NUM!`. "every `^`": 0, first detector 2^1023 | - |
| block `=IFERROR(2^1023,"err")` | "err", first detector 2^1023. "IFERROR shields": 2^1023, first detector `#NUM!` | - |
| block `=SQRT(2)` | `#NUM!`, first detector 2^1023 (the val3 canary) | 1.4142135623730951, first detector `#NUM!`: then the canary failure had another cause |
| block `=FVSCHEDULE(1E+300,1E+300)` (again) | 0, first detector `#NUM!` | - |
| blocks `=2^0`, `=2^10`, `=1.5^2`, `=2^0.5`, POWER, `=SQRT(K4)`, EXP, LN, SIN, `*`, `/`, `+`, `=1E+300*1E+300`, ROUND, VALUE, `=K1+0`, `=FVSCHEDULE(1,1)`, `=1` | census, no hypothesis depends on them; the model guesses `=2^10`, `=1.5^2`, `=2^0.5` `#NUM!` and `=2^0` 1 | - |
| val4-fvs-02 `=IFERROR(B*2^F,"err")` (F = -1023, -997) | G 0, first detector `#NUM!` (repeats val2-fvs-02 column G) | first detector 2^1023: val2's column G was read wrongly |
| val4-fvs-02 `=B*2^F` (F = -1023) | "literal operands": 0, `#NUM!`. "IFERROR shields" or "every `^`": 0, 2^1023 | - |
| val4-fvs-02 `=IFERROR(B*2^-1023,"err")` | "literal operands": 0, 2^1023. "IFERROR shields": 0, `#NUM!` | - |
| val4-fvs-02 `=B+2^F` (F = 1023) | "literal operands": 2^1023, `#NUM!`. "every `^`": `#NUM!`, 2^1023 | - |
| val4-fvs-02 `=B+2^1023` | `#NUM!`, 2^1023 (val2-fvs-01 B17's shape) | 2^1023 with first detector `#NUM!`: dependents run later than recalc.md says |
| val4-fvs-02 `=B+2^-52`; `=B+2^F` (F = -52) | 0, 2^1023; "literal operands": 2.220446049250313E-16, `#NUM!` ("every `^`": 0, 2^1023) | - |
| val4-fvs-02 `=B+SQRT(2)`; `=ISNUMBER(B)` | `#NUM!`, 2^1023 if SQRT reads it; TRUE, `#NUM!` | - |

## What each outcome means for the engine
- "literal operands" holds: the carried state is implementable as the research note states it (rule 2), in the engine's
  existing Excel-order pass, once the engine evaluates the arguments Excel evaluates.
- `=2^K1` is `#NUM!`: references are readers too; then val2's column G was shielded by IFERROR or by the order of
  dependents, and val4-fvs-02 says which.
- Every detector 2^1023 and 03..05 still refused: the reading is wrong and the 60-cell fit of val2 was a coincidence;
  go back to a session effect of Graph.
