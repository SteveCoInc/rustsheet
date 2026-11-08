---
id: T-4853
title: "One boolean-text parser per context (casting, criteria, typed input, xlsx) instead of seven variants"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Reading TRUE/FALSE from text uses one helper per context (cell casting, criteria, typed input, xlsx attributes), each rule backed by evidence.

## Context
Variants: `cast_bool_ref` (cast.rs, `to_lowercase() == "true"`); `beta_cumulative` (functions/statistical/beta.rs, `to_uppercase()` — also accepts "falſe"); DAYS360 (date_and_time.rs, `eq_ignore_ascii_case`); criteria (criteria.rs, `to_lowercase().parse::<bool>()`, four sites); D-function computed criteria (database.rs, `eq_ignore_ascii_case`). Localisation: typed input English-only, lexer and display use `language.booleans`. Sheet quoting: `address_sheet_text` quotes exact "TRUE"/"FALSE", `name_needs_quoting` never does. xlsx: `t="b"` compares `== "1"` (worksheets.rs, external_links.rs), attributes use `parse_bool_with_default` (any-case "true"), pivot caches accept "1" | "true".

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [x] Engine casting sites call one helper; ASCII-vs-Unicode case folding follows a probe ("falſe").
- [x] ADDRESS and other sheet-name writers agree on quoting TRUE/FALSE sheet names (probe).
- [x] xlsx boolean reads share one helper per OOXML type (xsd:boolean vs cell `t="b"`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).

## Outcome
- Probes probes/t4853-boolean-text (56 cells) and t4853-boolean-text-criteria (22 cells), excel-graph. Excel folds
  ASCII case only: `"falſe"` is `#VALUE!` in every casting context and matches nothing as a criterion. A sheet named
  TRUE or false is quoted in FORMULATEXT and ADDRESS (any ASCII case) but bare after `[Book]` in CELL("address").
  coercion.md "TRUE and FALSE text fold ASCII case only (T-4853)".
- One helper, expressions/utils `boolean_text`, now serves `cast_bool_ref`, BETA.DIST's `beta_cumulative` (it
  upper-cased, so `"falſe"` was FALSE), DAYS360, the four criteria.rs sites and database.rs computed criteria.
  `name_needs_quoting` quotes a boolean name, and ADDRESS uses `quote_name` instead of its own exact-"TRUE" check.
- xlsx: one `util::xsd_boolean` for every `xsd:boolean` attribute and one `util::cell_boolean` for `t="b"` values
  (xlsx-storage.md "Boolean attributes and boolean cells (T-4853)"). `showGridLines`, `tabSelected`, the `<f>` flags and
  `cm` read `true` too now.
- Typed input's boolean (`to_lowercase().parse::<bool>()` in model.rs `reenter_printed_input`) is left alone: it
  accepts the same set, and T-4851's PR #4075 is moving the code next to it.
- Eval delta: two new cases, 78/78 (main: 71/78, BETA.DIST C6:C7 and quoting H2, H3, H5, J2, J5 off), regressions 0.
- Follow-up: T-5082 (T-4851's deferred design minors).

