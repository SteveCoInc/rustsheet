---
id: T-4779
title: "RATE and IRR with several sign changes: decide and implement their math value (69 counted cells skipped as no single value)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A decision record says what the 50-digit math value of a RATE or IRR cell is when its cash flows have several roots
in r > -1, or when every rate is a root (all flows 0 at one time). `rustsheet-accuracy math` then follows that record.
The 69 visibly-off cells below are skipped today as `no single value`, so decision 0188 R1(c) counts them only because
they have no math value (decision 0325). After this task they either get a value and are sorted by it, or the
record says why they keep counting and who owns them.

## Context
- Gap triage 2026-11-07 (`make eval` at b9ae5216f, excel-graph). The definitions raise NoValueError here
  (`tools/src/rustsheet_tools/accuracy/defs/annuity.py` `unique_rate`, used by RATE and by IRR's `flow_rate` in
  `cashflow.py`):
  - "several sign changes: no evidenced root choice": 60 cells (t2927 36, t3906 13, t3963 9, t3348 2);
  - "every flow is 0: every rate is a root": 9 cells (t3906 4, t3963 4, t3876 1). For example
    `RATE(1,10654003.4635,0,-10654003.4635,0,0.491103171463906)` is 0.49085317146390595 in Excel and
    0.490603171463906 in ours (t3876 Sheet1!A5).
  SPEC-0029 says (line 122) the solvers return "Excel's root by Excel's root choice where several exist ... with no
  evidenced choice, points with several roots are not drawn". That keeps them out of Gate A's tables, but eval cells
  still need a value or a stated reason.
- Why this needs opus: it is a design choice with an owner-visible consequence, decision 0188 R1(c)'s count. Options to
  weigh in the record:
  (a) the root that Excel's documented secant reaches from the guess (financial.md "IRR is a secant in d = 1 - v ...
      (fitted, 263/263 bit-exact)", "RATE is a secant on the FV residual"). This is the evidenced root choice SPEC-0029
      asks for, but it borrows the engine's own model, so the math is no longer independent of the fit;
  (b) the root nearest the guess (or nearest 0.1 when there is no guess), with a separation test: no value when two
      roots are closer than 1e-9 relative;
  (c) the root nearest Excel's value, used only to tell which root is meant, with a value only when that root is unique
      within 1e-6. This reads the oracle, which no definition does today;
  (d) no value, and an R1(c) rule or a recorded exception that these cells count until their owners close them.
  The "every rate is a root" cells cannot get a math value under (a) to (c). Decide them on their own (most likely (d),
  or Excel's iterate as the target with no math, decision 0267 item 4).
- Example cells: t3348 Sheet1!A138 `RATE(178,-22732.9134599,3932956.643633,53604.53074074,0)` is 0.00017159557792573998
  in Excel and 0.00017159557792151078 in ours (2.5e-11 apart, flows + - ... - +, two roots possible); t2927 P!A113
  `IRR({300,-2500,1004.9999999999998})`.
- Owners of the cells' fix work, which keep them: T-2928 (blocked, time-boxed) and T-3117 (t2927); T-3970 and T-4239
  (t3906, t3963); T-3424 (blocked), T-3591 and T-3607 (t3348); T-4431 (t3876).

Cells this task owns for their math value:
- probes/t3348-rate-fv-nonzero.case.json (2): Sheet1!A138, Sheet1!A180
- probes/t3876-rate-noise-residual-fresh.case.json (1): Sheet1!A5
- probes/t3906-rate-restart-oos.case.json (17): Sheet1!A14, Sheet1!A45, Sheet1!A65, Sheet1!A67, Sheet1!A79, Sheet1!A95, Sheet1!A96, Sheet1!A99, Sheet1!A101, Sheet1!A107, Sheet1!A109, Sheet1!A124, Sheet1!A126, Sheet1!A130, Sheet1!A161, Sheet1!A164, Sheet1!A228
- probes/t3963-rate-keep-r0-fresh.case.json (13): Calc!A25, Calc!A26, Calc!A27, Calc!A40, Calc!A105, Calc!A106, Calc!A107, Calc!A156, Calc!A157, Calc!A183, Calc!A184, Calc!A185, Calc!A207
- probes-numeric/t2927-irr-fallback-sweep.case.json (36): P!A32, P!A33, P!A34, P!A35, P!A36, P!A37, P!A38, P!A39, P!A40, P!A41, P!A42, P!A44, P!A45, P!A47, P!A113, P!A114, P!A115, P!A116, P!A117, P!A118, P!A119, P!A120, P!A121, P!A122, P!A124, P!A126, P!A128, P!A130, P!A132, P!A133, P!A134, P!A138, P!A139, P!A140, P!A142, P!A144

## Acceptance criteria
- [ ] A decision record (`make new-decision`) chooses the rule for several roots and the rule for "every rate is a
      root". It works through at least t3348 A138, t2927 P!A113 and t3876 A5, and states what R1(c) does with a cell the
      rule leaves without a value. SPEC-0029's solver paragraph is updated to match.
- [ ] `rustsheet-accuracy math` implements the rule for RATE and IRR (XIRR too if the rule is general, with a test). A
      unit test covers one two-root series and one all-zero series. `rustsheet-accuracy math --check` passes, and
      `rustsheet-accuracy verify RATE` and `verify IRR` show no change on the Gate A tables (whose points have one root).
- [ ] After `make eval`, the Outcome gives the counts: how many of the 69 cells got a value, how many are Phase 1b
      evidence, and how many still count, each with its owner. `eval/baseline/` gains no new line.
- [ ] `make check` green; reviewers approve (excel-fidelity included).

## Out of scope
- IRR cells that pass a guess and have one root (T-4778).
- Any change to the engine's RATE or IRR solver.
