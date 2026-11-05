---
id: T-4406
title: "CUMPRINC and CUMIPMT last digits at tiny rates over a long range: 191 unowned t3524-cumprinc-tiny-rate-sweep cells within 15 digits on the geometric sum"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3216, T-4110]
defer: phase-1b
---

## Goal
CUMPRINC and CUMIPMT over a long range (33 periods or more, where the principal is the geometric sum or the closed
form) at rates from 1E-12 to 1E-4 return Excel's bits on the 191 cells of probes-numeric/t3524-cumprinc-tiny-rate-sweep
listed below. If they do not, the Outcome lists each form tried with its bit-exact count on them and says why none
reproduces Excel's digits.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 15371221a; every failing cell checked against every open task by
  `scripts/exit_owners.py`'s ownership rule): no open task names these cells. Every one is within half a unit of
  Excel's 15th significant digit, so the task is deferred to Phase 1b (decision 0114). It needs numeric fitting, so it
  is opus. T-4390 owns the case's visibly-off cells and says its long-range ones keep the geometric sum (T-3524).
- The case: docs/research/t3524/gen_sweep.py, seed 35241, rows 251-375 long type 0 and 376-500 long type 1 (33 to
  3000 periods log-uniform, leads log-uniform in [1E-3, 40]). Column A is CUMPRINC, B CUMIPMT. Math values (mpmath
  60) are in docs/research/t3524/gate_a.tsv.
- Measured on the 191 cells: 182 NumericUlp and 9 NumericClose. Ours and Excel differ by a median of 1 ulp, with a
  maximum of 16 on CUMIPMT. Excel is correctly rounded on 48 of the 191 cells and ours on 42. The median distance from
  the math is 1.07 ulp for Excel and 1.11 for ours. So the target is Excel's operation order, not the math.
- Examples (`make eval-show CASE=probes-numeric/t3524-cumprinc-tiny-rate-sweep.case.json`; Excel, ours, math):
  - S!A251 `CUMPRINC(5.34130702854326E-05,107,1910.503429,8,76,0)`: -1231.216516174731, -1231.2165161747312,
    -1231.2165161747308335 (Excel 0.47 ulp, ours 1.47).
  - S!A252 `CUMPRINC(8.63603865548707E-05,1571,32.22467317,28,1557,0)`: -31.40005380931875, -31.400053809318752,
    -31.400053809318756267.
  - S!B339 `CUMIPMT(2.14059600670365E-10,374537157,9.915562416,1,98,0)`: -2.080070645865527E-07,
    -2.0800706458655312E-07, -2.080070645865533297E-07 (16 ulp apart; Excel 23.8 ulp from the math, ours 7.8).
    CUMIPMT is `PMT·m - CUMPRINC` (T-3118), so its ulps follow the principal's and PMT's. Fix the principal first.
- Same root cause as T-3216 (CUMPRINC's own last bits where `v^nper` does not underflow, at rates 1E-5 to 2): Excel's
  operation order for the long-range geometric principal is not known. T-3216 found that with Excel's own PMT bits
  the best geometric form is exact on 22 of 58 type 1 cells. This task extends that search to tiny rates, where
  `ln(1+rate)` and `expm1` sit at the bottom of their range. Hence the dependency: take T-3216's ranking first.
- Cells (probes-numeric/t3524-cumprinc-tiny-rate-sweep, 191):
  - probes-numeric/t3524-cumprinc-tiny-rate-sweep S!A251, A252, A253, A256, A258, A259, A263, A266, A267, A270,
    A271, A273, A275, A277, A281, A282, A283, A284, A286, A288, A289, A290, A292, A293, A295, A297, A299, A300, A301,
    A305, A307, A309, A310, A311, A312, A313, A314, A315, A317, A319, A323, A324, A325, A326, A328, A329, A331, A332,
    A333, A334, A336, A338, A339, A340, A341, A344, A346, A347, A348, A349, A350, A351, A353, A355, A356, A357, A358,
    A359, A361, A363, A364, A367, A369, A373, A374, A375, A376, A377, A378, A379, A384, A387, A388, A389, A391, A393,
    A394, A395, A396, A398, A402, A403, A405, A407, A410, A411, A412, A415, A416, A417, A418, A419, A420, A423, A424,
    A426, A428, A429, A431, A435, A436, A437, A439, A440, A446, A449, A450, A451, A452, A453, A454, A456, A457, A461,
    A462, A464, A465, A466, A468, A469, A470, A471, A472, A473, A474, A475, A477, A478, A480, A482, A484, A486, A487,
    A490, A492, A493, A494, A496, A499, A500, B252, B253, B259, B263, B273, B281, B282, B283, B293, B300, B305, B307,
    B310, B314, B317, B324, B325, B326, B339, B344, B361, B367, B384, B389, B391, B405, B407, B416, B417, B419, B423,
    B424, B440, B449, B451, B454, B466, B473, B477, B478, B484
- Code: functions/financial/cumulative.rs `principal` and `geometric_principal`. A long range takes the geometric sum
  at any lead, then the closed form. T-3524 removed `LONG_GEOMETRIC_MIN_LEAD` because on this case the cut got only
  71 of 238 cells to the bit. annuity.rs (`discounted_payment`, PMT's `1-Q`, T-3014). Read docs/research/excel-kernel-playbook.md first.
  Decision 0056's time-box applies.

## Acceptance criteria
- [ ] Each of the 191 cells above passes, or financial.md "T-3067 notes" lists every candidate form tried with its
      bit-exact / within-4-ulp / within-15-digits counts on them, next to today's counts.
- [ ] probes-numeric is not gated (`gate = false`), so the Outcome gives the case's before and after counts
      (`rustsheet-eval show --all`) and lists any of its cells that leave 15 digits. There must be none. No new
      known-failure lines appear in any eval/baseline/<suite>.txt (`make eval-check` 0 regressions; `make
      eval-baseline` records the fixes). The gated CUMPRINC_CUMIPMT.xlsx, gen-m9 and the probes/fin-cumipmt-* and
      fin-cumprinc-* cases lose no passing cell.
- [ ] A form or selector chosen by scoring oracle cells meets the decision 0188 R4 guards of reviewer-excel-fidelity
      check 7, shown in the Outcome:
      - hold-out: train and hold-out counts (bit-exact, within 4 ulp, within 15 digits) side by side for the chosen
        candidate and the correctly rounded one, on a split fixed before scoring (about 30% by cell hash), with its
        margin;
      - fresh sweep: an `r4-T-4406` batch of CUMPRINC's and CUMIPMT's Gate A tables (SPEC-0029 "The R4 guards";
        the tables are T-4110's, hence the dependency), log-uniform over rates 1E-12..1E-4 and long ranges, generated
        after the fit is frozen and oracle-filled before merge. Decision 0267 item 5 allows no new probes-numeric
        case for this task, and no table batch may feed the fit;
      - Gate A envelope: the fresh sweep's count inside the envelope, every point outside listed;
      - `FITTED(T-4406, n_cells, range)` on each fitted piece, applied only inside its range, accurate math outside;
      - reproducibility floor: no step fits differences below the oracle's floor (web against desktop up to 4 ulp,
        bits that vary between fills, decision 0112);
      - no keep-matched branch (decisions 0056 and 0082).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The short-range rows 1-250 of the same case (T-4405); its visibly-off cells (T-4390); underflowed principals
  (T-3141).
