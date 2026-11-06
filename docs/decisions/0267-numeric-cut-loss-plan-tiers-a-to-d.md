# 0267 — Numeric cut-loss plan: tiers A to D, a 13-digit bar for the incomplete-beta family, then freeze probes-numeric

- **Status:** accepted (2026-11-04, owner rulings; recorded on branch claude/numeric-accuracy-strategy-ceaedp).
- **Amended by:** decision 0301 (item 3's tiny-p scope narrows; NPER and XIRR get no new R3 region).
- **Amends:**
  - decision 0083, for the incomplete-beta family only: the bar becomes 13 digits for the headline and the Phase 1
    counts (item 2 below);
  - decision 0114 item 3 and decision 0188 R5: `defer: phase-1b` also parks family work whose cells are wrong only
    at the 14th or 15th digit (item 4);
  - decision 0188 R3: the owner ruling for the tiny-p quantiles (item 3).
- **Context:** the owner asked for an honest breakdown of the 90,103 visibly-off numbers (docs/progress.md,
  2026-11-03), of which probes-numeric holds 89,493. The analysis:
  - `rustsheet-eval run --suite probes-numeric` at main 985d511b, using the same gate rule as `within_fifteen_digits`;
  - mpmath at 50 to 60 digits on a sample;
  - the research record (statistics.md; decisions 0114, 0188, 0056; the beta-family research tasks).

  Findings:
  - **Where the first wrong digit is.** Of 89,166 visibly-off numbers:
    - 61,874 (69.4%) first differ at the 15th significant digit;
    - 25,216 (28.3%) at the 14th;
    - 1,623 at the 13th and 159 at the 12th;
    - 294 at the 11th or earlier.

    394 more attributed cells have the wrong type or error. 174 cells are composite and unattributed: 14 of them are
    `#NUM!` behind a BETA.INV error, 1 is at the 13th digit, the rest at the 14th or 15th.
  - **Which functions.** BETA.DIST, F.DIST, F.DIST.RT, FDIST, BINOM.DIST and NEGBINOM.DIST hold 86,821 of the
    numbers (97.4%), across 119 cases. 99.7% of all the cells are in `t####` research sweeps: fit, hold, fresh and
    verify copies of the hardest regions.
  - **Excel is the noisy one there.** On 2,026 sampled visibly-off family cells with literal arguments:
    - Excel is correctly rounded on 20 (1%);
    - Excel's median error is 48 to 330 ulp per function, ours 0.7 to 240;
    - ours is closer to the truth on 72%. On BINOM.DIST and NEGBINOM.DIST ours is mostly correctly rounded and Excel
      is 48 to 66 ulp off.

    Five research rounds on Excel's TOMS 708 port each closed one branch. What is left grows with the term count and
    looks like Excel's own noise: bgrat's increment, bup with 10 or more terms, bfrac past 10 terms, the F density
    (statistics.md; T-3582, T-3568, T-3571, T-2586). Decision 0188 had already measured that a correctly rounded
    kernel stays visibly off on most of these cells.
  - **Real files are close.** Calipers is 99.99% bit-exact over 4.4M cells. Corpus-bench has a visibly-off cell in
    1 of 320 workbooks, poi in 3 of 115.
  - **The tiny-p quantiles.** Excel is more than G = 1e-4 off the math on 50 of the 59 visibly-off tiny-p cells of
    T.INV, T.INV.2T and TINV. For example `T.INV.2T(1E-300,100000)` is 79.12 in Excel and 37.19 in the math and the
    engine. CHISQ.INV.RT at df 1, p 1.5E-16 to 5E-16, is 0.4% to 1.1% off.
  - **One Tier A cluster is our bug.** t4017 at df2 1E7 to 1E10: Excel is within about 1E-13 of the math, and ours is
    3E-11 to 1E-10 off it.
- **Decision (owner, 2026-11-04):** "I want to do tier A through D now. I'm going to cut my losses after getting the
  important stuff. I think the 13th digit or earlier headline is good, let's do it for these functions, and after we
  get through a-d then I want to freeze the probes and move on with our lives." As rules:
  1. **Tiers.** A visibly-off cell is in Tier A if its type or error is wrong, or its first wrong significant digit
     is the 11th or earlier. It is Tier B at the 12th or 13th digit, Tier C at the 14th and Tier D at the 15th. The
     first wrong digit N is where the gap first exceeds half a unit in the N-th digit, the boundary rule of
     `within_fifteen_digits`. Work goes A, then B. Inside a tier, type and error cells come first.
  2. **The 13-digit bar for the incomplete-beta family.** The family is BETA.DIST, BETADIST, F.DIST, F.DIST.RT, FDIST,
     BINOM.DIST, BINOMDIST, NEGBINOM.DIST and NEGBINOMDIST.
     - Counted as visibly off: a number attributed to the family that is first wrong at the 13th digit or earlier.
     - Phase 1b residue: a family number first wrong only at the 14th or 15th digit. Every run reports the residue,
       and it is never silently dropped.
     - The rule applies to the headline, the Phase 1 rate and tiers, the campaign counts, the probe rows, Gate B and
       the milestone exits.
     - Unchanged: Gate A (decision 0188 R2's envelope and the R4 fresh-sweep check), which keeps 15 digits on
       Excel's side for every function. Also unchanged: type, error and text verdicts, and every other function. The bit-level baselines and
       `make eval-check` are unchanged too, but item 6 says when a family regression is allowed. The visible-parity
       lock counts visibly-off cells at this bar, so a family cell that moves from the 15th to the 14th digit no
       longer raises it.
     - SPEC-0030 defines the rule (T-4055), and the same PR implements it (T-4056). The owner asked for both now.
  3. **R3 ruling: follow the math (decision 0188 R3).** It covers T.INV.2T, TINV, T.INV, CONFIDENCE.T and BETA.INV at
     a tiny p, and CHISQ.INV.RT and CHIINV at df 1. It applies on each evidenced point where Excel is more than G off
     the 50-digit math. The marks go in eval/accuracy/math-over-excel.tsv with a `MATH-OVER-EXCEL` entry, and the
     engine must return the math within ε there. Points within G keep Excel as the target. df 1 near p = 1
     (`T.INV.2T(0.999999999999999,1)`, T-2858) is outside the ruling. Owners: T-3386 and T-3337.
  4. **Tier A and B owners, and the cut.** The tasks below own every Tier A cell (each has a "Decision 0267 cells"
     list) and the Tier B cells of their clusters:
     - P0, type and error first:
       - IRR: T-3411, T-3548, T-3408, T-3407, T-3347, T-3117;
       - RATE: T-3424;
       - GAMMA.DIST: T-3855;
       - GAMMA.INV: T-3409, T-3396;
       - BETA.INV: T-3351;
       - NEGBINOM.DIST: T-3576;
       - Excel's F quirks at a huge df2: T-4044 (`#NUM!` at df1 79) and T-4045 (exact zeros);
       - our F accuracy at a huge df2: T-3754.
     - P1: T-3386 and T-3337 (R3), T-2555, T-3061, T-3764, T-3961, T-4026, and T-4053 (new: Excel's df1 = 2 values at
       a huge df2).
     - P2 (Tier B mainly): T-3388, T-3959.

     Decision 0056's time-box applies to each task. When a task blocks as time-boxed, the cells it still lists are
     accepted, and no follow-up is filed for them. These Tier B clusters have no owner and are accepted now:
     - t3249-beta-large-shape fit and hold: 1,031 density cells at ln f near −600, where Excel's own density is up
       to 235 ulp off;
     - t2513-r3-binom-pmf: 173 cells; its owner T-3774 is already blocked as time-boxed;
     - the F density sweeps of t2586: 80 cells, Phase 1b under decision 0188 R5;
     - t3953-f-tail-bfrac-tiny: 24 cells, where Excel's error is unbiased noise (statistics.md);
     - about 45 scattered research-sweep cells.

     Tier C gets no new work. A Tier C cell is worked only inside a task that owns Tier A or B cells.

     Tier D is cut. These family tasks own only 14th or 15th-digit work, or probes and ports that serve only it, and
     get `defer: phase-1b`: T-2642, T-2658, T-2893, T-2924, T-3015, T-3183, T-3198, T-3322, T-3327, T-3563, T-3665,
     T-3712, T-3719, T-3750, T-3766, T-3770, T-3871, T-3886, T-3913, T-3933, T-3947, T-3952, T-3954, T-3968, T-4015,
     T-4016 and T-4018.
  5. **Probe freeze.**
     - From now on, a new probes-numeric case is allowed only for a task with a "Decision 0267" or "Decision 0267
       cells" section, and only for the cells it owns.
     - Once every such task is done, blocked or dropped, probes-numeric is frozen: no new cases and no new cells, only
       oracle refills of existing cells, unless a decision names an exception. T-4057 adds the check and stays
       blocked until then.
  6. **Net-improvement allowance for family regressions.** Owner, 2026-11-04: "It's ok if some of these regress to
     14 if the net is improvement. You can do this without my permission." A PR may record its regressions with
     `target/release/rustsheet-eval update-baseline --allow-regressions`, citing this item and with no further
     decision record, when all of these hold:
     - (a) every regressed cell is a number of a family formula (`closeness::THIRTEEN_DIGIT_FAMILY`) that stays
       within half a unit of Excel's 13th significant digit, so it lands in `residue13` or better;
     - (b) no cell of any other function regresses, and no type, error or text verdict gets worse;
     - (c) the net is an improvement. Over the suites the PR's `make eval` runs, the family's cells within 15 digits
       (`pass + ulp1 + ulp4 + digits15` of the family functions in report.json `functions[].closeness`) rise
       against main, and the family's visibly-off numbers (`visible_numeric`) do not rise;
     - (d) the PR body gives the before and after counts of (c) and lists the regressed cells, and the Excel-fidelity
       reviewer checks them.

     Anything else still needs its own decision record (CLAUDE.md prime directive 2).
- **Consequences:**
  - `make next-task` picks the P0 Tier A tasks first. The noise work leaves the ready queue.
  - Five of the deferred tasks own gen-m9 cells: T-3952, T-3954, T-4015, T-4016 and T-4018. Their FDIST and F.DIST
    cells are 15th-digit Excel noise, and are now residue for Gate B too.
  - The headline drops by the family's 14th and 15th-digit count. On probes-numeric at a262d6b0, attributed
    visibly-off numbers go from 89,170 to 4,238, and 84,932 become `residue13`. The residue gets its own bucket in
    report.json and report.md and its own line on the dashboard, so nothing is hidden. The steward's
    `--lower-visible-lock` then lowers the lock lines it frees.
  - Reopening a deferred task needs a cell it owns that is first wrong at the 13th digit or earlier. A new family
    algorithm study needs a new decision.
  - The analysis scripts and per-cell tables are not committed. The numbers above can be reproduced with
    `rustsheet-eval run --suite probes-numeric`, `report.json`'s `attributed_off` and `unattributed_off`, and the
    boundary rule of item 1.
