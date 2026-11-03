---
id: T-3867
title: "BETA.INV's search on L140 bup+bgrat with n >= 2 bup terms: cross T-3561's cdf without moving gated H28"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3561]
---

## Goal
BETA.INV's search crosses BETA.DIST's cdf on bratio's L140 bup+bgrat branch with n >= 2 bup terms (and one term at
c = 1, b0 = 2), as T-3561 ported it for BETA.DIST, and keeps every gated cell. Today the search
(`bratio::beta_search_tails`) keeps the Cephes kernel there.

## Context
- T-3561 ported T-3568's n-term composition to BETA.DIST's cdf (`bratio/bup.rs` `l140_bup_bgrat`). When BETA.INV's
  search crossed that cdf too, gated ironcalc-calc STATISTICAL/BETADIST_BETAINV.xlsx Sheet1!H28
  (`BETA.INV(G28,6,1000,0.1,F28)`, Excel 0.12000000000000073) went from bit-exact to 1 ulp off
  (0.12000000000000072): a `make eval-check` regression. So the search was left on today's kernel there.
- On the same run the crossing lifted non-gated cells: probes-numeric t1745-beta-inv-roundtrip (7 visibly off to within
  4 ulp, 1 visibly off to within 15 digits, 5 within 15 digits to within 4 ulp, 5 within 4 ulp to exact, 2 exact to
  within 4 ulp) and m9-beta-inv (A103 visibly off to within 4 ulp, 3 within 4 ulp to exact). Those gains are what
  this task gets back.
- H28 is a landing question (where the search stops against its own crossing), as with H5 and H26 (T-3562,
  statistics.md "BETA.INV"). Read T-3562's findings first.

## Acceptance criteria
- [ ] `beta_search_tails` crosses the n-term cdf on L140 bup+bgrat (or on a stated, evidenced part of it), and H28
      stays bit-exact (`make eval-check`: 0 regressions).
- [ ] The Outcome lists the probes-numeric BETA.INV cells that move, main against HEAD, with none going from within
      15 digits to visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST's own cdf on the branch (T-3561), F, BINOM and NEGBINOM (T-3712), and H5 and H26 (T-3562).
