---
id: T-4220
title: "BETA.DIST at a huge first shape and b = 40 below the mean: Excel is 5E-10 to 4.6E-8 off the math (26 Tier A t4080 cells owned only by deferred T-3209)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 26 visibly-off b = 40 cells of probes-numeric/t4080-beta-num-band-edge are within half a unit of Excel's 13th
significant digit (decision 0267's bar for the incomplete-beta family), from a model of what Excel's bratio does on
that branch rather than a correction fitted to the cells. If two attempts do not find that model, the task blocks as
time-boxed and decision 0267 item 4 accepts the cells.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 0516b4895; `make unowned-off ALL=1`): these 26 cells are the only
  Tier A cells in the report with no live owner. They show as `BETA.DIST (deferred)` because the only task that lists
  them is T-3209, which is `defer: phase-1b` and is about 15th-digit bgrat noise. Its own rule says that a point
  outside the envelope gets its own task without the deferral key.
- Cells: probes-numeric/t4080-beta-num-band-edge.case.json Calc!A21:B21, A42:B42, B63, A84:B84, A105:B105,
  A126:B126, B147, A168:B168, A189:B189, A210:B210, A231:B231, A252:B252, A273:B273, A294:B294.
  - All are `BETA.DIST(x,a,40,TRUE)` for a = 2E9 to 1E10. x is 1 - q/a, chosen for I of about 1E-6 (column A) or about
    0.05 (column B).
  - A63 and A147 pass, at 4.5E-10 and 6.4E-10 relative.
  - Column C, at I of about 0.5, agrees with Excel to 1.6E-15 relative or better in every row (1 to 7 ulp).
- Reference: 60-digit mpmath `betainc(a, 40, 0, x)` at the double arguments, computed at triage and not committed.
  - Ours is the math to within 9E-15 relative at all 28 A/B cells.
  - Excel is off by 4.5E-10 to 4.6E-8 relative, with no consistent sign. Examples:
    - A21 `BETA.DIST(0.999999961229869,2E9,40,TRUE)`: Excel 9.999984224817014E-7, ours 9.999984252839257E-7, math
      9.99998425283923E-7 (Excel -2.8E-9).
    - B294 `BETA.DIST(0.999999994906026,1E10,40,TRUE)`: Excel 0.049999949775626405, ours 0.049999952069376706, math
      0.04999995206937666 (Excel -4.6E-8).
    - B231 (a = 6E9): Excel -1.7E-8. A231: -1.1E-8.
  - Each cell is first wrong at the 8th to 10th significant digit, so it is Tier A under decision 0267 item 1.
  - Excel's error is at least 1e-9 off the math on most of the cells, with no R3 mark (G = 1e-4 is not reached), so
    decision 0188 R1(c) counts them in M9's probe row. Excel is the target (prime directive 1).
- The branch these cells take. Read this from the code first, then confirm it on the cells.
  - TOMS 708 `bratio`, with a0 = a, b0 = 40 and lambda = (a + b)·y - b > 0 (no swap). b0 >= 40 leaves L140; a0 > b0
    and b0 <= 100 then go to L110 → L120, which is `bfrac`.
  - Column C has lambda < 0, so bratio swaps the shapes and also takes bfrac, with a0 = 40. Excel agrees with ours there.
    So whatever is noisy depends on the unswapped orientation.
  - Ours: `incomplete_beta.rs` `beta_tails` takes `large_shape_lower` (`large_shape_beta.rs`, accurate arithmetic)
    at a >= `MIN_SHAPE` whenever it applies. The ported `bratio` bfrac (`bratio/bfrac.rs`, plus the Lentz variant
    `bfrac_lentz` and its probed region) is not reached here. Confirm which one runs.
- Candidate mechanisms. These are hypotheses from the size of the error, which is about 1e-8 at a = 1E10 (between
  eps·sqrt(a) and eps·a), and none has been tested:
  1. lambda formed from x, as a - (a + b)·x, instead of from y. The cancellation would leave an absolute error of
     about ulp(1E10).
  2. `brcomp`'s x^a y^b with a·LN(x) taken through Excel's own LN of x near 1, not `rlog1` or `log1p`.
  3. bfrac (or `bfrac_lentz`) stopping early at a0 of 1E9 or more, at the 15·eps tolerance.
  4. A rounding inside `bcorr` or `brcomp`'s const·sqrt(b·x0) at a huge a.

  Test each one on the 26 cells and on column C, which must keep its agreement. Use `docs/research/t2464/r2-bratio/bratio_excel.py`
  `kernel` and T-2510's models.
