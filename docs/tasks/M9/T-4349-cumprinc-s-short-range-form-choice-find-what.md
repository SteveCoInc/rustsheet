---
id: T-4349
title: "CUMPRINC's short-range form choice: find what decides it where the leading exponent does not (T-3524 follow-up)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3524]
---

## Goal
CUMPRINC and CUMIPMT over a short range (at most 32 periods) choose between the geometric sum and the per-period
sum or closed form by a rule that generalizes to fresh cells, or `GEOMETRIC_MIN_LEAD` is shown to be the best
available rule on a fresh sweep.

## Context
- T-3524 re-validated `GEOMETRIC_MIN_LEAD` (0.3, functions/financial/cumulative.rs) on a fresh sweep
  (probes-numeric/t3524-cumprinc-lead-fresh-sweep) and on the old cells (fit lab `cumulative`, `cumulative-fresh`;
  builds from docs/research/t3524/*.patch). Without the cut (the geometric sum at every leading exponent) the engine
  is one test cell better on the old cells (147 vs 146 bit-exact) and 6 better on the fresh sweep (514 vs 508), but
  38 old cells go visibly off (tiny-rate interests: `CUMIPMT(1E-10,296,197540,13,19,1)`, gen-m9 b72d952c5f52cb4d
  A257, 33,554,432 ulps; probes/fin-cumipmt-tiny-rate-structure A1, A3, K3; fin-cumipmt-gate-edges A8, E2)
  against 20 it fixes, and on the fresh sweep 9 lost against 15 won (short ranges). So the leading exponent alone does not decide which
  form is Excel's over a short range: below 0.095 the old cells favour the cut and the fresh sweep the geometric
  sum (fresh 46 vs 57 bit-exact), from 0.095 to 0.3 both favour the cut. The rate (cells below 1E-3 dominate the
  visible losses) or the type are candidate deciders. `docs/research/t3524/analyze.py` lists each cell either form
  wins, with its lead and ulps.
- Owns the visibly-off cells of probes-numeric/t3524-cumprinc-lead-fresh-sweep after T-3524 (61, nearly all
  CUMIPMT, whose interest carries an ulp of the principal and of PMT as hundreds of its own): over a short range
  S!B3, B9, B10, B19, B37, B45, B52, B54, B70, B71, B76, B77, B79, B85, B86, B101, B104, B106, B110, B119, B126, B128,
  B131, B132, B133, B148, B152, B154, B170, B172, B175, B177, B186, B190, B197, B206, B210, B214, B223, B226, B228,
  B233, B234, B241; over a long range S!A404, B258, B275, B287, B336, B346, B379, B385, B401, B415, B432, B434, B439,
  B456, B464, B497, B500. A cell this task's rule does not reach goes to a follow-up it files.
- T-3524's second sweep, rates 1E-12..1E-4 (probes-numeric/t3524-cumprinc-tiny-rate-sweep, fit `cumulative-tiny`),
  also favours the geometric sum over a short range (265 vs 237 of 500 to the bit; below a lead of 0.095, 99 vs 63),
  while the old tiny-rate cells favour the cut: the rate, the type and the range's length are the candidate deciders.
  This task also owns that sweep's visibly-off cells (140, all but one CUMIPMT, whose interest carries an ulp of the
  principal or of PMT as thousands of its own): over a short range S!B4, B5, B9, B10, B11, B12, B15, B17, B26, B32,
  B33, B36, B37, B39, B50, B52, B53, B54, B55, B59, B60, B69, B70, B78, B81, B85, B89, B90, B91, B98, B100, B101,
  B102, B108, B109, B111, B115, B121, B122, B126, B127, B131, B139, B140, B141, B142, B144, B150, B154, B160, B161,
  B162, B163, B164, B166, B169, B186, B193, B195, B197, B205, B209, B215, B218, B220, B224, B226, B236, B242, B243,
  B247; over a long range S!A476, B251, B256, B266, B267, B270, B271, B284, B286, B290, B292, B295, B297, B299, B301,
  B309, B312, B319, B323, B331, B332, B333, B336, B338, B340, B347, B349, B350, B351, B353, B356, B357, B358, B374,
  B376, B377, B378, B393, B395, B396, B410, B412, B420, B426, B428, B429, B435, B437, B446, B452, B453, B457, B462,
  B464, B465, B470, B474, B475, B480, B482, B486, B487, B488, B490, B493, B494, B496, B499, B500.
- A new cut or selector is a fit: decision 0188 R4 (hold-out split, a fresh sweep drawn after the freeze).
- Decision 0295 keeps the cut against R4.3 for now: removing it (measured in T-3524's round 3) regresses probes 171
  cells and fixes 158, regresses gen-m9 b72d952c5f52cb4d A257, d912671baa1364aa A300, e168be7f6d3f84c2 A379 and
  ironcalc-calc `CUMPRINC_CUMIPMT.xlsx` G6, H6, G22, H22, and puts 21 more gated cells visibly off (8 cases), the
  same number it brings back on the two fresh sweeps. Below a lead of 1E-3 only old cells back the cut (the fresh
  sweeps start there). This task re-decides it: replace the cut by a decider that wins, or, if none does, ask the
  owner to rule between decision 0083's order and R4.3.
- The cut's `FITTED` range (T-3524) spans the old probes' rates 1E-300..1E300, so in practice it bounds only the
  range length (at most 32 periods): whichever way this task decides, tighten that range to what is evidenced or
  remove the tag with the cut (T-3524 fidelity review, round 3).

## Acceptance criteria
- [x] A ranked comparison (fit lab, two or more engine builds) of candidate short-range selectors on the old and the
      T-3524 cells, with hold-out counts, and a fresh sweep drawn after the choice is frozen.
- [x] The chosen rule beats `GEOMETRIC_MIN_LEAD` on test and on the fresh sweep with no visibly-off regression
      beyond decision 0083's allowance, or the Outcome shows no candidate does and the cut stays.
- [x] `make check` green; reviewers approve.

## Out of scope
- Long ranges (T-3524 removed their cut); type 0's underflowed principal (T-3141).

## Outcome
No decider replaces `GEOMETRIC_MIN_LEAD`; a fitted band below it now takes the geometric sum (decision 0296).
- Method (docs/research/t4349): the engine built with the cut at 0 beside today's engine gives each cell's value under
  both short-range forms (`dump.py`), so every selector between them is scored without another build; the live
  build reproduced the simulated counts exactly. Over the 923 old and T-3524 cells where the forms differ, each is
  bit-exact on only 265 and 286: neither is Excel's algorithm there (T-4390 owns finding it).
- Ranked comparison (`select.py`, decision 0188 R4.1): selectors of one to three conditions on the type, rate, lead
  and period count, dropping any that put a train cell newly visibly off, ranked on the train side of fit
  `cumulative` (the fit lab's fixed split). Frozen first: at most 4 periods, rate at least 1E-3, lead below 0.095
  (`short_takes_geometric`, one `FITTED(T-4349, 4038 cells, ...)` tag; quality/fitted.json 1 to 2 for cumulative.rs).
  A single condition gained at most 6 on train (`lead<0.001`, which lost 6 on test); `periods<=1` alone was +4/+5.
- Bit-exact, cut vs band: train 386 vs 403 of 880; held out 149 vs 156 of 358 (train rate 45.8%, test 43.6%, within
  the 5-point margin); T-3524 fresh 566 vs 574 and tiny 475 vs 475 of 1,000. Fresh sweep drawn after the freeze
  (`gen_sweep.py`, seed 4349; probes-numeric/t4349-cumprinc-short-selector-fresh-sweep, fit `cumulative-short`,
  excel-graph 2026-11-05, 800 cells over 1 to 4 periods, rates 1E-3..0.0997, leads to 0.095): 258 vs 299. Visibly
  off: train 47 vs 45, test 16 vs 14, fresh 61 vs 57, tiny 140 vs 140, the new sweep 216 vs 190 (77 fixed, 51 new,
  all CUMIPMT).
- Selection uses the train side only: round 1's visibly-off filter also counted held-out cells (fidelity review);
  restricted to train cells (`select.py`, round 2) the same selector is first, so the freeze and the sweep stand.
  The cut-points were fixed before scoring (the lead's are T-3524's bucket edges, 0.095 among them).
- R4.1 (`rustsheet-fitlab cumulative` / `cumulative-short` at the PR head), bit-exact / within 4 ulps / within 15
  digits, engine vs correctly rounded: old train 403/827/835 vs 252/709/723 of 880; old test 156/342/344 vs
  106/298/303 of 358 (45.8% vs 43.6%, within the 5-point margin; accepted). New sweep train 216/401/435 vs
  120/331/390 of 566, test 83/157/176 vs 57/148/167 of 234 (accepted).
- R4.4, the 4-ulp floor: within 4 ulps cut vs band, old train 825 vs 827, test 340 vs 342, T-3524 fresh 922 vs 926,
  tiny 835 vs 835, the new sweep 542 vs 558 (its fit-lab test slice alone 165 vs 157); where the forms differ by more
  than 4 ulps on the new sweep the band is closer on 133 cells and the cut on 87, so the gain is not only below the
  floor.
- Gate A: the new sweep is 800 of 800 within 1e-9 of the 50-digit value (fit lab `within_1e-9`,
  `kernels.cumulative.exact`), none outside, max 1,024 ulps (CUMIPMT cancellation).
- Eval delta: probes known failures 4820 to 4796 (31 fixed, 4 of them visibly: fin-cumprinc-underflow-ends I3, I4,
  fin-cumprinc-near-1e15 E3, E4); 7 regressions under decision 0296 (decision 0083's conditions), each bit-exact to 1 ulp, within 15
  digits (decision 0296 lists values): fin-cumprinc-near-1e15 B2, B3; fin-cumprinc-single-period-rate-scan K52, K57,
  K59; fin-cumprinc-underflow-ends D2, D3. gen-m9 and ironcalc-calc unchanged. New case: 800 cells, 190 visibly off,
  owned by T-4390.
- The cut's own `FITTED` range is tightened to where it decides, a lead below 0.3 (rates below 0.35): 2,241 cells,
  old probes at rates 1E-300..0.1 and the fresh sweeps 1E-12..0.35. The cut itself stays (decision 0295); the ruling
  decision 0295 asks for is filed as T-4400 (blocked on the owner).
- Follow-ups: T-4390 (Excel's own short-range form; owns the 7 regressed cells and the three sweeps' visibly-off cells),
  T-4400 (owner ruling on the cut).
- Findings: docs/excel-semantics/financial.md, "Neither short-range form is Excel's below a lead of 0.3".
