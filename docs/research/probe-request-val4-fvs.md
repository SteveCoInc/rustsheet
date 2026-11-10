# PROBE-REQUEST val4, FVSCHEDULE: the state an overflowing call leaves (2026-11-10)

Owner task: T-5108. Six unfilled cases, `eval/suites/probes/cases/val4-fvs-01..06.case.json` (185, 46, 12, 12, 12 and
18 formula cells; every formula is in `check`; no expected values, no sidecars). Same format and tags as val3.

## Why
Read with Excel's fresh-load order (recalc.md "The order Excel calculates a freshly loaded workbook in"), every
`#NUM!` of val2-fvs-01 and val2-fvs-02 is the first `^` with two literal operands that Excel calculates after an
overflowing FVSCHEDULE: the overflowing call is a live 0 and leaves a state, and that `^` takes it once (`#NUM!` for a
positive exponent, 0 for a negative one). The same state reached the canary's `SQRT(2)` in val3-fvs-05, -06 and -08,
whose calculation ended with it pending, which is why those fills were refused. What is not known is which calls take
the state: in val2-fvs-02 the `^` of `IFERROR(B*2^F,"err")`, F a constant cell, does not.

## How to run
One workbook per case, each its own session, as val3 was run:

```bash
cd tools
for n in 1 2 3 4 5 6; do
  uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/val4-fvs-0$n.case.json
done
```

Do not merge cases, add sheets, add observers that read the probe cells, or reorder rows: where each formula sits is
the experiment (Excel calculates the last sheet first, the bottom row first, each row right to left, and a cell that
reads a formula cell moves to just after it). Every sheet ends, in that order, with `=2^1023` cells that take the
state, so the canary should pass. If a fill is refused all the same, that is a result: `rustsheet-oracle` deletes the
recalculated workbook when the canary fails (`_recalc_cached`), so fetch it once more with
`BACKENDS["excel-graph"]().recalc(embed_canary(build_xlsx(spec)))`, keep the bytes and return
`rustsheet-eval snapshot` of them, canary sheet included.

## What each probe pins, and what is expected
"2^1023" is 8.98846567431158E+307. Predictions were frozen before any fill.

- **val4-fvs-03, -04, -05**: val3-fvs-05, -06 and -08 again (`FVSCHEDULE(2^1023,{1,0})`, `{1,1E-300}`, `{1,-0.5}`), with
  `=2^1023` in Probe!A1, which Excel calculates after B1 and its observers. Pins: the value of the overflowing call
  alone, and that the state exists. Expected: A1 `#NUM!`; B1:B5 = 0, 1, TRUE, 0, 0; not refused. If A1 is 2^1023 the
  state is not what val2 suggests; a B1 that is `#NUM!` with A1 2^1023 would be a per-call classifier after all; a 05
  and 06 that differ would mean the stored factors are not enough.
- **val4-fvs-06**: the val4-fvs-01 layout with `FVSCHEDULE(1E+300,0)`, no overflow. Pins: the detector reacts to
  nothing else. Expected: every detector 2^1023, every A(4i+4) 1E+300.
- **val4-fvs-01**: P!A holds one block of four rows per candidate; Excel runs A(4i+4) `FVSCHEDULE(1E+300,1E+300)`
  (expected 0), A(4i+3) the candidate (label in C), then the detectors A(4i+2) and A(4i+1), both `=2^1023`. A candidate
  that takes the state shows its own damage and leaves the first detector 2^1023; one that does not leaves it `#NUM!`.
  The second detector is expected 2^1023 in every block (one state, taken once). Sheet Ctl holds each candidate alone,
  calculated with nothing pending. Pins: the class of readers.

  | Candidate | "two literal operands" expects: candidate, first detector | "every `^`" expects |
  |---|---|---|
  | `=2^1023`, `=(2^1023)`, `=0*2^1023` | `#NUM!`, 2^1023 | the same |
  | `=2^-52`, `=2^-1023` | 0, 2^1023 | the same |
  | `=2^K1`, `=K4^1023`, `=K4^K1` (K1 = 1023, K4 = 2) | 2^1023, `#NUM!` | `#NUM!`, 2^1023 |
  | `=2^K2` (K2 = -52) | 2.220446049250313E-16, `#NUM!` | 0, 2^1023 |
  | `=2^K3` (K3 = -1023), `=0*2^K3`, `=IFERROR(0*2^K3,"err")` | 0, `#NUM!` | 0, 2^1023 |
  | `=IFERROR(2^1023,"err")` | "err", 2^1023 ("IFERROR shields" expects 2^1023, `#NUM!`) | the same |
  | `=IFERROR(2^K1,"err")` | 2^1023, `#NUM!` | "err", 2^1023 |
  | `=SQRT(2)` | `#NUM!`, 2^1023 (from the val3 canary) | the same |
  | `=FVSCHEDULE(1E+300,1E+300)` again | 0, `#NUM!` | the same |
  | `=2^0`, `=2^10`, `=1.5^2`, `=2^0.5`, POWER, `=SQRT(K4)`, EXP, LN, SIN, `*`, `/`, `+`, `=1E+300*1E+300`, ROUND, VALUE, `=K1+0`, `=FVSCHEDULE(1,1)`, `=1` | the census: no hypothesis depends on them (guess: `=2^10`, `=1.5^2`, `=2^0.5` `#NUM!`; `=2^0` 1 with the first detector `#NUM!`; `=1+1`, `=VALUE("1.5")`, `=1` do not take it) | the same |

- **val4-fvs-02**: the same for a cell that reads the overflowing cell (val2-fvs-02's column G). In each block B is the
  overflow, G one dependent (label in I, constant F), and the two rows above hold the detectors. Pins: whether the
  reference exponent or the IFERROR shields val2's column G, and that a dependent runs right after its precedent.

  | G | "two literal operands" expects: G, first detector | Others |
  |---|---|---|
  | `=IFERROR(B*2^F,"err")`, F = -1023 or -997 | 0, `#NUM!` (repeats val2-fvs-02) | "every `^`": 0, 2^1023 |
  | `=B*2^F`, F = -1023 | 0, `#NUM!` | "IFERROR shields" and "every `^`": 0, 2^1023 |
  | `=IFERROR(B*2^-1023,"err")` | 0, 2^1023 | "IFERROR shields": 0, `#NUM!` |
  | `=B+2^F`, F = 1023 | 2^1023, `#NUM!` | "every `^`": `#NUM!`, 2^1023 |
  | `=B+2^1023` | `#NUM!`, 2^1023 (val2-fvs-01 B17's shape) | a dependent that runs later than recalc.md says: 2^1023, `#NUM!` |
  | `=B+2^-52` | 0, 2^1023 | the same |
  | `=B+2^F`, F = -52 | 2.220446049250313E-16, `#NUM!` | "every `^`": 0, 2^1023 |
  | `=B+SQRT(2)` | `#NUM!`, 2^1023 if SQRT takes it | - |
  | `=ISNUMBER(B)` | TRUE, `#NUM!` | - |

## What the outcomes decide
- "two literal operands" holds: the rule can be implemented in the engine's existing Excel-order pass, with the
  reader class known.
- `=2^K1` is `#NUM!`: references take the state too; val2's column G was then shielded by IFERROR or by the order of
  dependents, and val4-fvs-02 says which.
- Every detector 2^1023 and 03..05 refused again: the reading is wrong; the canary failures were something else.
