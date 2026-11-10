# PROBE-REQUEST val5, FVSCHEDULE: which reader runs first, and a second census of takers (2026-11-10)

Owner task: T-5108. Two unfilled cases, `eval/suites/probes/cases/val5-fvs-01.case.json` (61 formula cells) and
`val5-fvs-02.case.json` (263). Every formula is in `check`; no expected values, no sidecars. Same format, tags and
canary design as val4: every sheet ends, in Excel's order, with `=2^1023` cells that take a pending state.

## Why
val4 settled that an overflowing FVSCHEDULE step is a live 0 and leaves a state that the next `/`, `^`, POWER, SQRT,
EXP, LN, SIN or ROUND takes, once (financial.md "FVSCHEDULE whose running product overflows or underflows"). Two
things stayed open:
- probes/val2-fvs-02 shows that when four cells read the overflowing cell, the farthest of them does not run before the
  rows above, while a lone reader does (probes/val4-fvs-02). Which of several readers runs right after its precedent
  is not visible there, because the other three take nothing.
- Only the calls of val4-fvs-01 are known to take or leave the state.

## How to run
One workbook per case, each its own session:

```bash
cd tools
for n in 1 2; do
  uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/val5-fvs-0$n.case.json
done
```

Do not add cells, observers or sheets: where each formula sits is the experiment. If a fill is refused, keep the
recalculated workbook (`rustsheet-oracle` deletes it when the canary fails) and return `rustsheet-eval snapshot` of it.

## val5-fvs-01: which reader waits
Sheet P, blocks of three rows. In the bottom row B is `FVSCHEDULE(1E+300,1E+300)` and the cells to its right read it
(label in I); the two rows above hold `=2^1023` detectors in A. A reader `=B+2^1023` is `#NUM!` if it is the first
taker after the overflow, 8.98846567431158E+307 ("2^1023") otherwise. Expected by the rule the engine implements (one
reader waits behind its precedent, the last the calculation meets, which is the one loaded next to it; the others run
after everything else):

| Block (label) | Readers | Expected: readers; first detector |
|---|---|---|
| two_takers | C, D `=B+2^1023` | C `#NUM!`, D 2^1023; 2^1023 |
| three_takers | C, D, E `=B+2^1023` | C `#NUM!`, D and E 2^1023; 2^1023 |
| taker_then_plain | C `=B+2^1023`, D `=ISNUMBER(B)` | C `#NUM!`, D TRUE; 2^1023 |
| plain_then_taker | C `=ISNUMBER(B)`, D `=B+2^1023` | C TRUE, D 2^1023; `#NUM!` |
| lone_taker_next_to_it | C `=B+2^1023` | C `#NUM!`; 2^1023 |
| lone_taker_one_cell_away | C `=1`, D `=B+2^1023` | D `#NUM!`; 2^1023 |
| lone_taker_far | C, D, E `=1`, G `=B+2^1023` | G `#NUM!`; 2^1023 |
| reader_of_a_reader | C `=ISNUMBER(B)`, D `=IF(C,2^1023,0)` | C TRUE, D `#NUM!`; 2^1023 |
| two_takers_gap | C `=1`, D, E `=B+2^1023` | D `#NUM!`, E 2^1023; 2^1023 |
| val2_row | C `=ISNUMBER(B)`, D `=IF(ISERROR(B),ERROR.TYPE(B),0)`, E `=IFERROR(B=0,"err")`, G `=IFERROR(B*2^F,"err")` | TRUE, 0, TRUE, 0; `#NUM!` (repeats val2-fvs-02) |

Every B is expected 0 and every second detector 2^1023. What other outcomes mean:
- the farther taker `#NUM!` and the nearer one 2^1023 (two_takers, three_takers): the first reader the calculation
  meets keeps the place, not the last;
- every reader 2^1023 and the first detector `#NUM!` in two_takers: no reader waits when there are several;
- lone_taker_one_cell_away or lone_taker_far 2^1023 with the first detector `#NUM!`: only a reader loaded right next
  to its precedent runs after it;
- plain_then_taker's D `#NUM!`: the readers of one cell all run before the rows above after all, and val2-fvs-02's
  column G needs another reading.

## val5-fvs-02: second census
The layout of val4-fvs-01: P!A holds one block of four rows per candidate (the overflow A(4i+4), the candidate
A(4i+3) with its label in C, two detectors A(4i+2) and A(4i+1)); sheet Ctl holds each candidate alone in A3 and below
and two more detectors in A1:A2. A candidate that takes the state leaves the first detector 2^1023; one that does not
leaves it `#NUM!`. No hypothesis depends on the answers; the engine today treats as takers only `=POWER(2,-52)`
(expected 0), `=POWER(1.5,2)`, `=1.5^3`, `=0/5`, `=K1/K4` and the taken branch of `=IF(TRUE,1/3,0)` (expected `#NUM!`),
and leaves the state for every other candidate: COS, TAN, ASIN, ATAN, SINH, LOG10, LOG, ABS, INT, MOD, SIGN, PI, FACT,
COMBIN, SUM, AVERAGE, PRODUCT, MAX, SUMPRODUCT, `SUM({1,2}/3)`, ROUNDUP, ROUNDDOWN, TRUNC, MROUND, CEILING, FLOOR, `5%`,
`5-3`, `-K4`, `2^2`, `K4^2`, `1/0`, `SQRT(-1)`, `EXP(1000)`, `LN(0)`, PMT, FV, NPV, TEXT, `&`, the untaken `1/3`, DATE,
YEAR. Each candidate that takes it is a call to add to the engine's list.

