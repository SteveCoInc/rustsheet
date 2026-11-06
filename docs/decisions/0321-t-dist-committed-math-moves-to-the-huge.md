# 0321 — T.DIST committed math moves to the huge-x #NUM! cut

- **Status:** accepted (2026-11-06, T-4580)
- **Context:** T-4102 encoded Excel's huge-x `#NUM!` for T.DIST.RT, T.DIST.2T and TDIST as an evidenced domain rule
  (`student_t.HugeXCut`): from the smallest evidenced `#NUM!` |x| on, x is outside the domain; strictly between the
  largest evidenced number and that `#NUM!`, the point is unprobed and has no value. T.DIST kept a math number at every
  huge x, so 34 points of its committed batches (accuracy/T.DIST s1, e-T-3425 and r4-T-2585) held a 50-digit number
  where Excel's own cell is `#NUM!`. Decision 0199 makes a committed batch immutable; a corrected batch is a new batch
  unless a decision record authorizes the change. A new batch would need a fresh oracle fill of the same formulas, and
  would leave the old batches' math wrong beside it.
- **Decision:** T.DIST's definition uses `HugeXCut` per `cumulative` and per sign, with each bracket taken from every
  Excel cell of one T.DIST call with |x| above 1E10 (only the three tables have one; a cumulative cell with df above
  1E10 is `#NUM!` for its df and brackets nothing):

  | T.DIST | Largest number | Smallest `#NUM!` |
  |---|---|---|
  | cumulative, x > 0 | 7.34E+146 (e-T-3425 A92) | 1.90E+159 (r4-T-2585 A217) |
  | cumulative, x < 0 | 2.01E+153 (r4-T-2585 A223) | 5.43E+196 (e-T-3425 A88) |
  | density, x > 0 | 9.93E+137 (s1 A97) | 3.84E+200 (r4-T-2585 A267) |
  | density, x < 0 | 1.12E+153 (r4-T-2585 A236) | 9.72E+176 (r4-T-2585 A266) |

  The three committed math sidecars are rewritten in place, and only where the recomputed value changed: 34 cells move
  from a number to a domain error (8 in s1, 8 in e-T-3425, 18 in r4-T-2585), each one a cell whose Excel value is
  `#NUM!`. No other cell, no case file and no oracle file changes, and no table point falls inside a bracket (the
  brackets come from the tables' own cells).
- **Consequences:** `rustsheet-accuracy verify T.DIST` reports 0 differences. In Gate A, the 16 cumulative points move
  from "Excel only" to inside on both sides (the engine already gives `#NUM!`); the 18 density points stay error
  mismatches owned by T-3554 (engine 0, Excel `#NUM!`). The brackets are no wider than their cells: a new Excel cell
  inside a bracket narrows it, and a cell that contradicts one (a number past the smallest `#NUM!`) needs a definition
  change with its own record. This record authorizes only these 34 cells; any other rewrite of a committed
  batch still needs its own record.