- Related cells, owned elsewhere: T-4044 (`#NUM!` band, semantics) lists t4030 A39, A40, A61 and A62
  (`F.DIST.RT(2,80|100,1E10|5E9)`, the same L110 bfrac branch at b0 = 40 and 50).
  - At those cells Excel is within 9E-10 to 7E-9 of the math at the exact F argument df2/(df2 + df1·x), and ours is
    within 2.5E-9 of the math at the double-rounded x.
  - So the F side has a different first-order cause: which form of the beta argument the F transform hands bratio.
  - Check any model this task lands on those 4 cells too, but they do not gate this task.
- docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape", the T-4080 edge paragraph: the T-4220
  note there records the quirk and these numbers.
- Decision 0056's time-box and decision 0267 item 4 apply.

## Acceptance criteria
- [ ] The 26 cells listed above are within half a unit of Excel's 13th significant digit. The Outcome lists each
      cell's before and after relative error against Excel.
- [ ] Column C of t4080 (C21 to C294) and the band's `#NUM!` cells keep their verdicts. A63 and A147 do not get worse.
- [ ] No other cell of t4080, t4030, t4017-* or t2528-f-dist-huge-df2 moves away from Excel (`rustsheet-eval show`
      before and after). `make eval-baseline` reports no regression, or only what decision 0267 item 6 allows, with
      its (a) to (d) shown.
- [ ] The model is a structural choice (reviewer-excel-fidelity check 7 (c)), or a fit (7 (a)) if any constant or cut
      is scored against these cells. The Outcome or the PR shows the decision 0188 R4 guards that apply:
  - **Hold-out:** train and hold-out counts side by side for the chosen candidate and for the correctly rounded
    value (bit-exact, within 4 ulp, within 13 and within 15 digits). Use the fit lab split (T-3360) or a manual split
    of about 30% by cell hash, fixed with its margin before any scoring. The chosen candidate beats the correctly
    rounded value on hold-out.
  - **Fresh sweep:** after the model is frozen, a new oracle-filled probes-numeric case, generated after the
    freeze. Decision 0267 item 5 allows it only for this task's cells' region, so keep it to the region. Suggested
    region: log-uniform a over 1E9 to 1E10, b = 40 and a few b in (40, 100], tails I of 1E-8 to 0.3 with
    lambda > 0, plus column-C controls. Record its result in the Outcome.
  - **Gate A envelope:** the fresh sweep's count inside R2's envelope, against mpmath values the task attaches, with
    every point outside the envelope listed.
  - **`FITTED` tag** (fits only): `FITTED(T-4220, n_cells, range)` on each fitted piece. The code applies the piece
    only inside that range, with accurate math (today's `large_shape_lower`) outside it.
  - **Reproducibility floor:** no step targets a difference below the oracle's reproducibility floor (4 ulp web
    against desktop, or bits that vary between fills).
  - **No keep-matched branch:** no band or special case whose domain is defined by the cells that already match.
- [ ] statistics.md states the mechanism with its evidence, or records it as Excel noise that was tried and not
      pinned. In the second case the task is blocked as time-boxed under decision 0267 item 4.
- [ ] T-3209's line for these cells points here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` band at b < 40 (T-4044). Excel's 15th-digit bgrat noise at b < 40 (T-3209, deferred). The F transform's
  argument form at huge df2 (T-4044's value cells, T-3754, T-4053).

## Visible-parity stage
Decision 0083 and 0114 (`visible`): Excel is the inaccurate one here, so this means reproducing Excel's algorithm, not
an accurate kernel. Done when every cell this task owns is within the family's 13-digit bar (decision 0267 item 2).

## Decision 0267 cells
Tier A: each cell first differs from Excel at the 8th to 10th significant digit. It owns the 26 t4080 cells listed in
Context. This task is not in item 4's owner list because the cells came after it, from T-4080's probe on 2026-11-04.
The decision 0056 time-box applies: if two attempts do not move the cells, block the task as time-boxed, and item 4
accepts the cells that are left. This section is what lets item 5 allow this task's fresh-sweep case.
