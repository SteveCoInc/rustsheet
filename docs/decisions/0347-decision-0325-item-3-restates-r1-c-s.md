# 0347 — Decision 0325 item 3 restates R1(c)'s left-out rule exactly

- **Status:** accepted (2026-11-07, T-4681). Amends decision 0325 item 3.
- **Context:**
  - Decision 0325 item 3 says that once a probe cell's math is attached, the cell is Phase 1b evidence "when Excel
    and the engine are both within ε of it", and otherwise an ordinary visibly-off cell. Decision 0188 R1(c) is
    looser: a numeric cell is counted only when it fails Gate A's envelope, that is when the engine is neither within
    15 significant digits of Excel nor within ε = 1e-9 of the math (R2). Excel's own distance from the math does not
    decide whether the cell counts; it only decides `accuracy` against `visible` for a cell that still counts
    (decision 0114, the gap-triage rule).
  - The stricter text over-owned cells (harmless: a cell it sent to a `campaign:` task while R1(c) left it out only
    cost a task), but it read as a restatement of R1(c), and .claude/agents/gap-triager.md copied it ("Phase 1b
    evidence if Excel and the engine are within 1e-9 of it"). T-3994's review (PR #3727, excel-fidelity, MINOR)
    asked for the exact rule.
- **Decision:** decision 0325 item 3 now reads: once the math is attached, the cell is left out of the count when the
  engine is within ε of the math or within 15 significant digits of Excel (R1(c), Gate A's envelope); it is then
  Phase 1b evidence, needs no exit owner, and its deferred task keeps it. Otherwise it is an ordinary visibly-off
  cell and triage files its `campaign:` group as decision 0114 says: `accuracy` when Excel is the closer to the math,
  `visible` otherwise. Items 1, 2 and 4 of decision 0325 are unchanged.
- **Consequences:**
  - .claude/agents/gap-triager.md and the steward skill's "Deferred owners on the exit line" state the amended item 3.
  - No script or gate changes: `phase1_exit.py` and `exit_owners.py` already read R1(c) as written in decision 0188;
    only the prose that restated it changes.
