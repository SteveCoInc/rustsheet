---
id: T-4630
title: "CONVERT mi to km and in to cm: Excel for the web is one ulp from the engine (calipers formulas_engineering D26, D27, J26, J27)"
milestone: M9
lane: functions-engineering
status: todo
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
- [ ] The cells above match bit for bit, and no cell of probes/convert-*, probes-numeric/t3861-convert-prefix-sweep,
  probes-numeric/t3975-convert-mixed-prefix-sweep or ironcalc-calc ENGINEERING/CONVERT.xlsx regresses.
- [ ] engineering.md records the refined prefix rule with these cells; the R4 hold-out and fresh sweep are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The light-year and parsec factors (T-2047), unless the same rule fixes them.
