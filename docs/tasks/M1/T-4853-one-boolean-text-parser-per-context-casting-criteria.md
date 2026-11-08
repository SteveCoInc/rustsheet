---
id: T-4853
title: "One boolean-text parser per context (casting, criteria, typed input, xlsx) instead of seven variants"
milestone: M1
lane: engine
status: in-progress
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
- [ ] Engine casting sites call one helper; ASCII-vs-Unicode case folding follows a probe ("falſe").
- [ ] ADDRESS and other sheet-name writers agree on quoting TRUE/FALSE sheet names (probe).
- [ ] xlsx boolean reads share one helper per OOXML type (xsd:boolean vs cell `t="b"`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
