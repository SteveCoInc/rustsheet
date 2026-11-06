---
id: T-4504
title: "AMORLINC and AMORDEGRC: a salvage equal to the cost is 0, not #NUM! (Gate A T-4111)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
AMORLINC and AMORDEGRC with a salvage equal to the cost return Excel's value (0 at the periods probed) instead of
`#NUM!`, so the seven Gate A points below pass and their known-failures lines go.

## Context
- Found by T-4111's Gate A tables (excel-graph): Excel gives 0, the engine `#NUM!`, at
  accuracy/AMORDEGRC/s1!Calc!A114, A118, A128, A130, A131 (periods 1, 4, 1, 2, 1) and accuracy/AMORLINC/s1!Calc!A124,
  A125 (periods 5, 2), e.g. `AMORLINC(99977.707698474,42751,42993,99977.707698474,2,0.213533204746625)`.
- financial.md "AMORLINC and AMORDEGRC check order" records it (T-4111): `#NUM!` is for a salvage *above* the cost
  (gen-m9 5ded7594e597e31e A182, e8d55ec393460bb7 A176, probes/fin-amordegrc A21). The engine's
  `amortization_values_valid` (functions/financial/amortization.rs) requires `salvage < cost`; the lift's
  `ValueWork::Amortization` charge calls the same predicate.
- For AMORDEGRC the ported walk would charge `rate * remain` at period 1 when the remain equals the salvage, while
  Excel is 0: the fix is more than relaxing the comparison (an early 0 when nothing is depreciable, or `remain <=
  salvage` in the walk; probe to tell them apart).
- Unprobed: period 0 and periods between 0 and 1 with a salvage equal to the cost, and whether the equal case still
  meets AMORDEGRC's asset-life check. Probe them (excel-probe skill) before choosing the rule.
- Two assumptions in T-4111's Gate A definitions (defs/amortization.py) still need evidence, from the T-4111
  fidelity review:
  - Check order. A salvage equal to the cost with a rate of 0.5 or more (life 2 or less) is `#NUM!` from the
    asset-life check, which runs before the salvage-equal-to-cost branch.
  - AMORDEGRC's 0 at its half period (life - 3) and at the rest periods after it. Nothing probes those periods;
    the evidence covers periods 1, 2, 4 and 5.
  No table point hits either case. The probe settles both, and `fully_salvaged` becomes `NoValueError` wherever
  the probe leaves a case open.

## Acceptance criteria
- [ ] The seven points pass Gate A; their lines in eval/accuracy/known-failures.tsv are removed.
- [ ] A probe of the unprobed cases above, including the two definition assumptions, recorded in financial.md;
      the Gate A definition follows the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other AMORLINC and AMORDEGRC argument checks (T-2485).

## Probe predictions (committed before the fill)
Case probes/amor-salvage-equal-cost (column letters as in its description). Two rules are on the table:
**W** (the walk compares `remain <= salvage`, everything else as now with `salvage <= cost` allowed) and **E** (an
early 0 for a salvage equal to the cost once every argument check has run, the walk unchanged). Predicted, by both
unless marked:
- A1-A10, B1-B10, C1-C18: 0 (every period, the half and rest periods included; AMORLINC at fractions too).
- E1-E6: `#NUM!` (the asset-life check also refuses a salvage equal to the cost, at periods 0, 0.5 and 1);
  E7-E9: 0 (AMORLINC has no asset-life check); E10, E13, E25: `#N/A`; E11, E12, E14, E15: `#NUM!` (the basis is still
  read and checked); E16-E20: `#NUM!` (purchase after the first period, rate 0, period -1); E21-E24: 0 (cost = salvage
  = 0).
- F1, F8: W 970 (the fraction reads the 485), E 485. F7: W 970, E 485. F2: 485 + 0 + (W 0, the call leaves its 0;
  E 0 or 485). F3: 485 (AMORLINC leaves its clamped first charge, 0). F4: 970 (period 0 keeps the state). F5: 485.
  F6: W 485, E 485 or 970.
- G1: W 0, E 776. G2: 330. G3: 0. G4: 776. G5: W 0, E 485. G6: 0. G7: 360. G8: 0. G9: 0. G10: 360.
My pick before the fill: W.
