---
id: T-4852
title: "One error-literal table: engine #N/IMPL spelling and error-name case rules agree with the parser"
milestone: M1
lane: engine
status: done
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
- [x] One literal table in token.rs; the facade maps onto it; the value.rs workaround notes are gone.
- [x] Case handling of each entry point is probe-backed or documented in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).

## Outcome
- Probe probes/t4852-error-name-case (excel-graph, 72 cells): Excel reads an error name in any
  case as a typed constant, a formula literal (`=#n/a` is `#N/A`, FORMULATEXT `=#N/A`) and a
  COUNTIF criterion (`"#n/a"` counts `#N/A` cells). Recorded in docs/excel-semantics/syntax.md
  "Error names are read in any case".
- token.rs: `Error::english_literal` is the one en-US table (now `#N/IMPL!`, the spelling the
  parser and the facade use); `get_error_by_english_name` and `english_criterion_error`
  (replacing `is_english_error_string`) walk `Error::ALL` over it. `localized_names` is the one
  localized table, read in any case by `get_error_by_name` (typed input) and
  `formula_error_prefix` (the lexer's `consume_error`, which had twelve case-sensitive branches).
  Criteria compare error kinds instead of display strings.
- Facade: `ErrorKind::literal` and `stored_literal` map onto the engine table; the `#N/IMPL`
  workaround notes and `trim_end_matches('!')` in the value.rs tests are gone.
- Eval: `UNSUPPORTED_ERRORS` named `#N/IMPL`, which the facade never produced (it reports
  `#N/IMPL!`), so engine `#N/IMPL!` cells were scored as type or error mismatches rather than
  `Unsupported`; it now names `#N/IMPL!`.
- Eval delta: see the PR.
