---
id: T-5046
title: "Drop the round-trip ratchet's legacy per-case baseline path now every baseline is keyed by scenario"
milestone: M16
lane: harness
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The round-trip ratchet has one baseline form, keyed `<case>#<scenario>` (decision 0345), and no migration path left
in the code; the gate's documentation says the per-scenario rule.

## Context
T-4961 (#4007) keyed every round-trip baseline by case and scenario and migrated all five committed baselines. To
migrate without accepting a regression, `crates/rustsheet-eval/src/roundtrip.rs` still judges a baseline without `#`
in its keys with the old per-case rule (`by_scenario::is_legacy`, `roundtrip::baseline`). No committed baseline is
in that form any more, and a new suite's baseline is written in the new form, so the branch is dead weight that the
next reader must reason about. `docs/process/quality-gates.md`'s round-trip row still reads "a case may not compare
less", which decision 0345 made per case and scenario. #4007 promised this follow-up.

## Acceptance criteria
- [x] `roundtrip.rs` judges every baseline per case and scenario; `is_legacy` and the per-case `baseline` (with
      its tests that exist only for the migration) are gone or kept only where a remaining caller needs them,
      and a baseline without `#` keys is refused with a clear error rather than judged per case.
- [x] `docs/process/quality-gates.md`'s round-trip row says a case's scenario may not compare less or stop running.
- [x] `rustsheet-eval roundtrip --check` passes unchanged on every committed baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the cells-mode suites' per-case rule (decision 0345 keeps it).

## Outcome
- `roundtrip::baseline` (the per-case form) and `by_scenario::is_legacy` are gone. The ratchet writes and judges
  every round-trip baseline with `by_scenario::baseline`, whose doc now carries the `cells` and `cases` rules.
- `roundtrip::read_baseline` refuses a committed baseline with a key without `#`, naming the file and the key
  (`by_scenario::per_case_key`), rather than judging it per case. A unit test covers a `C` key and an `F` key.
- The tests that compared the old rule with the new one went with it. The round-trip tests now build their
  baselines per scenario, so their regression keys read `rt/a.xlsx#R0`.
- `docs/process/quality-gates.md`'s round-trip row says a case's scenario may not compare less or stop running.
  Decision 0345 gains a Consequences bullet (the governance record for that protected file).
- `rustsheet-eval roundtrip --check` passes unchanged on every committed round-trip baseline.

## Handoff (2026-11-08)
Work stopped before the PR. The branch `ccr-56acd608-c5kt9z-t5046` holds the finished change, based on main at 6f0c3d8e5; rebase it before opening the PR.
- Reviews: reviewer-design APPROVE (its minor on decision 0345's migration bullet is fixed), reviewer-security APPROVE (minor: `per_case_key` checks only the `C`/`F` maps; the round-trip ratchet never reads `TC`/`TF`/`LC`/`LF` lines, so it stays as is).
- Verified: `cargo test -p rustsheet_eval` and clippy `-D warnings` pass, as do fmt, governance (1 protected file, recorded in decision 0345), tasks validate, duplication and file size. `rustsheet-eval roundtrip --check` passes unchanged on roundtrip-fixtures, -excel-saved, -poi and -corpus-bench. roundtrip-calipers reported only "baseline is out of date", with 0 regressions: that run scored the two pv26 cases that usually time out. Calipers is nightly-only.
- Left to do: rebase onto main, run `make check-fast`, open the PR (review table above), and enable auto-merge.
