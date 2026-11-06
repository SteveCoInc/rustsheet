---
id: T-4484
title: "T.DIST density last digits after T-2585's Loader port: the 2,327 within-15-digit t2513 tdensity cells no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2586]
defer: phase-1b
---

## Goal
Every failing cell of the three t2513 Student t density probes below matches Excel to the bit, once T-2586 has
identified Excel's per-df Loader constant and its bd0 series. If it cannot, the Outcome records the bit-exact count per
case and why the rest cannot be derived.

## Context
- Gap triage 2026-11-06. `make eval` ran on origin/main d0ba57191 with every suite, calipers, poi and corpus-bench
  included. `make unowned-off ALL=1` and `make phase1-exit`'s owners line are clean.
- An ownership check over every failing cell of report.json leaves these 2,327 cells with no open owner. It used the
  `scripts/exit_owners.py` `owned` rule over every open task, deferred tasks included. Nothing else it found was
  unowned except the CUMPRINC underflow cells on the T-3141 branch, the t4045 family residue and pending cases that
  already have tasks.
- Why no one owns them: T-2585 (done) shipped the T density, `loader.rs` `excel_t_density`. Its Outcome says "The
  cells still off are T-2586's", but T-2586 is a research task and names none of them. So `make unowned-off` cannot
  see them. None is visibly off, so decision 0114 item 3 and decision 0188 R5 put this task under `defer: phase-1b`.
- Per case, cells that fail and that no task names (`make eval-show CASE=...`):
  - probes-numeric/t2513-r3-tdensity-a1 (1,445 cells). This task owns the whole case.
    - 834 T.DIST(t,n,FALSE) cells in column B, at df 3, 4 and 5, at most 5 ulp. For example B1
      `T.DIST(A1,3,FALSE)` is 0.3447399049150197 in Excel and 0.3447399049150196 in ours.
    - 609 F.DIST(t*t,1,n,FALSE) cells in column G, at most 6 ulp. For example G987 `F.DIST(A987*A987,1,5,FALSE)` is
      0.0008457106053927354 in Excel and 0.0008457106053927347 in ours.
    - 2 BETA.DIST cells in column H, 1 ulp each.
  - probes-numeric/t2513-r3-tdensity-hold2 (387 cells, column B, df 9 to 53, at most 4 ulp). This task owns the whole
    case. For example B795 `T.DIST(A795,41,FALSE)` is 0.03762071533481753 in Excel and 0.0376207153348175 in ours.
  - probes-numeric/t2513-verify-r3-tdensity-hold (495 cells, column B, df 13 to 50, at most 16 ulp, median 1). This
    task owns the whole case. For example B840 `T.DIST(A840,46,FALSE)` is 3.138383325379338e-05 in Excel and
    3.138383325379331e-05 in ours.
- The model and what is still open: statistics.md "Student t density is Loader's dt (T-2513)".
  - Excel's per-df constant sits ±1-2 units off the exact Loader constant (`loader_t_constant`).
  - The bd0 series is unresolved where x² < n/2.
  - It is unsettled whether the 1/sqrt(2π) product comes first near the mode (sw3 against sw2).
  - T-2586 item 3 owns the research and item 2 owns bd0. This task ships whatever they identify.
- Columns G and H of a1 are F.DIST and BETA.DIST, in the incomplete-beta family. Decision 0267 item 2 makes their
  14th- and 15th-digit misses Phase 1b residue, and item 4 gives Tier C and D no new work. This task owns them only so
  that they have an owner. Re-score them after the T change. If the T density work does not move them, list them as
  accepted family residue in the Outcome. Do not change the F density kernel for them (T-3174 and T-3231 own the F
  density's last digits).
- Code: `crates/rustsheet-engine/src/functions/statistical/loader.rs` `excel_t_density` (constant, bd0 held in a
  register, denominator order), reached through `student_t.rs` `t_density`.
- This is a numeric fit, so it is opus. The probes give 2,327 points, and T-2586's hold-outs already exist
  (t2513-r3-tdensity-hold2 and t2513-verify-r3-tdensity-hold were never fit data). Do not use them as the fresh sweep
  of R4.2. A fresh sweep needs a decision 0267 item 5 exception or a "Decision 0267 cells" section, because
  probes-numeric is frozen for new cases outside those tasks.

## Acceptance criteria
- [ ] The failing cells of the three cases above drop from eval/baseline/probes-numeric.txt (`make eval-baseline`)
      with no new known-failure line in any suite. Otherwise the Outcome gives the bit-exact count per case, before and
      after, and why the rest cannot be derived.
- [ ] The R4 guards of reviewer-excel-fidelity check 7 (decision 0188) hold for any fitted constant or form:
  - hold-out counts side by side with the training counts, the split fixed before scoring (decision 0188 R4.1);
  - a fresh oracle sweep scored once after the freeze (R4.2);
  - the Gate A envelope result (`rustsheet-eval accuracy` stays at 0 failing);
  - a `FITTED` tag with its range on every fitted constant (`scripts/check_fitted.py`);
  - the web-against-desktop reproducibility floor cleared, since most misses are 1 ulp;
  - no keep-matched branch, meaning no branch that only re-creates cells already matched.
- [ ] statistics.md "Student t density is Loader's dt (T-2513)" records what was found or refuted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T density cells that are visibly off or have the wrong type: T-3554 (`#NUM!` at a huge x) and T-4427 (`#DIV/0!` at
  df 1).
- The Student t cdf and its tails (T-0969, T-2930).
- Identifying the constant, which is T-2586's research. This task ships it.
