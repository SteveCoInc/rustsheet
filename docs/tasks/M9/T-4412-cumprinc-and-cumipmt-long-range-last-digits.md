---
id: T-4412
title: "CUMPRINC and CUMIPMT long-range last digits on T-3524's fresh sweeps: 362 cells within 15 digits that no task names, after T-3216 (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3216]
defer: phase-1b
---

## Goal
Each listed long-range cell (33 to 3,000 periods) of T-3524's two fresh sweeps returns Excel's double, or the Outcome
lists it with Excel, ours, the 60-digit reference (fit lab `kernels/cumulative.py`) and the reason Excel's last bits
cannot be derived. The cells are CUMPRINC, and the CUMIPMT built on it. Every one is already within half a unit of the
15th significant digit, so this is Phase 1b work only (decision 0114).

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 952336450). Ownership was checked with `scripts/exit_owners.py`'s
  `owned` over every failing cell, not only the visibly-off ones.
  - The tiny-rate sweep (rates 1E-12 to 1E-4, seed 35241) has 191 failing long-range cells (rows 251-500)
    that no open task names: 150 CUMPRINC (column A) and 41 CUMIPMT (column B).
  - The lead-fresh sweep (rates 1E-4 to 2, seed 3524) has 171 more of the same kind: 128 A and 43 B.
    That case is matched today only by T-4390's whole-case line, and T-4390 puts long ranges out of its scope.
    T-4390 lists only visibly-off cells, so this task shares none of them (split note, T-3229).
  - None of the 362 cells is visibly off.
  - Distance from Excel, tiny-rate sweep: 125 cells at 1 ulp, 40 at 2, 17 at 3-4, 10 at 6-16. Lead-fresh sweep: 124 at
    1 ulp, 26 at 2, 12 at 3-4, 9 at 6-16. The cells past 4 ulp are CUMIPMT. Since T-3118, CUMIPMT is `PMT·m -
    CUMPRINC`, so it carries the principal's ulps as many of its own (financial.md "CUMIPMT and CUMPRINC over many
    periods").
- Examples (Excel vs ours):
  - tiny A251 `CUMPRINC(5.34130702854326E-05,107,1910.503429,8,76,0)`: -1231.216516174731 vs -1231.2165161747312.
  - tiny B451 `CUMIPMT(2.31335185006963E-09,33442189,76.58508015,18,1100,1)`: -0.00019187012080587942 vs
    -0.000191870120805879.
  - lead-fresh A255 `CUMPRINC(0.00177938308233254,607,33.46634501,12,599,0)`: -32.410093462306364 vs
    -32.41009346230636.
- Root cause: CUMPRINC's own operation order for the long-range geometric sum. T-3524 removed the long-range cut
  (`LONG_GEOMETRIC_MIN_LEAD`), so every long range takes the geometric sum. On the fresh sweep's 500 long-range cells
  that form is bit-exact on 242 against 184 for the cut (T-3524 Outcome), but no form tried is Excel's to the bit.
  T-3216 owns the same open question, CUMPRINC's own last bits where v^nper does not underflow, on T-3067's probes
  (the fin-cumprinc form-boundary probes and others). This task depends on T-3216 and scores these 362 cells under
  whatever form T-3216 lands. A form chosen there cannot use these cells as its fresh sweep unless they were drawn
  after its freeze.
- Reproducibility floor (R4.4): most differences are 1-2 ulp, inside the 4-ulp web/desktop floor
  (docs/research/oracle-calibration.md), and these sweeps have one excel-graph fill each (2026-11-05). Refill both
  before fitting.
- The evidence is already cached: 500 long-range rows per sweep, both functions. A new fit needs a fresh sweep after
  its freeze (R4.2), drawn with `docs/research/t3524/gen_sweep.py` on a new seed and oracle-filled inside this task, so
  no separate research sweep task is needed first.
- Code: `crates/rustsheet-engine/src/functions/financial/cumulative.rs`. Read docs/research/excel-kernel-playbook.md
  first. Decision 0056 applies: after two attempts that do not raise the bit-exact count, time-box.
