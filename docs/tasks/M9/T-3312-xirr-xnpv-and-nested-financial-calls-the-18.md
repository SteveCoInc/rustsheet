---
id: T-3312
title: "XIRR, XNPV and nested financial calls: the 18 gen-m9 cells T-2611 left (numeric text in XIRR values, XNPV element order, inner-function causes)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The 18 gen-m9 known-failure lines T-2611 left give Excel's value: each one is traced to its cause, probed where the
order is unevidenced, and fixed here or moved to the task that owns the inner function.

## Context
Split from T-2611 (2026-10-30), which landed XIRR's guess-first order and the empty/numeric-text single-cell size checks
(financial.md "XIRR and XNPV under the gen-m9 statistical cells", T-2611 sentences). Remaining lines in
eval/baseline/gen-m9.txt (excel-graph, 2026-10-24), expected value in brackets:
- XIRR, top level:
  - 31474c20fc1dafd6 Calc!A351 `XIRR(Data!C9:C11,{39448,39508,39859},0.1)` (#NUM!, ours #VALUE!): C11 is the text "1".
    Text read as 1 (then all flows positive), skipped (then 2 values, 3 dates) or refused as `#NUM!` all fit; probe each
    reading separately (a numeric text in a range of mixed-sign flows with equal counts, and with one more date).
  - 30e20ba3302fea8b Calc!A208 `XIRR(-1.5,WEIBULL.DIST(0,1.5E+10,99999,FALSE))` (#NUM!, ours #N/A): ours computes the
    WEIBULL.DIST as 0; Excel's is probably `#NUM!` (99999^1.5E+10 overflows). Probe `WEIBULL.DIST(0,1.5E+10,99999,FALSE)`
    and hand it to functions-stat if so. Making two plain numbers `#NUM!` regressed 15 cells (b73c6cc96e093f03 A25,
    `XIRR(<number>,SLN(..))` is `#N/A`), so the XIRR order is not the cause.
- XNPV, top level: 1b60bf99d71424e7 Calc!A171 (#VALUE!: equal counts, text values C1:C2 and FALSE/empty dates C9:D9) and
  5ded7594e597e31e Calc!A12 `XNPV(1E+15,{-100;30;40;50},Data!D6:E7)` (#NUM!, a boolean in the dates): XNPV's element
  reading order of values and dates is unprobed.
- XIRR's text and boolean guess order (T-2611 review): ours reads a non-numeric text guess as `#VALUE!` before an error
  dates argument and the sizes, and a boolean guess as 1 or 0 that only skips the size checks; neither is evidenced.
  Probe `XIRR(<good values>,NA(),"x")`, `XIRR(<good values>,<other count>,TRUE)` and `XIRR(<good values>,NA(),TRUE)`
  (also an error values argument beside an error guess, and a numeric-text guess beside different counts).
- Nested (the inner call decides; trace each with a probe of the inner call):
  gen-m9/4bad303d32cc3f46 Calc!A364 (LOGNORM.INV over XIRR), gen-m9/530c636749a85209 Calc!A338 (HYPGEOM.DIST over IRR),
  gen-m9/58e3253bbd3a7a82 Calc!A189 (EXPONDIST over XNPV), gen-m9/5919660b7bb4d195 Calc!A144 (WEIBULL over XIRR),
  gen-m9/8f2ac25945af87da Calc!A137 (LOGNORMDIST over XNPV), gen-m9/a4090567758520df Calc!A277 (PEARSON over LOGEST over XNPV),
  gen-m9/b72d952c5f52cb4d Calc!A38 (ODDLYIELD over XIRR(MONTH(59085),Data!$B2)); the BETADIST-over-NPV cell of cde551ea787f713a (A37) is T-3335's (TEXTBEFORE).
- Values, not orders: 1b60bf99d71424e7 Calc!A181 (CUMPRINC, -90.53126715278742), 89f0aee953a55a45 Calc!A349
  (`FVSCHEDULE(5916,{-1E+300,1E+300})` is 0, ours `#DIV/0!`), 984cba67bbe6972b Calc!A141 (GAMMA.INV over FVSCHEDULE),
  a91bad4e84f49739 Calc!A158 (PPMT, -4.581304695604488e-303, an underflow).
Tool: a scratch suite holding one case's Data sheet with probe formulas in Calc runs through `rustsheet-eval --root <dir>
show p/x.case.json --all` (how T-2611 traced its cells).

## Acceptance criteria
- [x] The 18 lines above disappear from eval/baseline/gen-m9.txt, or each one left is listed in the Outcome with its
      traced cause and owner task; no new known-failure lines.
- [x] Every order changed is backed by a probe of the bare call and recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR/XNPV solver values.

## Outcome
Seven of the 18 lines had already passed on main (31474c20fc1dafd6 A351, 1b60bf99d71424e7 A171, 58e3253bbd3a7a82 A189,
a91bad4e84f49739 A158 and three others no longer in eval/baseline/gen-m9.txt). Two probe cases (excel-graph, 2026-11-04)
traced the rest: probes/t3312-xirr-xnpv-inner-calls (64 cells: each inner call on a copy of its case's Data sheet, and
the guess order) and probes/t3312-xirr-guess-dates-weibull (61 cells: guess at or below -1, dates read in order,
WEIBULL.DIST at x = 0).
- Fixed (6 gen-m9 lines): 30e20ba3302fea8b A208 (WEIBULL.DIST's density at x = 0 computes beta^alpha: `#NUM!` on
  overflow, `#DIV/0!` on underflow; statistics.md), 5ded7594e597e31e A12 and 8f2ac25945af87da A137 (XNPV: rate at or
  below 0 first, then a non-number first date `#VALUE!`, then value errors, then a non-number later date `#NUM!`;
  financial.md "XNPV's rate, first date and later dates (T-3312)"), 4bad303d32cc3f46 A364, 5919660b7bb4d195 A144 and
  b72d952c5f52cb4d A38 (XIRR: guess first, boolean guess `#VALUE!`, a guess at or below -1 turns the size errors into
  `#NUM!`, one empty cell is a number for the one-value `#N/A`, values and dates read pair by pair; financial.md "XIRR's
  guess, date and empty-cell order (T-3312)").
- Left, with their owners: 530c636749a85209 A338 is `IRR({-1,1E+15},143)` (Data!C2:D3 reads -1 and 1E+15), moved to
  T-3900's cell list; a4090567758520df A277 is T-2574's (LOGEST lift order, already listed there); 984cba67bbe6972b A141
  is GAMMA.INV's last digit (the FVSCHEDULE inside gives 36690 exactly), T-3239's; 1b60bf99d71424e7 A181 is CUMPRINC's
  last digit, T-1374's; 89f0aee953a55a45 A349 (`FVSCHEDULE(5916,{-1E+300,1E+300})`, 0 on excel-graph) cannot be probed
  on excel-graph (every smaller workbook with an overflowing FVSCHEDULE fails to open), so it goes to desktop Excel,
  T-4250 (M14, decision 0039).
- New visibly-off probe cell: probes/t3312-xirr-xnpv-inner-calls Calc!A30 (`XIRR({-100,200,-300},{1,2,3})` is
  2.98e-9 in Excel, a solver value), T-4251.
- compute_xirr's guess at or below -1 is `#NUM!`, not `#VALUE!` (t3312-xirr-guess-dates-weibull A9).
Eval delta: gen-m9 known failures 1937 -> 1931 (6 fixed, 0 regressions); probes +2 cases (124 of 125 cells pass, 1
new known failure owned by T-4251).

## Landing notes
- PR #3247 (draft), head is this commit's parent's successor (the commit that adds these notes). Round 1 reviews ran on
  8f478c998: security APPROVE, design APPROVE (MINOR: doc wrap, duplicated XNPV dates arms; both fixed in this push),
  excel-fidelity REQUEST_CHANGES (MAJOR: `xirr_flows` compared the raw date, so 2958465.5 became `#NUM!`; fixed by
  comparing `date.floor()`; MINORs: unprobed generalizations, now listed under "Unprobed" in the two financial.md
  entries). Round 2 has not run: re-run excel-fidelity and design on the new head.
- Blocking `make check`: the `*.math.json` sidecars of the two new probe cases are missing
  (`uv run rustsheet-accuracy math --suite probes --check` names t3312-xirr-xnpv-inner-calls A28, A30 and
  t3312-xirr-guess-dates-weibull A32). `rustsheet-accuracy math --suite probes` ran 60 minutes in this session without
  finishing (it has no per-case option) and was stopped; run it with a longer budget, commit only the two t3312
  `.math.json` files, and revert any rewrite it makes to unrelated sidecars (it rewrote three Bessel ones: `why` text only).
- Verified on this head: `rustsheet-eval check` 0 regressions in every suite (gen-m9 1931); engine tests pass.
- Next: generate the sidecars, push, re-run the two reviewers, mark the PR ready, enqueue.
