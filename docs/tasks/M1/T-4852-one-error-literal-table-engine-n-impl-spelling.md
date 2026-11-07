---
id: T-4852
title: "One error-literal table: engine #N/IMPL spelling and error-name case rules agree with the parser"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Error literals have one table: the engine prints, parses and recognises the same spelling and case rules as the facade.

## Context
`english_literal` (expressions/token.rs) prints `#N/IMPL` without the `!` that `get_error_by_english_name`, `is_english_error_string` and the facade's `ErrorKind::literal` (crates/rustsheet/src/value.rs) require (value.rs notes it). Case: `get_error_by_name` matches exactly except `#GETTING_DATA` (`eq_ignore_ascii_case`); typed input and the facade `from_literal` upper-case first; criteria's `is_english_error_string` is case-sensitive over 12 names (deliberate).

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] One literal table in token.rs; the facade maps onto it; the value.rs workaround notes are gone.
- [ ] Case handling of each entry point is probe-backed or documented in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
