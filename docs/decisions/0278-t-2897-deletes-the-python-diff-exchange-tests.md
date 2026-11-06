# 0278 — T-2897 deletes the Python diff-exchange tests

- **Status:** accepted (2026-11-04, T-2897). It applies SPEC-0026 decisions 1, 2 and 7.
- **Context:**
  - SPEC-0026 decision 1 rules that external diff lists are untrusted input, and its decision 2
    symbol table drops `UserModel.apply_external_diffs` and `UserModel.flush_send_queue` from the
    Python package with no warning release (decision 7, pre-v1). T-2897 removes them.
  - `bindings/python/tests/test_diffs.py` tests only those two methods (four tests: sync two
    models, styles, sheet operations, queue cleared after flush), and
    `bindings/python/docs/examples/collaboration.py` only demonstrates them. With the methods gone
    neither can run, and no rewrite tests anything the package still exposes.
  - The engine keeps `UserModel::apply_external_diffs` and `flush_send_queue` for Rust callers,
    and its own tests still cover them (`crates/rustsheet-engine/src/test/user_model/`, including
    `test_external_diffs_untrusted.rs` for crafted input; T-2896 owns hardening them).
  - `scripts/check_governance.py` fails a diff that deletes a test file without a decision
    record.
- **Decision:** T-2897 deletes `bindings/python/tests/test_diffs.py` and
  `bindings/python/docs/examples/collaboration.py`. In their place `tests/test_io.py` asserts that
  every removed name, `apply_external_diffs` and `flush_send_queue` included, is no longer
  reachable from `rustsheet`.
- **Consequences:**
  - The Python suite loses four tests of removed API and gains a test that the API stays removed.
    Diff exchange is tested only in Rust from now on.
  - Reviewers of T-2897's PR confirm the deletion covers only these two files. Any later deletion
    of a Python test still needs its own record.
