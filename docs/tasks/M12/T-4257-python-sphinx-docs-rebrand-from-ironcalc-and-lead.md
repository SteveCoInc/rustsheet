---
id: T-4257
title: "Python Sphinx docs: rebrand from IronCalc and lead with the rustsheet.Workbook facade; legacy pages under rustsheet.legacy"
milestone: M12
lane: api
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-2898]
---

## Goal
The Sphinx docs in bindings/python/docs describe RustSheet: no IronCalc branding, the index and usage pages lead
with `rustsheet.Workbook`, and the user/raw API pages say they document the deprecated `rustsheet.legacy` module.

## Context
T-2898 moved `Model`, `UserModel` and their constructors to `rustsheet.legacy` (SPEC-0026) and rewrote the README,
but the Sphinx pages still say IronCalc: docs/conf.py (`project`, `html_title`), index.rst, installation.rst
("pip install ironcalc"), usage_examples.rst ("IronCalc ships two APIs"), user_api_reference.rst, objects.rst.
T-2898 only fixed top_level_methods.rst's module note. Mirror the README's wording.

## Acceptance criteria
- [x] `rg -i ironcalc bindings/python/docs` finds nothing except a historical credit, if any.
- [x] index.rst and usage_examples.rst open with the facade; the legacy pages name `rustsheet.legacy` and its deprecation.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the API or the examples.

## Outcome
- conf.py, index.rst, installation.rst, usage_examples.rst, user/raw API pages and objects.rst now say RustSheet;
  `rg -i ironcalc bindings/python/docs` finds only the fork credit in index.rst. The IronCalc logo options
  (files that were never in `_static`) are gone.
- index.rst and usage_examples.rst open with `rustsheet.Workbook` (examples/facade.py); the legacy pages name
  `rustsheet.legacy` and its removal at the API v1 freeze.
- Eval delta: none (docs only).
