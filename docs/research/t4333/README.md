# T-4333: the elapsed count before a fraction of a second at a tie

Run these scripts from this directory with plain `python3`.

- `ulp_scan.py` built probes/t4333-count-ulp-scan (56 cells, excel-graph 2026-11-05). It steps eight tie serials of
  the t4323 probes by -3 to +3 of their own ulp under `[s].0`, four whose count Excel rounds up against the half-step
  register and four it truncates. `meta7.json` maps each cell to (day, seconds, ulp offset, ulp exponent).
- `count_models.py` scores the candidate count models on every elapsed-count cell with fraction digits: the seven
  t4323 cases, probes/t4332-fraction-fresh and the ulp scan (590 cells).

## What the scan shows
- Offset 0 is Excel's computed serial `d+k/86400`. Its distance from the decimal tie, in ulps of x, is -0.20, -0.30,
  -0.20, -0.26, -0.32, -0.61, -0.24 and -0.15 for rows 1 to 8. So the double nearest the tie is offset 0, except in
  row 6, where it is +1 (E6, +0.39).
- Every cell where the count disagrees with the fraction digit is a nearest double. At D1:D4 the count is rounded up
  beside ".9" (`TEXT(45000+0.95/86400,"[s].0")` is "3888000001.9"). At E6, `TEXT(3733.95/86400+1*2^-57,"[s].0")` is
  "3733.0": the fraction carried and the count did not.
- One ulp or more from the nearest double, the count and the fraction always agree: "3888000000.9" at -1 ulp,
  "3888000001.0" at +1. The other nearest doubles (D5, D7, D8) agree too. D7 truncates where the register carries.
  This favours a count read from a decimal form of the serial that differs from binary arithmetic only at the
  nearest double (the first reopen hypothesis of T-4333).
- Everywhere the fraction digit is T-4332's field-by-field cut (56 of 56). Only the count misses.

## Candidates (count cells right, of 590)
| model | right |
|---|---|
| register `floor(y*86400/u)` (the engine), `floor(y*{86400,1440,24})`, `floor(y*86400*s)//s` | 573 |
| `floor(days*86400 + (frac(x)+h)*86400)` | 571 |
| `floor(days*86400 + (frac(x)*86400 + 0.5/s))` | 568 |
| value rounded to the shown digit | 567 |
| `floor(x*86400 + 0.5/s)` | 561 |
| `days*86400 + floor(frac(x)*86400 + 0.5/s)` | 556 |
| `days*86400 + floor(frac(y)*86400)`, exact `(x+h)*86400`, x snapped to 17 digits | 554 |
| whole days plus the field-cut clock of `y` | 553 |
| `y*86400` snapped to 17, 15 or 13 digits | 550, 548, 545 |
| x snapped to 15 or 13 digits, then exact | 539, 534 |

Also tried in scratch, and no better than the register on the 534 earlier cells: every one of these in 64-bit
extended precision, with or without a final store to a double (503 to 523); a single half-step constant `c/(86400*s)`
for c within 0.5 +/- 1.5e-6 (best c = 0.5); a float32 half step (496 and 511); chains through 24 and 3600, 24, 60
and 60, or 1440 and 60, floored per field or not, with the half step on the serial or at the seconds stage; snaps of
x to n1 digits and the total to n2 digits (n1 in 15 to 17, n2 in 12 to 17; best 521).
Per unit, the register gets every `[h]` count (285 of 285) and every `[m]` count but carry-ties A27 (100 of 101). The
other misses are all `[s]` counts.
