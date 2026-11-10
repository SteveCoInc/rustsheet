# 0341 — T-4435's accuracy change leaves three ungated probe cells visibly off where Excel is noise

- **Status:** accepted (2026-11-07, T-4435; recounted 2026-11-10 after the fidelity review). The title's "seven" was
  the first count; the file name keeps it.
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
- **Decision:** accept as visibly off only the three cells where ours moved onto the math and Excel is clearly the
  noisy side, more than 1E-13 and a thousand units of |ln value|·2^-53 off:

  | cell | Excel vs math | ours vs math (main -> branch) | owner |
  |---|---:|---:|---|
  | t4434-huge-shape-outside-large-shape-beta B45 `F.DIST.RT(1.08783836134,393,1449802)` | 1.2E-12 | 1.2E-12 -> 0 | T-4214 |
  | t4434-... C45 `F.DIST(1.08783836134,393,1449802,TRUE)` | 1.5E-13 | 1.5E-13 -> 1.2E-16 | T-4214 |
  | t4434-... D40 `BETA.DIST(0.000204472489243,136.028,726684,TRUE)` | 2.1E-13 | 2.4E-13 -> 0 | T-4214 |

- **Not accepted as Excel noise.** These two cells lost an Excel that sits at its double floor:
  - t3568-r5-bup-fresh A223 `BETA.DIST(0.75774435,2500,39.8,TRUE)`: Excel is 1.7 units off the math, ours went from
    0.81 to 0.075 units.
  - t3568-r5-bup-tune A359 `BETA.DIST(0.76473,2489.3,10.3388,TRUE)`: Excel is 0.93 units off, ours went from 0.26 to
    0.013.
  - At |ln tail| of about 600, a floor-level difference passes the 13-digit bar. Main's Cephes fraction was not
    Excel's arithmetic either; it happened to sit nearer.
  - No accurate kernel reproduces Excel's rounding there short of modelling its many-term bup (T-3871, deferred),
    and a branch that keeps Cephes for these shapes would be the keep-matched branch decision 0188 R4 forbids. So
    the cells stay visibly off and are owned by T-3871. T-4435's Outcome lists them and every other cell that moves
    away from an Excel near the math, with the reason.
- **Also listed, not accepted as noise:** t4434 A295 `F.DIST.RT(27.1814481941,38,441015)`. Excel is 1.2 units off the
  math, at its floor; ours went from 0.5 to 0.003 units, past the 13-digit bar. It is owned by T-3209 (Excel's bgrat
  digits).
- **Consequences:**
  - This record allows only the three cells above. Every other cell that crosses the bar, or loses an Excel near the
    math, is listed with its reason in T-4435's Outcome.
  - A model of Excel's bfrac (T-4214), bgrat (T-3209) or many-term bup (T-3871) arithmetic is what would bring these
    cells back.