# Added after the first review of the change (2026-11-10): four more cases

`val5-order-01.case.json` (4 formula cells), `val5-order-02.case.json` (19), `val5-fvs-03.case.json` (71 + Precision)
and `val5-fvs-04.case.json` (28 + Precision), same directory, same command:

```bash
cd tools
for c in val5-order-01 val5-order-02 val5-fvs-03 val5-fvs-04; do
  uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/$c.case.json
done
```

Priority: val5-fvs-01, val5-order-01 and val5-order-02 decide the calculation-order rule and are what the change
waits for; val5-fvs-03 and val5-fvs-02 are censuses; val5-fvs-04 is optional and may be refused (see below).

## val5-order-01 and val5-order-02: several readers of one precedent, without FVSCHEDULE
The same question as val5-fvs-01, asked through WEEKNUM's stale `return_type` state (dates.md "WEEKNUM's stale
`return_type` state"), so the answer does not rest on one state carrier. A reader `WEEKNUM(d,99)` numbers weeks by the
`return_type` of the WEEKNUM call Excel made before it; joined over d = 2..7 it reads 111111 after a Sunday marker
`WEEKNUM(100,1)`, 222222 after a Monday marker `WEEKNUM(100,2)`, 122222 after a Tuesday marker `WEEKNUM(100,12)`.
Excel runs a sheet bottom row first, right to left; a reader of a formula cell not calculated yet waits for it.

| Layout | every reader runs right after its precedent | only the reader loaded next to it does (the others after the rest) | only the farthest does | none does when there are several |
|---|---|---|---|---|
| val5-order-01 (B2, C2) | 1, 1 | 1, 2 | 2, 1 | 2, 2 |
| TwoReaders (B2, C2) | 111111, 111111 | 111111, 222222 | 222222, 111111 | 222222, 222222 |
| ThreeReaders (B2, C2, D2) | 111111 x3 | 111111, 222222, 222222 | 222222, 222222, 111111 | 222222 x3 |
| TwoWriters A1 | 222222 | 111111 | 222222 | 122222 |
| LoneFar A1 | 111111 | 111111 | 111111 | 111111 |

LoneFar is the control for "only a reader loaded right next to its precedent runs after it": then its A1 is 122222.
Frozen prediction (the rule the first version of the change implemented, second column): (1, 2); 111111, 222222;
111111, 222222, 222222; TwoWriters A1 111111; LoneFar A1 111111. The engine on main today gives the first column.

## val5-fvs-03: FVSCHEDULE's own arguments, and a zero base under the state
Layout of val4-fvs-01 (overflow, candidate, two detectors, bottom to top). The first `^` a formula evaluates takes the
state (val4-fvs-01), so the candidate's value shows which argument Excel evaluates first:

| Candidate | principal first (left to right) | schedule first |
|---|---|---|
| `=FVSCHEDULE(2^1023,2^-1)` | `#NUM!`; detector 2^1023 | 8.98846567431158E+307; detector 2^1023 |
| `=FVSCHEDULE((1+33562624*2^-52)*2^1023,1-33562624*2^-51)` | about 1.7976931348623157E+308; detector 2^1023 | 0; detector `#NUM!` |
| `=SUM(2^1023,2^-1)` | `#NUM!` | 8.98846567431158E+307 |
| `=SUM(2^-1,2^1023)` | 8.98846567431158E+307 | `#NUM!` |

Frozen prediction: left to right in all four. `=FVSCHEDULE(1,2^-1)` and `=FVSCHEDULE(2^-1,1)` are controls (1 and 0
if the power takes the state). The others have no prediction: `=0^-1`, `=POWER(0,-1)`, `=0^0`, `=POWER(0,0)`,
`=(-8)^0.5`, `=0/0`, `=K4^K2`, `=POWER(K4,K2)` ask whether a power or a quotient that is an error by itself takes the
state, and what it returns then.

## val5-fvs-04: the state and dynamic-array formulas (optional)
No prediction. Each block has a spilling formula (`=SEQUENCE(1,2)+...`, spilling one cell to the right) next to an
overflow: does an overflow inside a spilling formula leave the state, does a SQRT, a `/` or a `^` inside one take it,
does a spilling formula that only reads the overflowing cell disturb it. Where Excel calculates a spilling formula is
not known: if it is after every other cell, the state is still pending when the calculation ends and **the canary will
refuse this fill**. Then keep the recalculated workbook and return `rustsheet-eval snapshot` of it; the refusal itself
is the answer to the first question.
