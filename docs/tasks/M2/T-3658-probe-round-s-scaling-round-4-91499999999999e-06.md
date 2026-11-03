---
id: T-3658
title: "Probe ROUND's scaling: ROUND(4.91499999999999E-06,8) is 1 ulp above the nearest double"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3555]
---

## Goal
Find out when Excel's ROUND returns a double other than the one nearest its decimal result. Record the rule in
docs/excel-semantics/numbers.md: either rewrite "ROUND one ulp above the nearest double (observation)", added by T-3555,
or fold it into "ROUND family rounds in two decimal stages". File the engine task if the engine differs.

## Context
- T-3555's Gate A table `eval/accuracy/tables/ROUND/s1` Calc!A61 (excel-graph, 2026-11-01; PR #2691) has
  `ROUND(4.91499999999999E-06,8)` = 4.9100000000000004E-06 in Excel. That is 1 ulp above 4.91E-06, the double nearest
  the decimal result, and the engine returns 4.91E-06. It is the only one of the 360 ROUND-family points where the
  engine is not bit-exact with Excel. Gate A's envelope holds it, so it is not a failing point.
- numbers.md "ROUND family rounds in two decimal stages" (VERIFIED) says ROUND returns the nearest double.
- A candidate mechanism, from the Excel-fidelity review of #2691: in doubles `491*1E-8` is 4.9100000000000004E-06 and
  `491/1E8` is 4.91E-06. So Excel may scale the kept digits by multiplying by 10^-n on some path, much as ROUNDUP
  adds its unit in binary (numbers.md "ROUNDUP adds the unit in binary").
- Probe a grid of kept-digit integers m × num_digits n (say m in 1..999 and n in 1..12, with small and large
  magnitudes). Compare Excel's ROUND against the nearest double, `m*10^-n` and `m/10^n`, and record which model
  matches every cell. Use the `excel-probe` skill. ROUNDDOWN at the same points is the control.

## Acceptance criteria
- [x] A probe case with oracle values separates the models; numbers.md states the rule (VERIFIED) with the cells.
- [x] If the engine differs from the rule, an engine task is filed with the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the Gate A tables (a batch never changes) or the math definition (decision 0223).

## Outcome
- Probe `probes-numeric/t3658-round-scaling` (2,539 cells, excel-graph, 2026-11-03). Calc!A1:A1929: a grid of
  kept digits m and n = 0..14 where m x 10^-n differs from m x double(10^-n); a double multiply is ruled out (every
  cell is the nearest double except those whose result is 4.91E-06). Narrowing showed every route to 4.91E-06 (the
  literal, VALUE, a stored cell, `491/100000000`, `ROUND(491E-8,8)`, ROUNDDOWN) gives …8e7a inside Excel (checked
  with `X*2^70`, so not a transport artefact). Hits!A1:K61: the 61 decimals (m < 20000, n <= 15) where a 64-bit
  intermediate rounding moves the result off the nearest double.
- Rule (numbers.md "Decimals reach a double through an x87 multiply by 10^-n", VERIFIED, replaces the T-3555
  observation): round53(round64(m x round64(10^-n))). Literals and VALUE use the digits as written (122/122),
  ROUNDDOWN keeps trailing zeros of the kept digits (362/362), ROUND drops them first (1,871/1,871). Division
  is "Division rounds twice" (61/61). Stored xlsx numbers fit no model (still OPEN).
- The engine returns the nearest double: 385 cells off by one ulp. Filed T-3931 (ROUND, ROUNDDOWN) and T-3932
  (literals, VALUE; stored-number cells listed there for ownership).
- Not a numeric fit (no fitted constant): the model is a structural choice among ranked candidates (nearest,
  double rounding of the exact decimal, double or 64-bit power, 15- and 17-digit mantissas, with or without
  stripped zeros), chosen on t3658-round-scaling. Fresh sweep after freezing it, predictions computed before the
  oracle ran: `probes-numeric/t3658-round-scaling-fresh` (724 cells: every model-vs-nearest point for m in
  20001..79999, n = 1..15, plus 30 trailing-zero discriminators; ROUND, ROUNDDOWN, literal, VALUE): 724/724
  fit, 677 of them off the nearest double.
- Eval delta: probes-numeric (ungated) +2 cases: 2,154 of 2,539 and 56 of 724 cells pass (the rest are T-3931/T-3932);
  gated suites unchanged.
