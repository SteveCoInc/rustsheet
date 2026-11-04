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
- New probe case `probes/t4204-fraction-denominator-mixed` (42 cells, excel-graph, tag M10). Rows 1-8 hold the mixed
  shapes `?#0`, `#?0`, `?0#`, `?#?`, `#?#`, `0#?`, `#??` and `0??`, and rows 9-14 the digit limit (0.123, PI(), 0.3).
  The excel-fidelity review of round 1 asked for these probes, which the task asked for too. They disproved round 1's
  "cut the run at its first `0`": the whole run bounds the digits (`?0` on 0.123 is 8/65).
- `formatter/fraction.rs` `fill_left` places the denominator's digits. Leading `#`s take none. A short denominator
  leaves out `0`s first, then `#`s, then the rightmost `?`s, and each left-out placeholder prints in place. The parser
  is unchanged from main.
- probes/t3711-fraction-trailing-placeholders: 57/57 (23 fixed); t4204: 42/42. number-formats.md "Fraction formats"
  has the rule and its cells.
- Eval delta: probes known failures -23, plus 42 new passing cells; 0 regressions in every suite.
