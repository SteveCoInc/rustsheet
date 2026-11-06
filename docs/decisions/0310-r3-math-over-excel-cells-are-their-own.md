# 0310 — R3 math-over-Excel cells are their own bucket, not visibly off

- **Status:** accepted (2026-11-06, owner approval; T-4520).
- **Amends:** decision 0083, for decision 0188 R3 cells only: a cell where the engine returns the math by design is
  not visibly off. Decision 0267 is the precedent (its `residue13` bucket, SPEC-0030).
- **Context:**
  - Decision 0188 R3 has the engine follow the 50-digit math where Excel is more than G = 1e-4 off it on evidenced
    points. `eval/accuracy/math-over-excel.tsv` marks those cells. R1(c) (`r1c.rs`, `R1c::R3Pass`) already counts
    such a cell as passing in the milestone probe rows, and Gate B passes it too.
  - `closeness::bucket` still put the cell in `visible_numeric`. So every other reader of visibly off counted a
    by-design result as a miss: report.json `closeness`, `attributed` and `excel`; the progress headline; the Phase 1
    rate and tiers; and the visible-parity lock. Example: the 66 probes-numeric/t3367-nper-tiny-rate cells, NPER
    below a rate of 1E-14 (docs/excel-semantics/financial.md).
  - The owner approved the change on 2026-11-06 ("Yes, go ahead with all three steps",
    https://claude.ai/code/session_017265sRitBHUeqGLLXeJqDz).
- **Decision:**
  1. A cell is `r3_math` when R1(c)'s R3 pass holds for it, and only then. The test is `r1c::r3_math`:
     - Excel's value and the engine's are both numbers;
     - math-over-excel.tsv marks the cell;
     - Excel is more than G off the cell's math value;
     - the engine is within ε = 1e-9 of the math, which is Gate A's envelope with Excel's side dropped.

     The run applies the test once (`r1c::mark_r3_math`, `CellResult::target` = `Target::Math`), before the report counts the buckets.
     `closeness::bucket` reads that mark, so there is one definition. A type, error or text cell never qualifies:
     R3 never applies to an error.
  2. `r3_math` is its own `Closeness` bucket:
     - every report counts it and shows it on its own line;
     - `scored()` includes it and `visible()` does not;
     - `Visible::is_off` is false for it.

     A function with such cells is placed no higher than `rate999`, as with `residue13`. A number within 15 digits of
     Excel stays `digits15`.
  3. Gate A does not change. Its envelope already drops Excel's side of an R3 point its own way. It reads
     `Visible::is_off_at_fifteen_digits`, which counts `r3_math` as off, and its own points never carry the mark.
  4. Nothing bit-level changes: not the known-failure baselines, and not the `eval-check` regressions.
- **Consequences:**
  - On probes-numeric at origin/main e72ae138, 66 cells move from `visible_numeric` to `r3_math`. Visibly off goes from
    4,820 to 4,754, and attributed visibly off from 4,647 to 4,581.
  - R1(c)'s classes and its `r1c_left_out` still list these cells as `r3_pass`, and so do the probe rows' `r3_pass`
    counts. They are no longer in the probe rows' `visibly_off` or in the frozen lists' visibly-off names.
  - The steward's next `--lower-visible-lock` lowers the lines these cells free.
  - A new R3 mark still needs an owner ruling (decision 0188 R3). Its cells join `r3_math` only when the engine
    returns the math.
  - SPEC-0030's "R3 math-over-Excel cells" section lists the consumers.
