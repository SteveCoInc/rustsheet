# 0350 — Decision 0042 for a spilled kept subnormal: an INDEX companion proves its first element

- **Status:** accepted (2026-11-07, T-5051). Extends decision 0042.
- **Context:**
  - Decision 0042 passes an excel-graph 0 against an engine subnormal only when the case has a FALSE companion
    `=(<formula>)=0`, because Graph saves a kept subnormal as 0.
  - For an array formula that companion is evaluated in array context. There Excel reads a number below 2^-1026 as
    equal to 0 (T-4766; docs/excel-semantics/numbers.md "Worksheet EXP (T-1931)"). So it is TRUE even when Excel keeps
    the subnormal: probes/subnormal-saved-negative-and-array D1, `=(EXP({-745,-708}))=0`, is TRUE.
  - A scalar read of the first element is FALSE in Excel, so Excel kept it non-zero:
    - `=(INDEX(A3#,1)=0)` in that case (A4);
    - `=INDEX(EXP({-745,-708}),1)=0` in probes/t4766-array-subnormal-compare (A5).

    `INDEX(A3#,1)*2^1000` recovers the kept value, 5.29E-23 (A5 of the first case).
  - Without this, the spilled anchors, subnormal-saved-negative-and-array S!A3 and t4766-array-subnormal-compare S!A30,
    stay known failures although the engine keeps the subnormal exactly as Excel does. Of these, S!A3 is part of the M11
    probe exit.
- **Decision:**
  - A FALSE companion of the form `INDEX(<x>,1)=0`, `(INDEX(<x>,1)=0)` or `(INDEX(<x>,1))=0` also marks `<x>` as kept
    non-zero (`case::kept_nonzero_formulas`). `<x>` is the formula itself or `<anchor>#` naming a cell of the same sheet.
  - Only element 1 is covered. That is the anchor cell, which holds the formula and is scored by formula, as decision
    0042 scores every cell. Elements past the first get no rule.
  - Every other condition of decision 0042 is unchanged: an excel-graph sidecar, an expectation of exactly 0, and an
    engine value that is non-zero and below the smallest normal double.
- **Consequences:**
  - The two anchors pass. The rule still catches a wrongly kept subnormal, because Excel's scalar INDEX read is TRUE
    where it flushed.
  - A spilled element past the first still needs its own evidence. A case that wants one scored adds a companion per
    element; this decision does not cover it.
