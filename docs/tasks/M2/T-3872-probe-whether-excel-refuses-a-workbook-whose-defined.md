---
id: T-3872
title: "Probe whether Excel refuses a workbook whose defined-name body or rule formula passes the parenthesis or sign cap"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for what Excel for the web does with a stored defined-name body, formula conditional format
(`cfRule type="expression"`) and custom data validation whose formula passes the 256-parenthesis cap
or the 1,024-sign cap: refuse the workbook (createSession 501) or load it.

## Context
- Follow-up of T-3869: the importer refuses a workbook whose cell formula passes either cap (as Excel
  does, probes/t3172-*, t3266-*), but a name body or a rule formula past them still loads as a parse
  error, because those forms are unprobed. The call cap's refusal there was probed (T-3143
  probes/t3143-name-body-65, T-3263 probes/t3263-cf-abs-65, t3263-dv-abs-65).
- Importer check: `crates/rustsheet-xlsx/src/import/call_cap.rs` (`refuse_past_syntax_cap`,
  `check_rule_formulas`, and the name-body parse in `import/mod.rs`).
- Use the `excel-probe` skill: probe 256 and 257 parentheses and 1,024 and 1,025 minus signs before
  `1`, for a name body, a CF rule and a DV rule (stored).

## Acceptance criteria
- [ ] Probe cases for the three forms at and one past each cap, run on excel-graph (the refused ones
      recorded and dropped, as T-3172's were).
- [ ] syntax.md "Formula nesting depth cap" records the result; if Excel refuses, an engine-lane
      follow-up task extends the importer's check.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
