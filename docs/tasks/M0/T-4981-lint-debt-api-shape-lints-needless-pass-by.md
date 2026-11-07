---
id: T-4981
title: "Lint debt: API-shape lints (needless_pass_by_value, unused_self, unnecessary_wraps, excessive bools) fixed or given a reasoned expect"
milestone: M0
lane: quality
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4856]
---

## Goal
The API-shape lints of the fork's debt are fixed or carry a reason at the site: needless_pass_by_value (28), unused_self (14), unnecessary_wraps (12), struct_excessive_bools (12), trivially_copy_pass_by_ref (4), struct_field_names (4), inline_always (3), ref_option (2), mut_mut (1), fn_params_excessive_bools (1), implicit_hasher (1).

## Context
Decision 0335 moved the fork's crate-wide lint allows into per-file `#![expect]` blocks locked in quality/lint-debt.json (T-4856). `python3 -c "import json;d=json.load(open('quality/lint-debt.json'));print([f for f,v in d.items() if any('LINT' in e for e in v)])"` (put the lint name for LINT) lists the files. Each fix deletes the entry from the file's block (the build fails until it does, because the expectation goes unfulfilled), then `python3 scripts/check_lint_debt.py --update`. A file whose block empties loses the whole block. Where a violation is deliberate, replace the file-level entry with a statement-level `#[expect(clippy::x, reason = "...")]` at the site; never an `allow`. Semantics must not move: `make eval-check` shows 0 regressions and 0 fixes.

## Acceptance criteria
- [ ] Those 11 lints have no file-level entry left; each kept site has a statement-level expect with a reason.
- [ ] No public API of crates/rustsheet changes; engine-internal signatures may.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other lint groups (T-4980 to T-4986 each own one), and the xlsx importer's prints (T-4572).
