# 0341 — T-4435's accuracy change leaves six ungated probe cells visibly off

- **Status:** accepted (2026-11-07, T-4435; recounted and amended 2026-11-10 after two fidelity reviews). The file
  name keeps the first count, seven.
- **Context:**
  - T-4435 is decision 0114's `accuracy` stage. It moves the incomplete beta at one huge shape outside
    `large_shape_beta`'s old bounds onto the math:
    - region 1 is b0 < 40 past L140's 9 bup terms, at a0 from 2000 to 1E6;
    - region 2 is bfrac at b0 of 40 to 200 past T-3571's probe coverage (a0 above 5000).
  - In that range the expansion forms nu, z, F's ratio and the gamma density in double-double.
  - The comparison is against origin/main bbbc45e77 (docs/research/t4435: moved_cells.py, lost_excel.py,
    gate_a_moves.py).
  - **Gates:**
    - `rustsheet-eval check`: no regression in any gated suite, and the baselines are unchanged.
    - Visible parity: 8,750 in-scope cells outside the envelope, against the lock's 9,702 in 211 cases.
    - Gate A: 0 failing and the same 35 known failures. 6 of its 45,082 points move, all toward the math, and 2 of them
      (F.DIST A172 and FDIST A176) go from math-only into both envelopes.
  - **Eval:** 889 cells move:
    - 177 to a better closeness class and 19 to a worse one;
    - 479 within their class toward Excel and 210 away;
    - the incomplete-beta family's cells within 15 digits go from 368,209 to 368,246, and its visibly-off numbers
      from 3,798 to 3,677.
  - Six cells cross decision 0267's 13-digit bar into visibly off. All are in ungated probes-numeric cases with no
    baseline (decision 0203).
- **Decision:** accept all six cells as visibly off (decision 0267 item 6 needs a record for each). They fall into two
  groups, for different reasons.
  - **Excel is the noisy side.** Ours moved onto the math, and Excel is more than 1E-13 and a thousand units of
    |ln value|·2^-53 off it:

    | cell | Excel vs math | ours vs math (main -> branch) | owner |
    |---|---:|---:|---|
    | t4434-huge-shape-outside-large-shape-beta B45 `F.DIST.RT(1.08783836134,393,1449802)` | 1.2E-12 | 1.2E-12 -> 0 | T-4214 |
    | t4434-... C45 `F.DIST(1.08783836134,393,1449802,TRUE)` | 1.5E-13 | 1.5E-13 -> 1.2E-16 | T-4214 |
    | t4434-... D40 `BETA.DIST(0.000204472489243,136.028,726684,TRUE)` | 2.1E-13 | 2.4E-13 -> 0 | T-4214 |

  - **Excel sits at its double floor, not noise.** Excel is only 0.9 to 1.7 units of |ln tail|·2^-53 off the math,
    as close as a double kernel gets, but on its own rounding. At |ln tail| of about 600 that floor passes the
    13-digit bar.

    | cell | Excel vs math (units) | ours vs math, units (main -> branch) | owner |
    |---|---:|---:|---|
    | t3568-r5-bup-fresh A223 `BETA.DIST(0.75774435,2500,39.8,TRUE)` | 1.0E-13 (1.7) | 0.81 -> 0.075 | T-90001 |
    | t3568-r5-bup-tune A359 `BETA.DIST(0.76473,2489.3,10.3388,TRUE)` | 6.4E-14 (0.93) | 0.26 -> 0.013 | T-90001 |
    | t4434-... A295 `F.DIST.RT(27.1814481941,38,441015)` | 6.0E-14 (1.2) | 0.5 -> 0.003 | T-90001 (and T-3209) |

    Main's kernel there (Cephes' fraction) was not Excel's arithmetic; it only happened to sit nearer Excel. Recovering
    these cells needs a model of Excel's own arithmetic: L140 bup with 10 or more terms for the BETA.DIST cells, and
    bgrat after bup for the F cell. Keeping Cephes for these shapes would be the keep-matched branch decision 0188 R4
    forbids.

    The fidelity re-review ruled the trade permitted under decisions 0114 and 0188: over the case it is a net gain
    against Excel. Of the 322 t3568-r5-bup cells that move, 195 move toward Excel and 122 away; 177 eval cells
    improve their class and 19 worsen.

    These three get an active owner, T-90001. A223 and A359 (a about 2500) lie outside the a0 > 3000 scope of T-3871,
    and T-3871, T-3712 and T-3209 are all deferred (`defer: phase-1b`). T-90001 owns the three so that a cell first
    wrong at the 13th digit has an owner Phase 1 can pick (decision 0267 item 4), without reopening the deferred
    tasks' wider scope. A295 stays listed in T-3209 too.

- **Consequences:**
  - This record allows only the six cells above. Every other cell that loses an Excel near the math stays within the
    13-digit bar and is listed with its reason in T-4435's Outcome.
  - A model of Excel's bfrac (T-4214), or of its L140 bup and bgrat arithmetic (T-90001; T-3871, T-3712 and T-3209
    for their wider scope), is what would bring these cells back.
