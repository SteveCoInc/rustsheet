---
id: T-4854
title: "A1-cell-shape checks agree on case: one predicate behind sheet quoting, name validity and the lexer"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every "is this spelled like an A1 cell" check agrees on case, so sheet-name quoting, name validity and the lexer classify the same text the same way.

## Context
`parse_reference_a1` (expressions/utils/mod.rs) accepts only `'A'..='Z'` and is used without upper-casing by `name_needs_quoting`, `is_valid_a1_identifier` and `is_cell_spelled_name`; `parse_spelled_a1_cell` / `spelled_like_a1_cell` upper-case, `is_r1c1_shaped` uses `eq_ignore_ascii_case`, the lexer upper-cases first. A sheet named "rc" gets quoted, "ab12" does not.

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] One case-insensitive A1-shape predicate behind all four callers.
- [ ] Probe how Excel treats `ab12` and `AB12` as a sheet name and a defined name; tests pin the probed answer for both.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
