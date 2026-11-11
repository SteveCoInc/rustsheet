---
id: T-1896
title: "HYPGEOM.DIST's cdf gives Excel's value at a large population past T-1873's term cap"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1873]
---

## Goal
HYPGEOM.DIST's cdf gives Excel's value where T-1873's term cap makes it `#NUM!` (a wide distribution at a population
above about 1.84E8, or a sum of more than 2^18 terms at a population above 2^52), within the lift weight T-1873 set.

## Context
T-1873 ported statrs' `Hypergeometric::cdf` (`statistical/hypegeom.rs`) with a per-call cap: it skips the terms that
are exactly 0 and sums at most 2^18 of the rest, statrs' double bit for bit within the cap, `#NUM!` past it
(statistics.md "HYPGEOM.DIST: per-call cap and lifting", a DEVIATION). No suite case is near the cap (the largest
sums 501 terms), so Excel's values there are unprobed. Main's values past the cap:
`HYPGEOM.DIST(1074741824,2147483648,2147483648,4294967296,TRUE)` 1.0000065293398746 (191 s in release; a cdf above 1,
so statrs' terms lose digits at that size), and `HYPGEOM.DIST(524288,1048576,2^59,2^60,TRUE)` `#NUM!` (its sum
overflows). Probe first (excel-probe skill, one milestone tag): a few wide distributions at populations of 1E9 to
1E15 and x around the mode, then pick a bounded formulation that matches (a normal or Edgeworth approximation for a
large spread, or a cdf ratio recurrence from the mode, kept only where it is bit-identical inside the cap).
- probes-numeric/t1896-hypgeom-cdf-wide was split into 5 parts (#4125). On 2026-11-11 excel-graph filled
  t1896-hypgeom-cdf-wide-p-02 (P rows 8-13). Owns: probes-numeric/t1896-hypgeom-cdf-wide-p-02 (every visibly-off
  cell): P!E8, E9, E10 and E13 are 0.97725.., 0.50002.., 0.022749.. and 1 in Excel, `#NUM!` in ours (past the cap).
- Refused (Graph's `calculate`, retries exhausted): t1896-hypgeom-cdf-wide-p-01, -rare-light (twice each),
  -rare-01 and -rare-02 (once). They stay pending as an oracle limit, not as Excel values. rare-light holds only two
  cdf cells (Rare rows 9 and 12), so a single cdf there is past what Graph's calculate answers; narrow with
  `split_cases.py --max-rows 1`.

## Acceptance criteria
- [ ] A probe case holds Excel's values for HYPGEOM.DIST's cdf at large populations past the cap.
- [ ] Those cells pass, or are known failures with a documented residue; the values inside the cap, probes-numeric and
  ironcalc-calc are unchanged; the call stays bounded at the weight in kinds.rs (`HYPGEOM_DIST_WEIGHT`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit residues of small arguments (T-0977).
