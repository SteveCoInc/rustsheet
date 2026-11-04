---
id: T-2897
title: "Python: drop the bitcode and icalc loaders, apply_external_diffs and test_panic (SPEC-0026)"
milestone: M12
lane: api
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
The Python package no longer accepts bitcode or icalc bytes and no longer ships a panic function. Removed: `load_from_bytes`,
`load_from_icalc`, `create_user_model_from_bytes`, `create_user_model_from_icalc`, `UserModel.from_bytes`,
`UserModel.from_icalc`, `UserModel.apply_external_diffs`, `UserModel.flush_send_queue`, the module function
`test_panic` and `Model.test_panic`. The writers that have no reader left go too: `save_to_icalc` and `to_bytes`
on both classes.

## Context
SPEC-0026 decisions 1, 2 and 7 (docs/specs/SPEC-0026-python-legacy-surface.md, its symbol table).
Code: bindings/python/src/lib.rs, raw_model.rs, user_model.rs; bindings/python/python/rustsheet/__init__.py (imports and
`__all__`) and _ironcalc.pyi. Tests to drop or rewrite: tests/test_io.py (`test_icalc_roundtrip` and the bytes
round-trip; keep the xlsx ones), tests/test_diffs.py (the whole file) and docs/examples/collaboration.py (delete it, with its
`literalinclude` in bindings/python/docs/usage_examples.rst). No test calls `test_panic`. Pre-v1, so this ships with no warning release
(SPEC-0026 decision 7); note it in the package changelog or README.

## Acceptance criteria
- [ ] None of the listed symbols is importable from `rustsheet` (a test asserts it for each name).
- [ ] No binding source (bindings/python/src), stub (`.pyi`), example or rst page defines, exports or shows any of
      them; the facade's `Workbook.from_bytes` (xlsx bytes) stays.
- [ ] `make check` green (python job included); reviewers approve.

## Out of scope
- Moving the classes to `rustsheet.legacy` (T-2898).
- The engine's own `Model::from_bytes` and `apply_external_diffs` (T-2913, T-2896).

## Outcome
Removed the listed loaders, writers, diff exchange and both `test_panic`s from the bindings, stubs, docs and
examples; dropped the unused `bitcode` dependency; `tests/test_io.py` asserts every name is gone; hidden-row test now
round-trips through xlsx. README notes the removal. Eval delta: none (bindings only). `make py-bindings`: 171 passed.

## Handoff (resumed)
Implementation is complete and pushed (PR #3212, head efbe9ff): all listed symbols removed, `bitcode` dep dropped, `make check-fast` green
(171 Python tests), design and security reviewers APPROVE. CI fails only on `hygiene`: `scripts/check_governance.py` reports
"test files deleted (requires a decision record and reviewer sign-off): bindings/python/tests/test_diffs.py". The task (and SPEC-0026)
requires deleting that file, so it needs a decision record in `docs/decisions/` (`make new-decision TITLE="..."`) covering the
deletion of test_diffs.py (and `docs/examples/collaboration.py`), citing SPEC-0026 decisions 1, 2, 7; protected path, so not mine.
Next: add the record, set `status: done` again, push, re-run design review (reviewers must confirm the deletion), enqueue.
Also fold in the design MINOR: reword remaining "produces diffs for collaboration" text (README:19 and :40, user_model.rs:29,
__init__.py:10, _ironcalc.pyi:354, docs/user_api_reference.rst:6, docs/usage_examples.rst:16).
