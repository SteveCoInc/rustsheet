---
id: T-4811
title: "Eval harness: a workbook locale per suite and case; per-locale runs, report rows and baselines"
milestone: M17
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4810]
---

## Goal
The eval harness runs a case under a workbook locale and language given by its suite or case, scores it against a sidecar of that locale, and reports and baselines each locale separately.

## Context
Today every case runs under `"en"` (rustsheet-eval loads workbooks with `load_from_xlsx_bytes(.., "en", ..)` and `Model::from_workbook(.., "en")`). Per the T-4810 spec: a `locale` key in suite.toml and case.json, typed entries applied through that locale's `set_user_input`, displayed text read in that locale, per-locale rows in the report and progress dashboard, and probe milestone rows per locale.

## Acceptance criteria
- [ ] A case with `locale = "de-DE"` runs and scores under de-DE; en-US cases are unchanged (0 regressions).
- [ ] The report has a row per locale; `make eval-baseline` and the gates handle locale cases.
- [ ] eval-harness.md documents the key.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
