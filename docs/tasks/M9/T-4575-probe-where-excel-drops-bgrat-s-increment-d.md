---
id: T-4575
title: "Probe where Excel drops bgrat's increment D = x0^a0 from BINOM.DIST's cdf on L140 bup+bgrat at an integer b0 (ln D -595 to -708)"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph probe case in the `probes` suite brackets, to about 0.25 nat of ln D, where Excel stops adding bgrat's
increment D = I_x0(a0, 1) = x0^a0 to BINOM.DIST's cdf on L140 bup+bgrat at an integer b0 (c = 1), for several bup term
counts and first shapes. With these values T-4576 can model the cut as a mechanism instead of guessing from one cell.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main aabdd668c). `make unowned-off ALL=1` leaves one visibly-off cell,
  owned only by deferred T-3712: probes-numeric/t3568-r5-bup-hold Calc!A1371 `=BINOM.DIST(2,1925,0.297025813,TRUE)`,
  Excel 7.562827113889096E-290, ours 7.562873290726655E-290. It is decision 0267 Tier A: first wrong at the 6th digit.
- mpmath at 50 digits (the finite sum): 7.56287329072722E-290. Ours is 7.5E-14 relative off it. Excel is 6.1E-6
  off, but Excel equals the sum less x0^a0 = q^1923 = 4.6177E-295 to 1.3E-13. So Excel's value is ret·S without D.
  The oriented call is I_q(1923, 3), q = 1 - p = 0.70297: b0 = 3, so n = 2 bup terms and c = 1. That is L140
  bup+bgrat (statistics.md "Incomplete beta: the integrated model", bullet "L140 bup+bgrat with n >= 2 bup terms").
  At c = 1, D is x0^a0.
- Excel's own D is a number on that row: B1371 `BETA.DIST(x0,1923,1,TRUE)` is 4.617683716137408E-295. So the drop
  happens inside the composition. It is not D underflowing as a worksheet value.
- Same-branch Excel cells that keep D (Excel matches the full sum, and dropping D would be visibly off):
  - t3568-r5-bup-hold A1326 `BINOM.DIST(4,1807,0.2811393551,TRUE)`: ln D -595.1, D/total 3.6E-10.
  - t3568-r5-bup-tune A1263 `BINOM.DIST(2,1963,0.2202032,TRUE)`: ln D -487.7, D/total 1.1E-5.
  
  So on this branch the cut lies in ln D (-677.7, -595.1). It is not a ratio rule, since 1.1E-5 is kept and 6.1E-6
  dropped. Off the branch, t2145-binomial-sweep A3242 `BINOM.DIST(2,100,0.999,TRUE)` keeps D at ln D -677.0, but there
  x0 = 0.001 and the path differs. This was found by scanning every literal BINOM.DIST cdf cell of probes,
  probes-numeric and gen-m9 against mpmath.
- Related cuts, kept apart:
  - T-4045's L140 prefactor rule zeroes the whole value from ln ret -709. Here ln ret is about -665.
  - T-4437 brackets the exact 0 at n = 3 bup terms with c = 1/2. There D (5E6, 10.5) is a number at ln D -697.
  - So the c = 1 cut here sits earlier than either.
- Probe freeze (decision 0267 item 5) binds probes-numeric only. This case goes in `probes`, like T-4254's, and serves
  T-4576's "Decision 0267 cells".

## Acceptance criteria
- [ ] A seeded generator under docs/research/t4575/ writes one `probes` case `t4575-binom-l140-d-drop` (the
      `excel-probe` skill, excel-graph fill, oracle sidecar and `.math.json` committed, as for t4254). Each row has
      three cells at one (k, a0, x0):
  - A `=BINOM.DIST(k, a0 + k, p, TRUE)`, with p a 15-significant-digit literal;
  - B `=BETA.DIST(x0, a0, k + 1, TRUE)`, the same oriented call through BETA.DIST, with x0 = fl(1 - p) written
    exactly in t3568's split form `(m1*2^-26+m2*2^-52)`;
  - C `=BETA.DIST(x0, a0, 1, TRUE)`, Excel's D as a worksheet value.
- [ ] Rows cover:
  - k in {1, 2, 4, 9}, so b0 = 2, 3, 5 and 10 and n = 1, 2, 4 and 9 bup terms;
  - a0 in {2500, 1E4, 1E6}, so x0 > 0.7 everywhere;
  - target ln D = a0·ln x0 from -600 to -708 in steps of 4, plus -680 to -676 in steps of 0.25. p is solved from the
    target and then rounded to its literal.
  - The two t3568 anchors above are repeated: A1371's row, and A1326's row.
- [ ] Each row's ln D, ln ret and D/total (from the `.math.json`) and whether Excel's A and B equal the sum with or
      without D are tabulated in docs/research/t4575/RESULTS.md. The cut found for each (k, a0) is recorded in
      statistics.md under the L140 n >= 2 bullet, citing cells.
- [ ] Every cell where ours and Excel disagree is listed in a cell-list line of T-4576, or of another open task named
      in this Outcome. The PR records the new case's failures with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-4576).
- Non-integer b0 (c = 1/2) and the exact 0 (T-4437, T-4045).
