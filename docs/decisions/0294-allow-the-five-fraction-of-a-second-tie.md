# 0294 — Allow the five fraction-of-a-second tie regressions of T-4332's field-by-field cut

- **Status:** accepted (2026-11-05, T-4332)
- **Context:**
  - T-4323 cut the clock and the fraction digits of a second, with `.0`, `.00` or `.000` shown, from the half-step
    register `floor(frac(y)*86400*s)`, where `y = x + 0.5/(86400*s)`. That matched every tie cell from serial 1 up,
    but only 101 of the 156 tie cells below 1 in the five t4323 probe cases.
  - T-4332 cuts `frac(y)` field by field instead: multiply by 24, floor and keep the remainder, then the same with
    60, 60 and s, in plain doubles (`field_split`, formatter/elapsed.rs; docs/research/t4332/chain.py). This is a
    structural model, an operation order chosen from ranked candidates (reviewer-excel-fidelity check 7 (c)), with
    no fitted constant.
    - On the five t4323 cases: 134 of 156 below serial 1 and 469 of 470 from 1 up. The register gets 101 and 467.
    - On the fresh sweep probes/t4332-fraction-fresh: 103 of 120. That sweep has 120 cells, drawn after the model
      was frozen (seed 43320; excel-graph 2026-11-05), and its first 60 separate the two models. The register and
      the engine's earlier rounding of the value get 72 each.
  - Gated suites: probes fixes 40 cells. Five cells the register matched now miss, all ties below serial 1 where
    Excel rounds up and the cut's last remainder lies 2 to 4 ulp below a whole unit:
    - t4323-fraction-ties A121, A122, A124 (`TEXT(0.0005/86400,"[h]:mm:ss.000")` and the like);
    - t4323-fraction-sweep-2 A124;
    - t4323-fraction-holdout A100 (`TEXT(3.55/86400,"[s].0")` is "3.6", now "3.5").
    Excel truncates other ties whose remainder lies 8 ulp below: holdout A5, `TEXT(20.65/86400,"[s].0")` is "20.6".
    So no snap on that remainder keeps the five without losing others.
- **Options considered:**
  - *Keep the register where it matched (max of the two cuts).* This is a branch chosen to keep previously matching
    cells. Decisions 0056 and 0082 forbid it, and it scores 84 of 120 on the fresh sweep against the cut's 103.
  - *Keep the register.* That leaves 35 more probed cells off.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-4332 PR, for exactly these five
  cells.
- **Consequences:**
  - `eval/baseline/probes.txt` gains the five lines and loses the 40 fixed ones. Reviewers confirm the baseline diff
    adds no other line.
  - T-4370 owns the cells the cut still misses, these five included.
  - number-formats.md "Fraction digits of a second at a tie" records the cut and the open cells.
