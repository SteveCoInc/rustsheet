---
id: T-4707
title: "BETA.DIST at (2E6, 10) and (2E6, 10.5) and T.DIST.RT at df 1E5 in tails near 1E-285: the 9 visibly-off t4254 numbers no task names (Excel 1.3E-13 off the math)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
campaign: visible
depends_on: [T-4463]
---

## Goal
The 9 deep-tail numbers below are Excel's value to the bar that applies: the incomplete-beta family's 13 digits for
BETA.DIST (decision 0267 item 2), 15 digits for T.DIST.RT. Where a cell cannot get there, the task records why. They
are the visibly-off cells of probes/t4254-beta-t-l140-zero-cut and probes/t4254-beta-t-zero-wide that no open task
names.

## Visible-parity stage
`visible`. Excel is the inaccurate one on the BETA.DIST cells: it is 1.6E-14 to 1.4E-13 off the 50-digit math, and
ours is 5E-16 to 9E-14 off it (eval math sidecar, checked with mpmath `betainc` at 60 digits on the double arguments).
On the T.DIST.RT cells, Excel and ours are both about 3E-12 off the math, in the same direction, and 4E-14 to 1.4E-13
apart (see Context). Reaching these cells means reproducing Excel's arithmetic, not a more accurate kernel. Target:
the 5 BETA.DIST cells within half a unit of Excel's 13th significant digit, and the 4 T.DIST.RT cells within half a unit
of its 15th. Time-boxed (decision 0056). If the time-box runs out, the cells it still lists are accepted, and no
follow-up is filed for them (decision 0267 item 4).

## Context
- Gap triage 2026-11-06 (`make eval` on agent/functions-lookup/T-4570-xlookup-ref-element, based on origin/main
  5aed684f1). I checked every failing cell of report.json against open tasks with
  `scripts/exit_owners.py`'s `owned` rule. All of the report's counted visibly-off cells have an owner. These two
  probes are a gap: T-4564 owns only their zeros and near-edge 8th/9th-digit numbers (l140-zero-cut E1:E11, F1:F11;
  zero-wide A29, B28, B29, F30). No task names the other failing cells. This task owns the visibly-off ones, and
  T-4708 owns the rest.
- Decision 0267 cells. Tier B: the BETA.DIST cells, first wrong at the 13th digit. Tier C: the T.DIST.RT cells, first
  wrong at the 14th digit, worked here under item 4 because this task owns Tier B cells. They are Phase 1b evidence
  under decision 0188 R1(c), because Excel and ours are both within 1E-9 of the math, so the M9 exit row does not count
  them. They do count in the 15-digit headline. Excel vs ours (`make eval-show CASE=...`):
  - probes/t4254-beta-t-l140-zero-cut.case.json:
    - A2 `BETA.DIST(0.999648862690922,2000000,10.5,TRUE)`: Excel 8.827616059802568E-285, ours 8.827616059803741E-285
      (math 8.8276160598037375E-285).
    - A7 `BETA.DIST(0.999646361798262,2000000,10.5,TRUE)`: Excel 6.339701457651208E-287, ours 6.339701457652083E-287.
    - A9 `BETA.DIST(0.999645361446463,2000000,10.5,TRUE)`: Excel 8.800435701638453E-288, ours 8.800435701637005E-288.
    - C6 `BETA.DIST(0.999645280690786,2000000,10,TRUE)`: Excel 8.791832158301332E-289, ours 8.791832158301955E-289.
    - C11 `BETA.DIST(0.999642778066738,2000000,10,TRUE)`: Excel 6.2660562731917494E-291, ours 6.266056273192754E-291.
  - probes/t4254-beta-t-zero-wide.case.json, T.DIST.RT at df 1E5:
    - E27 `T.DIST.RT(36.3265708325879,100000)`: Excel 2.3208225941828692E-287, ours 2.3208225941832032E-287.
    - E28 `T.DIST.RT(36.6042414372482,100000)`: Excel 1.0536518276677526E-291, ours 1.0536518276676718E-291.
    - E29 `T.DIST.RT(36.8798794622512,100000)`: Excel 4.783571897032202E-296, ours 4.783571897032386E-296.
    - E30 `T.DIST.RT(37.1535300690244,100000)`: Excel 2.1717382813834264E-300, ours 2.17173828138357E-300.
- Suspected causes. Neither is traced yet:
  - BETA.DIST: (2E6, 10) and (2E6, 10.5) are L140 bup+bgrat at about 10 bup terms, past `bup.rs`'s n <= 9 port
    (MAX_BUP_TERMS 9), so the fallback kernel computes them. T-4463 works that same fallback on the t4456 cells at
    n = 25 and 29, and T-4577's Context notes that the n >= 10 residue grows with n. Hence `depends_on: [T-4463]`.
    Re-measure these cells against its model first: they may pass, or they become its first hold-out at n of about 10.
    Ours is 2E-14 to 9E-14 off the math at A9, C6 and C11 too, so part of the gap there is on our side.
  - T.DIST.RT: the kernel is K(df/2, 1/2) through `t_kernel_tails` (bgrat at b = 1/2), as at T-4463's t4456 E17:E20
    and F6:F8. Both values are about 3E-12 off the exact-t math. That size is what rounding x = df/(df+t^2) to a
    double gives: the tail goes as x^(df/2), so a relative error of 1.1E-16 in x becomes df/2 * 1.1E-16, about 5.5E-12,
    in the tail. So Excel and ours appear to share the same x. Only the 1E-13 left over between them is the kernel's.
    Confirm this with a 50-digit value taken at the rounded x before fitting anything.
- docs/excel-semantics/statistics.md, "Where the exact 0 starts (T-4045)", the T-4254 bullets (probe design, cut
  rules). It now records that Excel's numbers at (2E6, 10.5) and (2E6, 10) are up to 1.4E-13 off the math.

## Acceptance criteria
- [ ] Each cell above meets its bar of Excel (13 digits for BETA.DIST, 15 for T.DIST.RT), or statistics.md records
      it with its Excel, ours and 50-digit values and the reason it stays. These known-failure lines disappear from
      eval/baseline/probes.txt with no new ones (any family regression to the 14th digit only under decision 0267
      item 6, checked with `scripts/family_net.py`).
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), for any constant, cut, selector or formulation chosen
      by scoring:
  - hold-out counts (exact / 4 ulp / 15 digits) on a hold-out split fixed before scoring, for the chosen candidate
    and the correctly rounded one;
  - a fresh sweep, drawn after the formulation is frozen, is scored. Use an excel-probe case near (2E6, 10.5), at
    n of about 10 to 15, and near T.DIST.RT df 1E4 to 1E6 at tails of 1E-280 to 1E-300. Since this task has a
    Decision 0267 cells list, decision 0267 item 5 allows the sweep in probes-numeric;
  - the Gate A envelope result is in the Outcome (within 15 digits of Excel, or within 1E-9 of the 50-digit value),
    listing every point outside, and `rustsheet-eval accuracy` reports 0 failing;
  - the code carries a `FITTED` tag with the fitted range (none for a structural model, decision 0188 R4 (c));
  - the reproducibility floor is met;
  - there is no branch that keeps already-matched cells by special case.
- [ ] No gated or probes-numeric cell goes from within 15 digits to visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The zeros and the 8th/9th-digit near-edge numbers of these cases (T-4564).
- The cells of these cases already within their bar (T-4708).
- The t4456 cells (T-4463, T-4577).
