---
id: T-4545
title: "INDIRECT sheet prefixes lookup.md leaves unprobed: unquoted non-ASCII and digit-then-letter sheet names, an R1C1 text on an all-digit sheet, and a two-sided 2024!B1:2024!B3 range: probe and match"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
INDIRECT reads the sheet-prefix spellings that docs/excel-semantics/lookup.md lists as unprobed the way Excel does, on
excel-graph evidence. Those spellings are an unquoted non-ASCII sheet name, an unquoted name made of digits then
letters, an R1C1 text on an all-digit sheet, and a range text that names its sheet on both corners. lookup.md cites
the cells for each.

## Context
- Gap triage 2026-11-06 (`make eval` on main 25779368). No lookup function has an open visibly-off cell. INDIRECT
  (M6, prior 1.5) has the highest catalog prior of the open lookup questions.
- lookup.md, "INDIRECT of `Sheet!Name` text: the prefix must be spelled as a formula would spell it" (T-2660,
  probes/indirect-sheet-scoped-name) says: "Verified for ASCII sheet names only; unquoted non-ASCII names
  (`"Données!tn"`) and letter-after-digit names (`"1Q!tn"`) are unprobed (the engine accepts them)". It also says:
  "A typed `=2024!B1`, the R1C1 form `INDIRECT("2024!R1C1",FALSE)` and `"2024!B1:2024!B3"` are unprobed."
- What the engine does today (a scratch run on main 25779368, sheets `Données`, `1Q` and `2024`, each with B1:B3 = 7,
  8, 9): `INDIRECT("Données!B1")` is 7, `INDIRECT("1Q!B1")` is 7, `INDIRECT("2024!R1C2",FALSE)` is 7, and
  `SUM(INDIRECT("2024!B1:2024!B3"))` is `#REF!`. The first three accept forms that the evidenced quoting rule
  ("unquoted, the sheet name may hold only letters, digits, `_` and `.`") does not settle for non-ASCII letters or a
  digit-led mixed name. The fourth refuses a form that Excel may accept.
- Probe (excel-probe skill, excel-graph, tag M6). Use the three sheets above, plus `Ünïcode`, `Q1` and `x.y` as
  controls. Use cell texts and the names of probes/indirect-sheet-scoped-name (`tn` scoped to each sheet). Read each
  result through ISREF, SUM and ERROR.TYPE:
  - `"Données!B1"`, `"'Données'!B1"`, `"Données!tn"`, `"données!B1"` (case), `"Ünïcode!B1"`;
  - `"1Q!B1"`, `"'1Q'!B1"`, `"1Q!tn"`, `"1q!B1"`;
  - `INDIRECT("2024!R1C2",FALSE)`, `INDIRECT("'2024'!R1C2",FALSE)`, `INDIRECT("2024!R[0]C[1]",FALSE)`,
    `INDIRECT("1Q!R1C2",FALSE)`, `INDIRECT("Données!R1C2",FALSE)`;
  - range texts with the sheet on both corners: `"2024!B1:2024!B3"`, `"Q1!B1:Q1!B3"`, `"'1Q'!B1:'1Q'!B3"`,
    `"Q1!B1:x.y!B3"` (two different sheets), `"2024!B1:B3"` as a control.
  If Excel for the web will not load a workbook with one of these sheet names, record the refusal and leave that form
  as a desktop question (decision 0039). Do not guess it.
- Commit the engine's values as predictions before the fill (docs/research/t4545/). The rule is structural.
- Code: `fn_indirect` and `indirect_of` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs),
  `indirect_defined_name` (indirect_fallbacks.rs), and the sheet-prefix lexer that T-2678 extended. Keep one reader
  for the prefix rule (directive 3). The same rule for formulas is in references.md ("Sheet-qualified range corners:
  order and unprobed forms").

## Acceptance criteria
- [ ] probes/t4545-indirect-sheet-prefix-spellings (excel-graph, canary-verified) covers every form in Context, or
      the PR records which ones Excel for the web refused to load. Its predictions were committed before the fill.
- [ ] Every cell of the case passes. Any cell that does not is recorded in lookup.md and owned by a task filed with
      `make new-task` (`campaign: semantics` with a `## Visible-parity stage` section) and named in the Outcome.
- [ ] lookup.md: the two "unprobed" sentences quoted in Context are replaced by the evidenced rule, with the cells
      cited. The typed `=2024!B1` stays listed as unprobed unless the case covers it.
- [ ] The cells of probes/indirect-sheet-scoped-name and t2577-indirect-ref-array keep their current verdicts. No new
      known failures in any baseline; `make check` green; reviewers approve.

## Out of scope
- How the formula parser reads a typed `=2024!B1` (references.md; parser lane work).
- INDIRECT's a1 lift and 1x1 array arguments (probes/t2577-indirect-ref-array; T-2347) and its a1 read from a cell
  (T-4538).
- Structured-reference text in INDIRECT.
