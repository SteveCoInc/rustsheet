---
id: T-4564
title: "BETA.DIST at n = 2 bup terms and T.DIST.RT: model where Excel's exact 0 starts (T-4437's remainder)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
BETA.DIST at n = 2 bup terms and T.DIST.RT give Excel's exact 0 wherever Excel does near a tail of 1E-290 to
1E-303, with each cut modelled as a mechanism, as T-4437 did for n >= 3.

## Context
- T-4437 (statistics.md "Where the exact 0 starts (T-4045)"): at n >= 3 bup terms L140 bup+bgrat's tail is 0 when
  bup's sum W is below 2^-970 (`bup.rs` `l140_bup_sum_flushed`). That rule does not hold at n = 2:
  - probes/t4437-beta-t-zero-bracket D (5E6, 2.5) and F (5E6, 3) are 0 at every row, at W 0.22 and 0.16 times
    2^-970 and below (consistent with the rule);
  - probes/t4437-beta-zero-fresh D (2E6, 2.5) is a number at every row, down to W 0.15 times 2^-970 (row 17).
  - probes/t4437-beta-zero-fresh2 (T-4437's second sweep, ln of the true tail by 0.05 nats around ln 2^-970):
    I (3E6, 2.5) is a number at every row, to 0.525 nats below the edge; J (7E6, 2.75) is a number to 0.225
    below it (J16) and 0 from 0.275 below (J17). So the n = 2 cut moves with a0: lower at smaller a0.
  - The density P is no separator either (0 at ln P - ln 2^-970 = +0.56, a number at -0.71). Probe n = 2 across
    a0 from 1E6 to 1E7 and below the walked range, to find what a0 changes.
- T.DIST.RT (kernel K(df/2, 1/2), small-shape branches, never L140): t4437-beta-t-zero-bracket G:I walk ln of the
  true tail by 0.5. Excel's last number and first 0 are at -687 / -687.5 (df 1E7, G15/G16), -692 / -692.5 (df 1E6,
  I25/I26) and -696.5 / -697 (df 1E5, H34/H35), with numbers as small as 3.3E-303. Neither x^a (ln -683, -688,
  -692.5 at the first 0) nor bgrat's prefactor crosses a common threshold there (T-4437's notes,
  docs/research/t4437). Ours is a number in every row.
- Owner of the visibly-off cells of probes/t4254-beta-t-l140-zero-cut E1:E11, F1:F11; t4254-beta-t-zero-wide F30;
  t4437-beta-t-zero-bracket D1:D37, F1:F37, G16:G37, H35:H37, I26:I37; t4456-beta-t-zero-cut E1:E9, F1:F9;
  t4456-beta-t-zero-reach E21 and F9 (the t's zeros T-4437 named; T-4577 owns F3:F5 there). Also owner of the numbers near the edge
  that differ from Excel in the 8th or 9th digit: t4437-beta-t-zero-bracket, t4437-beta-zero-fresh and
  t4437-beta-zero-fresh2, every NumericMismatch cell (and t4437-beta-zero-fresh2 J17:J22, Excel's n = 2
  zeros), and t4254-beta-t-zero-wide A29, B28, B29.

- BETADIST shares BETA.DIST's worksheet cdf, so it takes T-4437's n >= 3 rule unprobed: put a few BETADIST cells
  across the t4437-beta-zero-fresh2 edge in this task's next probe (T-4437's review).

## Acceptance criteria
- [ ] The cells above match Excel's 0 or number; the near-edge numbers reach the family's digits bar.
- [ ] Each cut is bracketed to about 0.5 nats and modelled from the mechanism (decision 0188 R4 guards: hold-out
      and fresh sweep), recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The n >= 3 rule (T-4437).
