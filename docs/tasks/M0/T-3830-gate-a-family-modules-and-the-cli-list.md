---
id: T-3830
title: "Gate A family modules and the CLI list test stop being shared hand-kept lists"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Two Gate A PRs no longer conflict unless they edit the same family module: adding a Gate A family
means adding one new module under `tools/src/rustsheet_tools/accuracy/defs/`, with no edit to a
shared file.

## Context
During the 2026-11-02 merge-queue cleanup, 6 of the first 7 PRs the queue dropped were Gate A table
PRs. All of them conflicted on the same two hand-kept lists that every such PR edits:
- `tools/src/rustsheet_tools/accuracy/defs/__init__.py` imports every family module on one line
  and lists them all in `__all__`;
- `tools/tests/test_accuracy.py::test_cli_lists_and_generates` asserts the complete, literal,
  alphabetical list of covered functions that `accuracy_cli.main(["list"])` prints.

Each merge forced the next Gate A PR through another repair and CI round, and the cleanup had to
land the 8 open Gate A PRs one at a time.

## Acceptance criteria
- [ ] `defs/__init__.py` discovers its family modules itself (e.g. `pkgutil.iter_modules` over the
      package, importing each so that it registers its definitions), with no hand-kept module list.
      Discovery is deterministic (sorted), and a module that fails to import fails loudly.
- [ ] `test_cli_lists_and_generates` no longer pins the whole literal list: it checks that the CLI
      lists exactly the registered definitions, sorted, plus a few fixed spot entries.
- [ ] Adding a new family touches only its own new module (shown by a test, or by a note in the
      Outcome on the diff an existing family would have needed).
- [ ] `make check-fast` green; CI green.

## Out of scope
- Splitting the eval baselines or `eval/accuracy/` tables (generated files).
