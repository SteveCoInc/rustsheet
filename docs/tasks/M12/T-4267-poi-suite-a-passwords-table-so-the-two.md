---
id: T-4267
title: "poi suite: a [passwords] table so the two encrypted fixtures load and score on cached values (SPEC-0031)"
milestone: M12
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
spec: docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md
depends_on: [T-4266]
---

## Goal
The poi suite loads `58616.xlsx` and `protected_passtika.xlsx` with their passwords and scores their cells on
cached values. Their `!InvalidCase` baseline lines are gone.

## Context
- SPEC-0031 test plan: a `[passwords]` table in eval/suites/poi/suite.toml (file path to password) that the runner
  passes as `LoadOptions::password`. The passwords are POI's public test data (T-4263 records them).
- Code: crates/rustsheet-eval (suite config and runner), eval/baseline/poi.txt via `make eval-baseline` only.

## Acceptance criteria
- [ ] Both fixtures leave `!InvalidCase`, and the eval delta is reported. Any cell that still fails is a known
      failure with an owner.
- [ ] xlsx-storage.md drops "known gap" for encrypted xlsx.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decryption itself (T-4265).
