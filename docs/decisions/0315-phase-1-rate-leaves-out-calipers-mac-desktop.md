# 0315 — Phase 1 rate leaves out calipers Mac-desktop golden cells where Excel for the web gives the engine's value (trig at k pi/2, CHAR(255), INFO release)

- **Status:** accepted (2026-11-06, T-4534), under the owner's request recorded in decision 0308 ("put the desktop only
  functions on the exception list").
- **Context:**
  - The calipers goldens formulas_math_trig, formulas_text and formula_stress_test (and the pv26 pv_goldens) were saved by desktop Excel for Mac (docProps/app.xml: Microsoft Excel 16.0300). 18 cells not yet excepted are
    visibly off. In every one, Excel for the web returns exactly the engine's value
    (probes/t4534-desktop-golden-web-values, excel-graph 2026-11-06, 36 cells):
    - 16 trig cells at multiples of pi/2 (formulas_math_trig Sheet1 C23, C25, D22, D24, E22:E25, F22, F24, G23, G25,
      H22:H25). The web reduces by the x87 64-bit pi (numbers.md "SIN / COS at multiples of 90° return
      k·6.1257422745431E-17"; `COS(PI()/2)` is 6.1257422745431E-17, `TAN(PI()/2)` 1.6324552277619072E+16,
      `CSC(PI())` 8162276138809536). Mac desktop gives the correctly rounded value (6.123233995736766E-17).
    - formulas_text C190 `CHAR(255)`: the web gives ÿ (Windows-1252); Mac desktop gives ˇ (Mac Roman).
    - formula_stress_test Information!B3 `INFO("release")`: the web gives `#VALUE!`; desktop gives "16.0".
    - The probe also confirms the web's `#VALUE!` for the pv26 `MID(CELL("filename",A1),...)` sheet-name cells; they
      are decision 0308's lines already.
  - These cells keep SIN, COS, TAN, SEC, CSC, COT, CHAR and INFO below Gate B's rate. No engine change can move them
    while the engine follows the oracle of record (decision 0039 rule 2; desktop out of scope, decision 0266).
- **Options considered:**
  - *Count them.* Eight functions stay below the Phase 1 rate for good, although the engine matches the web.
  - *List them as Phase 1 rate exceptions (chosen),* decision 0134's mechanism, as decision 0308 did for the
    FORECAST.ETS family and CELL's desktop-only info types.
- **Decision:**
  1. The 18 cells are lines of eval/milestones/phase1-exceptions.tsv under this decision, owned by T-4373 (the placeholder
     for cells no oracle in scope can settle). They stay known failures; only the Phase 1 rate leaves them out.
  2. If Excel for the web ever returns the desktop value for one of them, its line goes and the engine follows the web.
- **Consequences:**
  - SIN, COS, TAN, SEC, CSC, COT, CHAR and INFO lose these cells from their Phase 1 count; each meets the rate unless
    another cell keeps it below.
  - T-4373 names these cells; T-4534 keeps the cells it still owns (poi, ironcalc-calc ENGINEERING, formula_stress_test
    NORM.INV and NORMINV, poi SUMPRODUCT).
