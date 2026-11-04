---
id: T-4050
title: "Salvage PR 1885: land the PV/PMT/IRR-near-rate--1 probe and quirk entry"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The Excel evidence that PR #1885 (T-2657) gathered reaches `main`: its probe case and its financial.md quirk entry.

## Context
Closed as a stale draft on 2026-11-04 (decision 0264, steward skill "Stale drafts" bucket 5). The branch is kept and read-only: copy files from it with `git show <sha>:<path>`. Do not merge or rebase it. Its baselines are hundreds of commits stale, so regenerate them with `make eval-baseline` and never copy them.
- Source: branch `agent/functions-financial/T-2657-pv-irr-div0` at 7e740e29b.
- Files: `eval/suites/probes/cases/t2657-pv-irr-div0.{case,oracle}.json` (431 cells, excel-graph, 2026-10-26); the financial.md entry "PV, PMT and IRR near a rate of -1"; the task notes in T-2657 (what was tried; next hypothesis).
- The PR said that PV's gen-m9 cell was already fixed on main, and that the probe's PV, FV and PMT error cells matched; the IRR cells and some PMT last digits did not.

## Acceptance criteria
- [ ] The probe case lands unchanged, with its milestone tag valid under decision 0057 (`check_governance.py`).
- [ ] The financial.md entry lands, reconciled with any IRR or PV entry that `main` has gained since.
- [ ] T-2657 on `main` carries the branch's progress notes and next hypothesis, with `status: todo`.
- [ ] The new failures are recorded by `make eval-baseline` as new coverage, with 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-2657 owns it).
