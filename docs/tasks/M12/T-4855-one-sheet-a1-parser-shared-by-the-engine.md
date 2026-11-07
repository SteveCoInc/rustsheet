---
id: T-4855
title: "One Sheet!A1 parser shared by the engine test helpers, the xlsx importer and the facade"
milestone: M12
lane: api
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One `Sheet!A1` reference parser (quoted sheets, `''`, `$`, case, bounds) serves the facade, the xlsx importer and the engine's helpers.

## Context
`Model::parse_reference` (model.rs) pushes bytes as chars (garbles non-ASCII sheet names) and bounds-checks; xlsx `parse_reference` (rustsheet-xlsx/src/import/worksheets.rs) walks chars, does not bound-check rows, and carries a FIXME saying both should be fixed; neither handles `'My Sheet'!A1` or `$`. The facade's `split_sheet` / `parse_part` (crates/rustsheet/src/refs.rs) handles quotes, case and bounds and says it did not reuse the engine parser because it is upper-case only.

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] The engine owns one parser with the facade's rules; the facade and the importer call it; the FIXME is gone.
- [ ] Tests: non-ASCII and quoted sheet names, `$` and out-of-range rows through each caller.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
