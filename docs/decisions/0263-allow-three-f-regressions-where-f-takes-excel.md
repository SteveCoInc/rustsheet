# 0263 — Allow three F regressions where F takes Excel's r >= 1 beta arguments

- **Status:** accepted (2026-11-04, T-3580)
- **Context:**
  - T-3580 gives F.DIST(., TRUE), F.DIST.RT and FDIST the incomplete-beta arguments Excel forms (r >= 1,
    statistics.md "F's incomplete-beta arguments keep r >= 1", found by T-3567) and puts F at a df of 1 on bgrat at
    b = 1/2, as the Student t and BETA.DIST already are. This is a structural model (reviewer-excel-fidelity check 7
    (c)) frozen before its fresh sweep was drawn.
  - Hold-out probes-numeric t3567-r5-fdf1-hold (3,578 cells, built and filled by T-3567 before this model was
    written): bit-exact / within 4 ulp / within 15 digits 2,879/3,430/3,487 against main's 1,984/2,850/3,029 and the
    correctly rounded value's 1,407/2,484; no cell goes from within 15 digits to visibly off, 458 the other way. Fresh
    sweep probes-numeric t3580-f-fresh (2,400 cells): 1,867/2,280/2,335 against the correctly rounded 932/1,718/1,872;
    Gate A's envelope holds every point of it and of the F.DIST batch r4-T-3580.
  - Gated suites: gen-m9 fixes 18 cells and probes 20. Three gated cells (two distinct inputs) that matched now miss
    by one or two ulp, both in branches that run bup, whose arithmetic on Excel's arguments T-3567 left open:
    - gen-m9/c18dbf21f11093f3 A151 `FDIST(Data!$D5,1,9)` and gen-m9/7c20351562a2d71d A205
      `FDIST(COUNTIF(Data!C8:D11,">0"),1,9)`, both `FDIST(2,1,9)` (bup+bgrat at b = 1/2): Excel 0.190947117169724,
      now 0.19094711716972404. T-3567 already found the research kernel 1 ulp off here (RESULTS.md section 2, `c4.py`).
    - gen-m9/a790a4d4a6a82756 A121 `F.DIST.RT(Data!$E12,7,9)`, x = TRUE = 1 (L140 bup+bpser): Excel
      0.48796816908209784, now 0.48796816908209795.
    The old arguments with today's kernel happened to hit these bits. Per branch on the hold-out, Excel's arguments
    still win: bup+bgrat 65/98 against 31/98 bit-exact (old: today's kernel) and 48/75 (old arguments with bgrat),
    L140 bup+bpser 220/293 against 204/270 (bit-exact / within 4 ulp of 294).
  - The task's acceptance asked for `FDIST(2,1,9)` to stay at Excel's value. Keeping it needs one of the options
    below, each of which the evidence rules out.
- **Options considered:**
  - *Keep today's kernel for bup+bgrat at b = 1/2, or the old arguments for L140 bup+bpser.* Each is a branch chosen
    because it keeps the previously matching cells, the keep-matched branch decisions 0056 and 0082 forbid, and each
    loses on the hold-out (34 and 16 bit-exact cells).
  - *Model bup's arithmetic on these arguments.* That is open research (T-3567 RESULTS.md "bup's arithmetic, open";
    T-3582 owns bgrat at b != 1/2), not something to fit to two cells.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3580 PR, for exactly these three
  cells.
- **Consequences:** `eval/baseline/gen-m9.txt` gains these three lines and loses the 18 fixed ones; reviewers confirm
  the baseline diff adds no other line. The cells come back when bup's arithmetic on F's arguments is modelled.
