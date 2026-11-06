---
id: T-4630
title: "CONVERT mi to km and in to cm: Excel for the web is one ulp from the engine (calipers formulas_engineering D26, D27, J26, J27)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`CONVERT(255,"mi","km")` is 410.38272 and `CONVERT(255,"in","cm")` 647.7 in the engine, as in Excel for the web (and
their negatives), with no loss on the cells the CONVERT model already fits.

## Context
- T-4534 probed calipers roundtrip/formulas_engineering/golden.xlsx Sheet1!D26, D27, J26, J27 on excel-graph
  (probes/t4534-bessel-convert-web-values B1:B4 and D1:D4, 2026-11-06): the web gives the desktop golden's values,
  410.38272, -410.38272, 647.7 and -647.7. The engine gives 410.38272000000006 and 647.6999999999999 (1 ulp; B1:B4 and
  D1:D4 are known failures).
- engineering.md "CONVERT constants and evaluation order": `value * F_from / F_to`, then the prefixes as one factor,
  the nearest double to the combined power of ten (T-3861, fitted with hold-out). For mi to km, 255 * 1609.344 is
  410382.72000000003; times 1E-3 gives the engine's 410.38272000000006, while dividing by 1E3 gives the web's 410.38272.
  For in to cm, 255 * 0.0254 is 6.476999999999999, and both times 1E2 and divided by 1E-2 give 647.6999999999999, while
  255 * 2.54 is 647.7. So the cause may be the inch factor or the order of the factors, not only the prefix step. `CONVERT(1,"m","ft")` is T-2046/T-2047.
- `fn_convert` in crates/rustsheet-engine/src/functions/engineering/convert.rs.
- Any change to the prefix step is a structural model choice (decision 0188 R4): it owes a hold-out and a fresh sweep
  against the T-3861 and T-3975 cells (reviewer-excel-fidelity check 7).
Cells:
- calipers/roundtrip/formulas_engineering/golden.xlsx (4): Sheet1!D26, D27, J26, J27
- probes/t4534-bessel-convert-web-values.case.json (8): Sheet1!B1, B2, B3, B4, D1, D2, D3, D4

## Acceptance criteria
- [x] The cells above match bit for bit, and no cell of probes/convert-*, probes-numeric/t3861-convert-prefix-sweep,
  probes-numeric/t3975-convert-mixed-prefix-sweep or ironcalc-calc ENGINEERING/CONVERT.xlsx regresses.
- [x] engineering.md records the refined prefix rule with these cells; the R4 hold-out and fresh sweep are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The light-year and parsec factors (T-2047), unless the same rule fixes them.

## Outcome
- **Cause:** the factors, not the prefix step. Excel keeps distance in angstroms (m = 1E10), as it keeps area in
  square angstroms. In metres, `255 * 1609.344 / 1 * 1E-3` rounds to 410.38272000000006. In angstroms,
  `255 * 16093440000000 / 1E10 * 1E-3` rounds to Excel's 410.38272, and the same holds for in to cm, 647.7.
- **Engine:** convert.rs's distance table now holds each unit's exact length in angstroms, as the nearest double. The
  pica is 1/6 inch and the point 1/72 inch. The evaluation order and the prefix rule are unchanged.
- **Structural model choice (decision 0188 R4):** ranked candidates on the 160 literal distance CONVERT cells with
  oracle values in probes and probes-numeric.

  | candidate | cells fitting (of 160) |
  |---|---|
  | metre base (old) | 143 |
  | angstrom base | 160 |
  | 1E-8 m | 157 |
  | 1E-7 m | 154 |
  | `value*((F_from/F_to)*P)` (order change) | 8 more fixed but 16 other CONVERT cells lost; rejected |

  - Hold-out by cell hash (30%, SHA-256 of case and cell). The split was computed after the candidates were scored on
    all 160 cells, so it is not blind; the fresh sweep is the blind test. Margin declared now: hold-out exact within
    5 points of train.

    | candidate | train (116) exact / 4 ulp / 15 digits | hold-out (44) exact / 4 ulp / 15 digits |
    |---|---|---|
    | angstrom base | 116 / 116 / 116 | 42 / 44 / 44 |
    | metre base | 104 / 116 / 115 | 39 / 44 / 44 |
    | correctly rounded | 107 / 116 / 115 | 42 / 44 / 44 |

    The angstrom base's hold-out (95.5%) is within the margin of its train (100%). It ties the correctly rounded
    value on the hold-out and leads it on train.
  - Fresh sweep after the factors were frozen: probes-numeric/t4630-convert-distance-sweep (seed 4630, 150 calls
    between distance units, with decimal prefixes on neither, one or both units), excel-graph 2026-11-06. 100 of the
    calls are `#N/A` (a prefix on a unit that takes none), and every candidate gets those right; 50 are numeric.

    | candidate | numeric cells exact (of 50) |
    |---|---|
    | angstrom base | 50 |
    | metre base | 33 |
    | correctly rounded (the .math.json) | 33 |

    All 50 are inside Gate A's envelope (within 1E-9 relative of the math).
  - No numeric constant was fitted: each factor is an exact length (no `FITTED` tag).
  - Open, unevidenced (fidelity review): with factors 1E10 larger, `value * F_from` overflows from about 1.8E298 for
    m to m, which then takes the divide-first path. The probed round values (1E290, 1E300) pass; a non-round value
    there may be 1 ulp off, as already holds for area in square angstroms.
- **Eval delta:**
  - The task's 8 probe cells (t4534 B1:B4, D1:D4) are fixed. The 4 calipers cells are the same formulas; calipers was
    not fetched here.
  - Also fixed: m to ft (scalar-lift-open-a Z8:Z10, convert-unit-names B33, eng-convert-boolean-unit), the
    light-year/parsec cells of the T-3861 sweeps, and gen-m9/932b33b1ef8b7608 A14 (in to mi).
  - In all: probes 15 fixed, gen-m9 1 fixed, 0 regressions in every suite.
  - ironcalc-calc CONVERT.xlsx and Gate A are unchanged.
  - T-2047's m to ft cells now pass; its area cells (`m2` to `ly2`) are untouched.
- **Follow-up:** T-4666 (from #3708's review, T-4475).
