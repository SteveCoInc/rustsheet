---
id: T-3713
title: "runner test pending_case_that_panics_is_reported_as_panic reads Timeout under load: a panic's backtrace outlasts the 5 s case limit"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`runner::tests::pending_case_that_panics_is_reported_as_panic` (crates/rustsheet-eval/src/runner/tests.rs) passes on
a loaded machine. A panicking case is reported as `Panic`, never as `Timeout`.

## Context
- On 2026-11-01, during `make check` on a machine running a second `make check` and the sidecar generator, the test
  failed with `left: Timeout, right: Panic`, from the test's `assert_eq!(r.status, CaseStatus::Panic)`. The
  scripted engine's `panic!("boom")` did happen. With `RUST_BACKTRACE=1`, which this environment sets, capturing and
  symbolizing the backtrace took longer than the test's `Duration::from_secs(5)` case limit. `run_isolated`
  (runner.rs) decides the verdict with `rx.recv_timeout(timeout)`, and a panic that the worker sends after the
  deadline is reported as `Timeout`.
- Alone it passes in about 0.2 s (3 of 3 runs).
- A Timeout verdict for a real panic would also misreport a panicking eval case under load, not only in this test.
- Possible fixes:
  - classify a panic caught after the deadline as `Panic` when the worker reports it;
  - or set a panic hook that skips backtrace capture in the isolated runner;
  - or give the test a generous limit;
  - whichever matches the runner's contract.

## Acceptance criteria
- [x] The test passes with `RUST_BACKTRACE=1` under CPU load (for example while `cargo build` runs), or the runner
      reports a caught panic as `Panic` whatever the time.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other runner timeouts.

## Outcome
- The test's scripted engine now panics with `std::panic::resume_unwind(Box::new("boom"))`, which unwinds like
  `panic!` but skips the panic hook. The test harness keeps the default hook, whose backtrace capture under
  `RUST_BACKTRACE=1` was what outlasted the 5 s case limit; with no hook there is nothing slow before
  `catch_unwind` sees the payload. The test still checks the `Panic` verdict and the `boom` message.
- The runner itself is unchanged: the CLI installs a silent panic hook before it runs cases (`execute` and
  `cmd_accuracy` in main.rs), so a real eval run never captures a backtrace either, and the Timeout misreport
  the task's context worried about does not arise there.
- Demonstrated: with `RUST_BACKTRACE=1` and 12 busy loops on 4 cores, the test passed 10 of 10 runs (0.02 to 0.06 s).
- Eval delta: none (test-only change).
