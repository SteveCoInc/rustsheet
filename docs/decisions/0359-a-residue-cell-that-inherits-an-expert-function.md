# 0359 — A residue cell that inherits an expert function's open miss is a Phase 1 exception

- **Status:** accepted (2026-11-08, owner ruling; recorded by T-5101). Amends decision 0134. It widens what
  eval/milestones/phase1-exceptions.tsv may list.
- **Context:**
  - Decision 0355 measures the usage-tier gates. Its foundational tier allows no wrong type and at most 0.01% of
    numbers outside the envelope.
  - On the full eval of main e53d43fa (2026-11-08), the foundational tier has exactly one number outside the
    envelope: ironcalc-calc/FINANCIAL/IRR.xlsx `General!M17`.
    - The formula is `=NPV(L17,A17:F17)`. Excel's value is −3.066e-11, and ours is −3.485e-13.
    - L17 is `=IRR(A17:F17)`, so M17 evaluates NPV at IRR's root. Both values are rounding residue near 0.
    - The cell has no math value, and two different residues cannot agree to 12 significant digits.
  - The engine's NPV is right for the rate it is given. The rate differs because L17's IRR takes the fallback
    path, where our iterate stops elsewhere than Excel's. T-2928 owns that fit. It is blocked, time-boxed under
    decision 0056 after two attempts, until T-3117's probe evidence arrives.
  - The cell counts toward NPV, a foundational function, because attribution (decision 0118) blames the only
    function a formula calls. The real miss is IRR's, an expert function.
- **Decision:**
  1. eval/milestones/phase1-exceptions.tsv may also list a cell that meets all three conditions below. The cell
     is listed with this decision and with the open task that owns the inherited miss:
     - the function its formula calls computes a near-zero residue (the cell's expected value is 0 up to rounding,
       such as NPV at an IRR root);
     - the engine's result is right for the inputs it is given;
     - its only miss comes from an input another function computes, and that function's fit is an open task.
  2. The first such cell is ironcalc-calc/FINANCIAL/IRR.xlsx `General!M17`, owned by T-2928.
  3. As with every line of the list (decision 0134), the line hides nothing from eval/baseline or the CI gate. The
     cell keeps its `F` and `E` lines, so it stays a known failure and the ratchet still guards it. It leaves only
     the Phase 1 measures that leave out decision 0134's cells, the v1.5 checkpoint rate and the usage-tier gates
     among them.
  4. A new line of this kind needs a decision record naming its cell, as every line does (decision 0044).
- **Consequences:**
  - The foundational tier's only value miss leaves the gate. It still has TEXT's 28 and EDATE's 1 wrong-type
    cells.
  - When T-2928 lands and L17 holds Excel's rate, M17 should pass. The line then reports as stale and is
    removed in that PR.
