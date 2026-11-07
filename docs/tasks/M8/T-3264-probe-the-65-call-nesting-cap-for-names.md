---
id: T-3264
title: "Probe the 65-call nesting cap for names defined through the API and for a name's body counted in the formula that uses it"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for two unprobed corners of the 65-call cap on names, and the engine matching it.

## Context
- T-3143 (docs/excel-semantics/syntax.md "Formula nesting depth cap") found a stored name body of 66 nested calls
  refuses the workbook and 65 loads (probes/t3143-name-body-65); the importer checks bodies in
  `reparse_formula_hack` (rustsheet-xlsx import/mod.rs).
- Unprobed: (1) whether a cell formula's calls plus those of a name it uses add up (`N` = 40 calls, `=ABS^30(N)`
  in a cell); (2) a name defined through the API (Graph `names/add` with a 66-call body) and the engine's
  name-definition API for the same body.
- Probe at the boundary on both sides; refused inputs are dropped after the run (T-1091, T-3097).

## Acceptance criteria
- [x] excel-graph probes (milestone M8) for both questions; syntax.md records the result.
- [x] Where Excel refuses what the engine accepts (or the reverse), the engine matches, with tests at the boundary.
- [x] `make check` green; reviewers approve.

## Out of scope
- Conditional formats and data validation (T-3263).

## Outcome
- Harness: a spec's `"api_names": [{"name", "formula"}]` defines workbook-scoped names through the API
  after load (`tools/src/rustsheet_tools/api_names.py`, custom properties `rustsheet.apiname.<n>`): the
  excel-graph backend replays each with Graph `workbook/names/add` (a 400 `InvalidArgument` leaves the
  name undefined, decision 0179), the other backends refuse such cases, and the engine adapter replays
  each with `Txn::define_name` (`CaseSpec::api_names`, `define_api_names`).
- Evidence (excel-graph 2026-11-04, 7 probes/t3264-* cases, all with sidecars): `names/add` accepts a
  65-call body and refuses 66 (`=N` is 1 / `#NAME?`); a name's calls do not count toward the formula
  that uses it (40+25, 40+26 and 65+65 calls load and evaluate, stored and typed).
- Engine: `Model::define_name` (non-loading path, so `Txn::define_name` and `define_sheet_name`) refuses
  a body nesting more than 65 calls (`TOO_MANY_NESTED_CALLS_MESSAGE`); the importer keeps refusing the
  loaded case (T-3143). syntax.md "Formula nesting depth cap" records it all.
- Eval delta: probes +7 cases, 16 cells, all passing (eval/baseline/probes.txt records them) (t3264-api-name-66 S!A1 failed before the engine
  change: 1 instead of `#NAME?`); no regressions.
- Review round 1 (excel-fidelity): the Graph backend's api-name loop shadowed the uploaded file's
  name, so the two api-name sidecars recorded `workbook_name` "N"; fixed (with a test) and both
  re-filled from excel-graph with the stale oracle cache entries removed (same values).
- Follow-up: T-4014 (sheet-scoped `names/add`, which the engine now refuses by extension, and undo of a
  deleted name whose body was edited past the cap).