- Cells:
  - probes-numeric/t3524-cumprinc-tiny-rate-sweep.case.json (191): S!A251, A252, A253, A256, A258, A259, A263, A266, A267, A270, A271, A273, A275, A277, A281, A282, A283, A284, A286, A288, A289, A290, A292, A293, A295, A297, A299, A300, A301, A305, A307, A309, A310, A311, A312, A313, A314, A315, A317, A319, A323, A324, A325, A326, A328, A329, A331, A332, A333, A334, A336, A338, A339, A340, A341, A344, A346, A347, A348, A349, A350, A351, A353, A355, A356, A357, A358, A359, A361, A363, A364, A367, A369, A373, A374, A375, A376, A377, A378, A379, A384, A387, A388, A389, A391, A393, A394, A395, A396, A398, A402, A403, A405, A407, A410, A411, A412, A415, A416, A417, A418, A419, A420, A423, A424, A426, A428, A429, A431, A435, A436, A437, A439, A440, A446, A449, A450, A451, A452, A453, A454, A456, A457, A461, A462, A464, A465, A466, A468, A469, A470, A471, A472, A473, A474, A475, A477, A478, A480, A482, A484, A486, A487, A490, A492, A493, A494, A496, A499, A500, B252, B253, B259, B263, B273, B281, B282, B283, B293, B300, B305, B307, B310, B314, B317, B324, B325, B326, B339, B344, B361, B367, B384, B389, B391, B405, B407, B416, B417, B419, B423, B424, B440, B449, B451, B454, B466, B473, B477, B478, B484
  - probes-numeric/t3524-cumprinc-lead-fresh-sweep.case.json (171): S!A253, A255, A256, A258, A259, A260, A261, A264, A267, A268, A269, A272, A275, A276, A277, A282, A284, A286, A287, A288, A290, A291, A296, A297, A301, A303, A305, A306, A307, A311, A312, A313, A314, A317, A318, A320, A323, A325, A327, A328, A330, A332, A334, A336, A337, A339, A340, A342, A343, A345, A346, A347, A348, A353, A356, A359, A360, A361, A362, A364, A369, A370, A371, A372, A373, A377, A379, A381, A382, A383, A385, A386, A388, A389, A394, A396, A397, A401, A406, A409, A410, A412, A414, A415, A416, A419, A421, A424, A425, A427, A429, A431, A432, A434, A435, A436, A438, A439, A440, A441, A443, A444, A445, A446, A447, A449, A452, A456, A461, A463, A464, A466, A473, A474, A475, A477, A481, A485, A486, A489, A490, A492, A493, A495, A497, A498, A499, A500, B255, B264, B267, B272, B276, B277, B291, B297, B307, B313, B318, B323, B339, B340, B342, B348, B356, B359, B360, B362, B373, B382, B386, B388, B389, B396, B406, B407, B412, B421, B424, B425, B440, B443, B449, B452, B461, B463, B473, B475, B477, B492, B495

## Acceptance criteria
- [ ] After T-3216, a refill of both sweeps (excel-graph) reports how many listed cells changed bits. Cells that vary
      between fills stop being targets (R4.4, decision 0112) and are listed in the Outcome.
- [ ] One of these holds:
      - a long-range principal form makes the listed CUMPRINC cells bit-exact, with the CUMIPMT cells following through
        `PMT·m - CUMPRINC`;
      - or the Outcome lists every cell still off, with Excel, ours and the 60-digit value, and ranks the forms tried
        by bit-exact count (fit lab `cumulative-tiny` and `cumulative-fresh`).
- [ ] Any kept form or cut chosen by scoring passes every R4 guard of reviewer-excel-fidelity check 7 (decision 0188),
      each shown in the Outcome:
      - hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the
        candidate and for the correctly rounded reference, from the fit lab's T-3360 split, with the margin fixed
        before scoring;
      - a fresh log-uniform sweep, drawn and oracle-filled after the freeze, and its result;
      - the Gate A envelope count on that sweep, with every point outside the envelope listed;
      - a `FITTED(T-4412, n_cells, range)` tag, applied only inside its range;
      - no step that fits differences below the reproducibility floor;
      - no keep-matched branch.
- [ ] The listed cells leave the probes-numeric failing list. Because probes-numeric is gate = false, it has no
      eval/baseline file. No new failing cell appears in either sweep, and no eval/baseline/*.txt file gains a line
      (in particular probes.txt's fin-cumprinc-* and fin-cumipmt-* cases and ironcalc-calc's CUMPRINC and CUMIPMT
      workbook).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Short ranges, 32 periods or fewer (T-4390, T-4413); an underflowed v^nper (T-3141).
