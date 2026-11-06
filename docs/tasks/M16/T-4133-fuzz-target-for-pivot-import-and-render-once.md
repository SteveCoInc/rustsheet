---
id: T-4133
title: "Fuzz target for pivot import (and render once it exists)"
milestone: M16
lane: quality
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A cargo-fuzz target feeds hostile pivot parts (indices, items, rowItems, groups, formulas, huge counts) to the importer, and later the renderer: no panic, bounded work.

## Context
Directive 5. SPEC-0015 limits; SPEC-0032 adds render limits. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Fuzz target committed with a seed corpus from the builder.
- [x] A short CI run (or documented local run) shows no crash.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing findings beyond trivial ones (file tasks).

## Outcome
- New fuzz target `fuzz/fuzz_targets/pivot_import.rs`. Its input is the three pivot parts (cache definition, cache records,
  table definition) separated by NUL bytes. They replace the first pivot's parts of
  crates/rustsheet-xlsx/tests/pivot_tables.xlsx, and the package is rebuilt in memory, so mutations land in the pivot
  reader rather than in a deflated zip stream. The workbook is then loaded through `Workbook::from_xlsx_bytes` with
  `xlsx_import`'s limits, four GETPIVOTDATA calls read the pivot model, every used range is read, and the workbook is saved.
- Seed corpus: `fuzz/seeds/pivot_seeds.rs` writes 12 seeds, run by `fuzz/seed_corpus.py`. They are the fixture's two
  pivots, plus ten hostile edits of the first: a 2^32-1 record count, an 11-digit recordCount, a shared-item index of
  4,000,000,000, a negative item index, a `-2` row field, a data field past the cache, a reversed full-sheet
  location, a numeric fieldGroup with a 1E-300 interval over 0..1E+300, a calculated field, and a page field with
  item 99. The builder fails if an edit no longer matches its fixture.
- Nightly: `.github/workflows/fuzz.yml` gains `pivot_import` (`-max_len=65536`), decision 0312. web.md's SPEC-0015
  importer entry names the target.
- Local run (no nightly toolchain here, so the target was built with stable `cargo build --release` in fuzz/: libFuzzer
  with no coverage instrumentation). It replayed all 12 seeds (`-runs=0`), then mutated for 240 s (`-max_len=65536
  -timeout=30 -rss_limit_mb=2048`): 126,153 runs, no crash, no timeout, peak RSS 37 MB. The nightly job runs the
  instrumented build.
- Render: SPEC-0032's renderer does not exist yet. The target's doc and decision 0312 note that it should also render
  the pivot once it lands.
- Eval delta: none (fuzz tooling only).
