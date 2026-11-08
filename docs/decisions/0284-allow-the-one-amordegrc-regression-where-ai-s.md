# 0284 — Allow the one AMORDEGRC regression where -ai's 2026-10-30 value does not reproduce

- **Status:** accepted (2026-11-05, T-3393)
- **Amended by:** decision 0358 (amordegrc-fraction-state-ai Sheet1!A1 and gen-m9 d912671baa1364aa Calc!A76 are
  scored against their reproduced later fills).
- **Context:**
  - T-3393 explains AMORDEGRC's left-over value around dynamic-array formulas (financial.md "AMORDEGRC below
    period 1"). Particular function calls empty the value at the point Excel evaluates them, left to right
    within a formula; SEQUENCE is one of them. This is evidenced by 70 excel-graph probes (2026-11-05). The engine
    now follows the rule and fixes -ag and -ah.
  - `probes/amordegrc-fraction-state-ai` puts `=SEQUENCE(1,2)*0+AMORDEGRC(..485..)` below a fraction in A1.
    Its 2026-10-30 fill has A1 = 0, which main matched. Under the rule A1 is 485, and the engine now gives 485.
  - The same input bytes (sha256 3f8ec3b8…) give 485 on 2026-11-05 in two genuine excel-graph fills: the
    byte-identical copy -av, and a refill of -ai. The refill is not committed. So A1 is a cell whose Excel value
    varies between fills of the same bytes, like gen-m9 d912671baa1364aa Calc!A76 (T-3590).
  - The Excel-fidelity review of T-3393 rejected replacing the committed 10-30 sidecar with the refill. That
    would hide the regression, and it departs from the A76 precedent, where the old value stays committed as a
    known failure.
- **Options considered:**
  - *Refill -ai.* This overwrites committed evidence with a later run. T-3610's draft rule ("re-filling until a
    run matches us is ruled out") rejects it.
  - *Special-case the engine so -ai gives 0.* This fits one cell against the rule that every 11-05 cell shows,
    and decision 0056 forbids it.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3393 PR, for exactly
  `probes/amordegrc-fraction-state-ai` Sheet1!A1. The 10-30 sidecar stays committed. -av holds the 11-05 value.
  T-3610 owns the cell, together with A76, until it decides how such cells are scored.
- **Consequences:** `eval/baseline/probes.txt` gains one F line for -ai A1, and `eval/baseline/visible-parity.txt` gains
  one line for the case (1 cell). The baseline's only other new
  F line is the new-coverage cell -cc A2 (`SUM(LINEST(..))`, 4 ulps). Reviewers confirm that the diff adds no other
  regression line. If T-3610's rule excludes cells that vary between fills, this line goes too.
