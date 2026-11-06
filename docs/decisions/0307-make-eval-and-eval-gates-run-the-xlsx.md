# 0307 — make eval and eval-gates run the xlsx round-trip suites

- **Status:** accepted (2026-11-06, T-4172)
- **Context:**
  - SPEC-0033 (T-4167) defines round-trip fidelity suites. T-4172 adds them: `rustsheet-eval roundtrip` and the first
    suite, `eval/roundtrip/fixtures`.
  - A suite only protects what it gates. The Makefile is a protected path, so wiring the command into the gates
    needs this record.
- **Decision:**
  - `make eval` runs `rustsheet-eval roundtrip` over every round-trip suite and writes
    `eval/runs/latest/roundtrip.{json,md}`.
  - `make eval-gates`, and so CI's eval job and `make check`, runs `rustsheet-eval roundtrip --check`. It fails when
    a gate round-trip suite regresses a fidelity item or its baseline is out of date.
  - `make eval-baseline` runs `rustsheet-eval roundtrip --update-baseline`, which refuses regressions as the cell
    ratchet does.
- **Consequences:**
  - A change that loses a part the export kept before, or makes the output fail the package validator, fails CI.
  - A change that fixes one needs `make eval-baseline` and a commit of `eval/baseline/roundtrip-*.txt`, as cell
    fixes do.
  - The fixtures round-trip suite adds about a second to `eval-gates`.
  - `--update-baseline` has no `--allow-regressions`. A round-trip regression needs a fix, or a decision that adds
    the flag.
