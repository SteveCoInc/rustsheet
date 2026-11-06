---
id: T-4451
title: "One shared _parts helper for the spec2xlsx tests instead of five copies"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The spec2xlsx tests share one helper that builds a spec and returns its XML parts, instead of each test file defining
its own `_parts`.

## Context
- PR #3472 (T-4394) design review, MINOR: `tools/tests/test_spec2xlsx_active_sheet.py` added a fifth copy of
  `_parts(spec)` (`zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))`, the `.xml` members decoded). Find every copy with
  `rg -n "def _parts" tools/tests`.
- Put one helper in `tools/tests/conftest.py` (or a small `tools/tests/spec_parts.py` module the tests import), and
  replace the copies. Keep any copy whose body differs only if it really needs to, and say why.

## Acceptance criteria
- [ ] `rg -n "def _parts" tools/tests` finds one definition; every spec2xlsx test still passes; ruff and mypy strict are
      clean.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`tools/tests/spec_parts.py` now holds the one `spec_parts(spec, suffixes=(".xml",))` helper. The date-system tests call it
directly; the pivot tests call it with `(".xml", ".rels")`. The calculation and table tests keep a differently named
wrapper (`_calc_parts`, `_table_parts`) because each merges a default sheet into the spec first. The oracle-canary
`_parts(xlsx: bytes)` unzips every member of already-built bytes and takes no spec, so it is renamed `_unzip` rather than
merged. `rg -n "def _parts" tools/tests` finds nothing; the 258 spec2xlsx and canary tests, ruff and mypy strict pass.
