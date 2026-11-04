# 0280 — make py-check tests against a dev rustsheet-eval build

- **Status:** accepted (2026-11-04, T-3714). It changes which build of `rustsheet-eval` the
  tools/ tests run against, not which tests run or what they assert. It amends the last bullet
  of decision 0233's Consequences.
- **Context:**
  - The python CI job's `make py-check` took 205 s on PR #3189 (run 41864419240, 2026-11-04);
    decision 0233 measured 199 s on run 41294866508. The tests themselves take about 40 s. The
    rest is the release `rustsheet-eval` build that `rustsheet_tools.snapshot.eval_binary`
    starts on demand when the first test needs the binary, while every xdist worker that reaches
    such a test waits on cargo's build lock.
  - `[profile.release]` has `lto = true`. Rebuilding the workspace crates of `rustsheet_eval` in
    release took 187 s locally (4 cores); turning LTO off did not help (234 s). A cold dev build,
    dependencies included, took 57 s.
  - Only 17 of the 1,632 tools/ tests need the binary (run with `RUSTSHEET_EVAL_BIN` pointing at
    a missing file). Against a dev binary they pass in the same time as against a release one
    (139 tests of the four affected files: 10 s vs 9 s with `-n auto`).
- **Decision:**
  - `make py-check` runs pytest with `RUSTSHEET_EVAL_BIN` set: the caller's own value if set,
    else `target/release/rustsheet-eval` if it exists, else `target/debug/rustsheet-eval`, which
    it builds first with `cargo build -q -p rustsheet_eval`.
  - `snapshot.eval_binary` is unchanged: the oracle and fitting tools run outside pytest and keep
    the release build.
- **Consequences:**
  - In CI (no release binary in the python job) the tests run against a dev build, which also
    shares dependency artifacts with `make py-bindings`' `maturin develop`.
  - `make check` builds the release binary before `py-check` (`eval-math-check: build`), so a
    full local check builds nothing more. As before, an existing release binary is used even if
    it is older than the sources.
  - Measured in CI: the python job's `make py-check` step took 72 s on this change (run
    41905116166) against 205-217 s on four PR runs of the same day.
  - A tools/ test that becomes too slow on a dev binary should cut its own work, not switch the
    whole job back to release.
