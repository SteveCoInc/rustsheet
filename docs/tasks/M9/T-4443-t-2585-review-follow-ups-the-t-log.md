---
id: T-4443
title: "T-2585 review follow-ups: the t log-branch cut beyond its measured extremes, the flush test's evidence pointer, decision 0299's waiver date"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2585]
---

## Goal
The T density's fitted cuts, its subnormal flush test and decision 0299 state their evidence and scope fully: no reader can take the cuts as applied beyond their evidence without the argument that it cannot matter, or the R4.1 waiver as a precedent.

## Context
MINOR findings of T-2585's step-5c review (PR #3422, all three reviewers approved 7ab3c7a):
- excel-fidelity: `T_LOG_BRANCH_CUT` and the reused `EXCEL_BD0_CUT` (functions/statistical/loader.rs) run for every
  df >= 3 where t·t is finite, past the tag's measured extremes (df 3..2.7E297, |t| up to 1.1E153). Both cuts only
  matter at xx between about 0.2n and n, where the density underflows to 0 on either branch once n is above about
  1E4; record that argument in decision 0299 (or the tag comment), or clamp the Loader path to the measured box.
- excel-fidelity: the df >= 3 flush (`t_density_flushes_a_subnormal_result`, test_fn_t_dist.rs) rests on Gate A
  T.DIST/r4-T-2585 A138 plus scratch probes; point the test's comment at T-4427, which commits the probe case.
- excel-fidelity: decision 0299's R4.1 waiver should say why it is allowed: the cut was frozen on 2026-10-24, before
  decision 0188 R4 (2026-10-31), so it is no precedent for later fits.
- design: `T_LOG_BRANCH_CUT`'s doc comment says twice that the cut applies over the whole range; merge the sentences.
  Optionally note in `student_t_density`'s doc that T.DIST goes through `t_density`.
- security: the test module's `#![allow(clippy::unwrap_used, reason = ...)]` in loader.rs is redundant with
  clippy.toml's allow-unwrap-in-tests (a repo-wide pattern; drop it only if that is the repo's direction).

## Acceptance criteria
- [ ] Decision 0299 (or the tag) carries the underflow argument for the region past the measured extremes, or the
      Loader path is clamped to the measured box with tests.
- [ ] The flush test cites T-4427; decision 0299 states the freeze date as the waiver's reason.
- [ ] `T_LOG_BRANCH_CUT`'s doc comment says its range once.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
