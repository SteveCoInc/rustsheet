---
id: T-4133
title: "Fuzz target for pivot import (and render once it exists)"
milestone: M15
lane: quality
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A cargo-fuzz target feeds hostile pivot parts (indices, items, rowItems, groups, formulas, huge counts) to the importer, and later the renderer: no panic, bounded work.

## Context
Directive 5. SPEC-0015 limits; SPEC-0030 adds render limits. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Fuzz target committed with a seed corpus from the builder.
- [ ] A short CI run (or documented local run) shows no crash.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing findings beyond trivial ones (file tasks).
