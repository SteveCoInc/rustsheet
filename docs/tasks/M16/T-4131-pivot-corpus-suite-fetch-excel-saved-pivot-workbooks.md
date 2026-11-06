---
id: T-4131
title: "pivot-corpus suite: fetch Excel-saved pivot workbooks, classify fresh or stale, score import"
milestone: M16
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A fetched suite of public Excel-saved workbooks with pivots (LibreOffice sc/qa/unit/data, ClosedXML resources, plus the pivot files of calipers, poi and corpus-bench by reference), each pivot classified fresh (its cache records reproduce its saved cells) or stale, scoring import and GETPIVOTDATA now and re-render later.

## Context
oss-engines.md §5 lists about 150 files and their licences (LibreOffice MPL-2.0 and ClosedXML MIT: fetched at a pinned commit, never vendored; EPPlus Polyform-NC: excluded). Files are untrusted input. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] eval/suites/pivot-corpus/suite.toml with pinned sources and a fetcher, like calipers.
- [ ] Fresh/stale classification recorded per pivot; stale ones reported, not scored for render.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Render scoring before the PV1 renderer lands.
