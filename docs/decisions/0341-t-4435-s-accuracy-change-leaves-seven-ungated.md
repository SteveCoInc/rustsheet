# 0341 — T-4435's accuracy change leaves seven ungated probe cells visibly off where Excel is noise

- **Status:** accepted (2026-11-07, T-4435).
- **Context:**
  - T-4435 is decision 0114's `accuracy` stage. It moves the incomplete beta at one huge shape outside
    `large_shape_beta`'s old bounds onto the math:
    - region 1 is b0 < 40 past L140's 9 bup terms, at a0 from 2000 to 1E6;
    - region 2 is bfrac at b0 of 40 to 200 past T-3571's shapes (a0 above 5000).
  - On its grid (docs/research/t4435, 800 points, 60-digit mpmath) the worst error falls from 5.2E-8 to 1.7E-13, and
    no point gets worse.
  - `make eval` against main (same suites; docs/research/t4435/moved_cells.py) shows:
    - 890 cells move: 188 to a better closeness class, 18 to a worse one;
    - no gated suite regresses, and the visible-parity lock holds;
    - Gate A has 0 failing and the same 188 known failures;
    - the incomplete-beta family's cells within 15 digits rise from 361,502 to 361,553, and its visibly-off numbers
      fall from 2,494 to 2,372 (decision 0267 item 6 (c)'s counts, taken from `functions[].closeness` because
      `scripts/family_net.py` stops on the `r3_math` bucket).
  - Seven of the 18 cross decision 0267's 13-digit bar, from `residue13` to visibly off. All are in ungated
    probes-numeric cases with no baseline (decision 0203), so no gate fails. Decision 0267 item 6 covers only
    regressions that stay within 13 digits, so this record says why the seven are accepted (decision 0263's
    precedent).

    | cell | Excel vs math | ours vs math (main -> branch) | owner |
    |---|---:|---:|---|
    | t4434-huge-shape-outside-large-shape-beta A295 `F.DIST.RT(27.1814481941,38,441015)` | 6.0E-14 | 2.5E-14 -> 1.6E-16 | T-3209 |
    | t4434-... B45 `F.DIST.RT(1.08783836134,393,1449802)` | 1.2E-12 | 1.2E-12 -> 0 | T-4214 |
    | t4434-... C45 `F.DIST(1.08783836134,393,1449802,TRUE)` | 1.5E-13 | 1.5E-13 -> 1.2E-16 | T-4214 |
    | t4434-... D40 `BETA.DIST(0.000204472489243,136.028,726684,TRUE)` | 2.1E-13 | 2.4E-13 -> 1.2E-15 | T-4214 |
    | t3568-r5-bup-fresh A223 `BETA.DIST(0.75774435,2500,39.8,TRUE)` | 1.0E-13 | 5.0E-14 -> 3.1E-14 | T-3871 |
    | t3568-r5-bup-hold B502 `BETA.DIST(0.765049,2150,...,TRUE)` | 3.8E-14 | 9.5E-14 -> 4.3E-14 | T-3871 |
    | t3568-r5-bup-tune A359 `BETA.DIST(0.76473,2489.3,10.3388,TRUE)` | 6.4E-14 | 1.8E-14 -> 5.9E-14 | T-3871 |

  - In the four t4434 cells, ours goes onto the math and Excel is the side off by 6E-14 to 1.2E-12. Main matched Excel
    only because both took the same inaccurate bfrac or Cephes path. T-4434's sweep lists them in T-4214 (bfrac
    digits) and T-3209 (bgrat digits).
  - The three t3568 cells are deep lower tails at a of 2100 to 2500. Both sides are 3E-14 to 1E-13 off the math in
    different directions. Over all 321 t3568-r5-bup cells that move, 216 get closer to the math and 104 farther. The
    worst error falls from 2.3E-13 to 6.2E-14, the expansion's own floor there (statistics.md, T-4435 item). A359 is the
    one of the seven that moves away from the math, by 4E-14.
- **Options considered:**
  - *Keep Cephes (or bfrac) on cells where it matched Excel.* That is a keep-matched branch (decisions 0056, 0188 R4):
    the dispatch would be chosen by the cells that already match, and it loses on the grid and on t3568 as a whole.
  - *Raise MIN_SHAPE above 3000 to leave T-3568's probes alone.* That is a cut picked to avoid the moved cells, not
    from the math. The boundary grid sets the bound: from a = 2000 the expansion is at least as accurate as the old
    kernels at every b.
- **Decision:** accept the seven cells as visibly off. They are owned: T-4214 for bfrac at b0 >= 40, T-3209
  (deferred) for F's bgrat digits at b0 < 40, and T-3871 (deferred) for BETA.DIST on L140 past T-3561's caps, which
  names the t3568-r5-bup cases. No baseline line is written, because their suites have none.
- **Consequences:**
  - T-4435's Outcome lists the seven cells and cites this record.
  - A model of Excel's bfrac (T-4214), bgrat (T-3209) or many-term bup (T-3871) arithmetic is what would bring them
    back.
  - This record allows only these seven cells. Another cell that crosses the 13-digit bar needs its own analysis.
