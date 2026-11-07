---
id: T-4405
title: "CUMPRINC and CUMIPMT last digits at tiny rates over a short range: 190 unowned t3524-cumprinc-tiny-rate-sweep cells within 15 digits (T-4390 follow-on)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4390, T-4110]
defer: phase-1b
---

## Goal
CUMPRINC and CUMIPMT over a short range (at most 32 periods, `CUMULATIVE_LOOP_PERIODS`) at rates from 1E-12 to 1E-4
return Excel's bits on the 190 cells of probes-numeric/t3524-cumprinc-tiny-rate-sweep listed below, or the Outcome
lists each form tried with its bit-exact count on them and says why none reproduces Excel's digits.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 15371221a; every failing cell checked against every open task by
  `scripts/exit_owners.py`'s ownership rule): no open task names these cells. T-4390 owns the 140 visibly-off cells of
  the same case under its own rule, not these. Every cell here is within half a unit of Excel's 15th significant
  digit (the report's `--all-off` list is empty), so the task is deferred to Phase 1b (decision 0114). It needs
  numeric fitting, so it is opus.
- The case: docs/research/t3524/gen_sweep.py, seed 35241, the same draw as t3524-cumprinc-lead-fresh-sweep at rates
  log-uniform in [1E-12, 1E-4]. Rows 1-125 are short ranges of type 0, rows 126-250 short ranges of type 1. Column A
  is CUMPRINC and column B CUMIPMT, with the same arguments. 50-digit (mpmath 60) values for every cell are in
  docs/research/t3524/gate_a.tsv (`mpmath_60` column).
- Measured on the 190 cells: 176 NumericUlp and 14 NumericClose. Ours and Excel differ by a median of 2 ulp, with a
  maximum of 16 ulp on CUMIPMT. Excel is not correctly rounded here. It is correctly rounded on 43 of the 190 cells and
  ours on 51, and the median distance from the math is 1.17 ulp for Excel and 1.22 ulp for ours. So the target is
  Excel's operation order (an `accuracy` stage would not move these cells), and the work is a fit.
- Examples (`make eval-show CASE=probes-numeric/t3524-cumprinc-tiny-rate-sweep.case.json`; Excel, ours, math):
  - S!A10 `CUMPRINC(3.16465044916096E-07,44471,15887.03498,69,76,0)`: -2.837959739224426, -2.8379597392244267,
    -2.8379597392244270154 (Excel 2.75 ulp from the math, ours 0.75).
  - S!A2 `CUMPRINC(1.73797338437276E-05,25313,5073.681369,81,97,0)`: -2.7168801112792225, -2.716880111279223,
    -2.7168801112792226452 (here Excel is the correctly rounded one).
  - S!B196 `CUMIPMT(2.19177236876901E-06,64685,3778792.145,20,38,1)`: -157.29904264943298, -157.29904264943343,
    -157.29904264943347567 (16 ulp apart; ours 1.6 ulp from the math, Excel 17.6). CUMIPMT is `PMT·m - CUMPRINC`
    (T-3118), so the interest carries an ulp of the principal or of PMT as many of its own ulps; fix the principal
    first.
- Cells (probes-numeric/t3524-cumprinc-tiny-rate-sweep, 190):
  - probes-numeric/t3524-cumprinc-tiny-rate-sweep S!A2, A4, A5, A7, A9, A10, A11, A12, A15, A17, A18, A21, A22, A23,
    A26, A30, A31, A32, A33, A35, A36, A37, A38, A39, A44, A46, A49, A50, A52, A53, A54, A55, A56, A57, A58, A59, A60,
    A61, A63, A65, A66, A67, A69, A70, A73, A75, A78, A81, A82, A85, A86, A87, A89, A90, A91, A92, A94, A97, A98, A99,
    A100, A101, A102, A108, A109, A111, A112, A115, A116, A117, A120, A121, A122, A124, A126, A127, A129, A131, A132,
    A133, A137, A138, A139, A140, A141, A142, A143, A144, A146, A148, A150, A154, A155, A158, A160, A161, A162, A163,
    A164, A165, A166, A169, A171, A173, A174, A176, A180, A181, A182, A184, A185, A186, A192, A193, A194, A195, A196,
    A197, A201, A204, A205, A208, A209, A210, A211, A213, A214, A215, A216, A218, A220, A221, A222, A223, A224, A225,
    A226, A229, A230, A236, A240, A242, A243, A244, A246, A247, A248, A250, B2, B18, B21, B22, B23, B30, B35, B38,
    B46, B56, B57, B58, B63, B66, B67, B75, B94, B97, B117, B124, B133, B138, B143, B158, B165, B171, B174, B176,
    B180, B184, B185, B192, B194, B196, B201, B208, B210, B213, B221, B222, B225, B230
- Code: functions/financial/cumulative.rs, the short forms (per-period sum for type 0, closed form for type 1, the
  geometric sum above `GEOMETRIC_MIN_LEAD` and in `short_takes_geometric`'s band, both `FITTED`). T-4390 is testing
  candidate operation orders for the same short-range forms (a PPMT sum built as PMT less IPMT, the order of
  `pmt·series·v^lead`, Excel's own `ln(1+rate)`). Start from its result. If its winner also fits these cells, this
  task is only the measurement.
- Read docs/research/excel-kernel-playbook.md first. Decision 0056's time-box applies: two attempts that do not raise
  the bit-exact count, then stop and record the negatives in financial.md "T-3067 notes".

## Acceptance criteria
- [ ] Each of the 190 cells above passes, or financial.md lists every candidate form tried with its bit-exact /
      within-4-ulp / within-15-digits counts on them, next to today's counts.
- [ ] probes-numeric is not gated (`gate = false`), so the Outcome gives the case's before and after counts
      (`rustsheet-eval show --all`: passed, NumericUlp, NumericClose) and lists any of its cells that leave 15 digits.
      There must be none. No new known-failure lines appear in any eval/baseline/<suite>.txt (`make eval-check` 0
      regressions; `make eval-baseline` records the fixes). The gated CUMPRINC_CUMIPMT.xlsx, gen-m9 and the
      probes/fin-cumipmt-* and fin-cumprinc-* cases lose no passing cell.
- [ ] A form or selector chosen by scoring oracle cells meets the decision 0188 R4 guards of reviewer-excel-fidelity
      check 7, shown in the Outcome:
      - hold-out: train and hold-out counts (bit-exact, within 4 ulp, within 15 digits) side by side for the chosen
        candidate and the correctly rounded one, on a split fixed before scoring (about 30% by cell hash), with its
        margin;
      - fresh sweep: an `r4-T-4405` batch of CUMPRINC's and CUMIPMT's Gate A tables (SPEC-0029 "The R4 guards";
        the tables are T-4110's, hence the dependency), log-uniform over rates 1E-12..1E-4 and short ranges, generated
        after the fit is frozen and oracle-filled before merge. Decision 0267 item 5 allows no new probes-numeric
        case for this task, and no table batch may feed the fit;
      - Gate A envelope: the fresh sweep's count inside the envelope, every point outside listed;
      - `FITTED(T-4405, n_cells, range)` on each fitted piece, applied only inside its range, accurate math outside;
      - reproducibility floor: no step fits differences below the oracle's floor (web against desktop up to 4 ulp,
        bits that vary between fills, decision 0112);
      - no keep-matched branch (decisions 0056 and 0082).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The long-range rows 251-500 of the same case (T-4406).
- The visibly-off cells of this case (T-4390) and type 0's underflowed principal (T-3141).
