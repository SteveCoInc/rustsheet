---
id: T-4204
title: "A fraction's denominator run splits at its first 0 after a ?, and a leading # drops: ?/?0, #0, 0?"
milestone: M10
lane: functions-text
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Placeholder runs after a fraction's `/` that mix `?`, `#` and `0` print as Excel does (24 cells of probes/t3711-fraction-trailing-placeholders pass).

## Context
- Found by T-3711 (trailing `0` after a literal prints its zero; fixed there). Excel's text for runs that mix
  placeholder kinds right after the `/` (12.5 and 0.5 and 12.125, excel-graph) is not the engine's:
  `# ?/?0` "12 1/20" (ours "1/02"), `# ?/??0` "12 1/2 0", `# ?/?0?` "12 1/20 " (so the run splits at its first
  `0` when it starts with `?`: denominator `?`, then trailing `0?`), but `# ?/#0` "12 1/2", `# ?/#?` "12 1/2",
  `# ?/0?` and `# ?/0#` "12 1/02", `# ?/00?` "12 1/002", `# ?/#0#` "12 1/02" (the whole run is the denominator,
  a leading `#` takes no width) while `# ?/?#` and `# ?/??#` print no zero for the `#`.
- Cells: probes/t3711-fraction-trailing-placeholders.case.json (23): S!A5, A9:A11, A13:A16, B9:B11, B13:B16, C5, C9:C11,
  C13:C16.
- Code: formatter/fraction.rs `fill_left`, formatter/parser.rs (the denominator run). Rule to confirm with more
  probes (`?#0`, `#?0`, `?0#`) before generalizing.

## Acceptance criteria
- [x] The 23 cells above pass; the rule is in number-formats.md "Fraction formats".
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Parser (`formatter/parser.rs`): a placeholder denominator run that starts with `?` ends before its first `0`; the
  rest are ordinary placeholders after the denominator (T-3711's rule). Formatter (`formatter/fraction.rs`): a run
  that starts with `#` or `0` is right-aligned with the existing `fill`; a `?` run keeps `fill_left`.
- The 23 cells of probes/t3711-fraction-trailing-placeholders pass (57/57). The rule fits every cell of rows 5 and
  9-19, so no extra probes (`?#0`, `#?0`, `?0#`) were needed to choose it; whether the cut changes the denominator's
  digit limit stays unprobed and is recorded as such in number-formats.md.
- number-formats.md "Fraction formats": new bullet with the rule and cells; the T-2209 unprobed `0`/`?` choice
  dropped (now evidenced: `?/0?` is "1/02").
- Eval delta: probes known failures 4043 -> 4020 (fixed 23), 0 regressions in every suite (poi, calipers and
  corpus-bench fetched and scored).

