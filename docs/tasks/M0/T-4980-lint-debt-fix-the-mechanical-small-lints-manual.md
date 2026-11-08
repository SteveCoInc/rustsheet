---
id: T-4980
title: "Lint debt: fix the mechanical small lints (manual_let_else, elided lifetimes, format_push_string and ten more)"
milestone: M0
lane: quality
status: todo
priority: P3
size: M
tier: sonnet
depends_on: [T-4856]
---

## Goal
The mechanical lints of the fork's debt are gone: manual_let_else (22 files), elided_lifetimes_in_paths (23), format_push_string (15), items_after_statements (9), default_trait_access (6), assigning_clones (6), if_not_else (4, xlsx: clippy's own fix did not compile there), ignore_without_reason (3), enum_glob_use (2), doc_link_with_quotes (2), comparison_chain (2), redundant_else (1), single_match_else (1).

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. Semantics must not move: `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] Those 13 lints have no entry left in quality/lint-debt.json.
- [ ] `make eval-check`: 0 regressions, 0 fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
