# 0276 — Tests build the engine at opt-level 1

- **Status:** accepted (2026-11-04, T-3709). It revisits the opt-level option that decision 0122
  measured as a net loss; the shards of 0122 and 0232 are unchanged.
- **Context:**
  - `test::test_tailed_array` checks every tailed formula against the ordinary reading over
    1,048,576-row arrays, in a debug build. On 2026-11-04 (4 threads, this container) it took
    679 test-s, 186 s wall; `make test` took 512 s wall (compile 114 s, the engine's unit tests
    359 s).
  - Stack samples (gdb, `sumproduct_of_several_arguments`: 1.4 s tailed, 35.7 s full for 13
    formulas) show the full reading spread over the generic evaluator (operators, function
    calls, array collection): no single hotspot that a test-side change or one engine fix
    removes. Sharing one model per formula is already done (one per test and reading).
  - 0122 measured opt-level 1 at compile +127 s, tests -110 s when the suite was 585 CPU-s. The
    engine's tests are now about 1,400 CPU-s, so the trade has turned:

    | `[profile.test.package.rustsheet_engine]` | compile | engine tests | `make test` |
    |---|---|---|---|
    | none (opt-level 0, full debuginfo) | 114 s | 359 s | 512 s |
    | `opt-level = 1` | 223 s | 159 s | 415 s |
    | `opt-level = 1`, `debug = "line-tables-only"` | 151 s | 192 s | 377 s |

    With the last row `test_tailed_array` takes 317 test-s, 85 s wall (from 679 and 186).
- **Decision:** `Cargo.toml` sets `opt-level = 1` and `debug = "line-tables-only"` for the
  `rustsheet_engine` package in the `test` profile. Only `cargo test` builds change: `cargo
  build`, clippy, the release eval binary and the fuzz targets keep their profiles. Debug
  assertions and overflow checks stay on (opt-level does not change them), so the tests check
  what they checked before.
- **Consequences:**
  - The same tests run, each once, in the same shards; `make test` is about a quarter shorter
    locally. CI's shards each compile the engine at opt-level 1; the PR's run records the shard
    times, and the steward's rebalance rule (0122, 0232) applies as before.
  - A panic's backtrace in a test still has file and line numbers; a debugger session on the
    test binary sees fewer local variables. Drop the override locally (or build with
    `--profile dev`-style flags) when stepping through a test.
  - When the suite's balance changes, re-measure the three rows above before changing the
    override.
