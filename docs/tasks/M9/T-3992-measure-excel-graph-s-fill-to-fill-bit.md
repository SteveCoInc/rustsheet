---
id: T-3992
title: "Measure excel-graph's fill-to-fill bit stability on the kernels the open R4 fits target (incomplete beta, lnGamma, gamma and t quantiles, RATE/IRR, CUMPRINC), so fits below the 4-ulp floor know which cells are targets"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
For each numeric kernel that open fit tasks are fitting at 1 to 3 ulp, docs/research/ records how often Excel for the
web returns different bits for the same input bytes across independent fills. Each kernel's cells are split into
fill-stable and fill-unstable. That gives T-3610's decision and decision 0188 R4.4 a measured web-side floor per
kernel, in place of the desktop reproducibility check that no session can run today (excel-desktop is unavailable,
T-1553).

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main eda15c023, excel-graph). Every failing cell of the report now has
  an open owner (`scripts/exit_owners.py --all-off` and a per-cell `owned` pass over every failing cell, deferred tasks
  included: only nine CONVERT ulp cells had none, now added to T-2047). Most of the remaining numeric work is last-digit
  fitting, and many of those tasks stop at the same guard.
- Decision 0188 R4.4 forbids fitting below the oracle's reproducibility floor (web against desktop up to 4 ulp,
  docs/research/oracle-calibration.md) "without a desktop-backed reproducibility check or an owner ruling". 22 open
  tasks cite R4.4. For example:
  - T-3937 (GAMMALN/GAMMA, 719 cells 1-3 ulp) says a coefficient set chosen on its cells "needs a desktop-backed
    reproducibility check or an owner ruling (R4.4)";
  - T-3813 (bfrac) says to "refill the probes (decision 0112) and check" before fitting below 4 ulp;
  - T-3963 (RATE) records that a second fill on 2026-11-03 reproduced every bit, "but one refill doesn't clear the
    floor".
- What is known about fill-to-fill variation, all of it scattered:
  - LN: T-2143 filled one 380-point workbook three times within 35 s (probes-numeric/t2143-ln-near-ties-fill1..3). 23
    points differ, always between adjacent doubles (decision 0112, numbers.md "LN is FLDLN2 then FYL2X").
  - ERFC/EXP: the T-1916 re-fill changed 9 of 20 ERFC near-tie cells (T-1940's Context).
  - AMORDEGRC: gen-m9 d912671baa1364aa Calc!A76 is 1 in one fill and 0 in two later fills of the same bytes (T-3610).
  - RATE: one refill of T-3963's same-state probe reproduced every bit (T-3963).
  No fit-target kernel other than LN has a measured rate.
- The question is evidence, not a fit. A cell whose bits change between fills cannot be a fitting target (decision
  0112's principle). A kernel with no unstable cell over three fills gives a measured case for fitting below 4 ulp on
  web evidence. That case goes to T-3610's decision, or to an owner ruling under R4.4.
- Candidate cases, one or two per kernel family. Counts are checked cells in this run; keep the total inside the
  oracle's budget. Each case is named with one example cell, and its owners keep their cells: this task measures,
  it owns nothing.
  - incomplete beta: probes-numeric/t2464-r2-bpser-small-gl (5,843, or a sample; e.g. Calc!A1000, owners T-2122,
    T-3042, T-3331) and t3953-f-tail-bfrac-tiny (4,800; Calc!A1, owners T-3954, T-3968);
  - lnGamma/GAMMA: probes-numeric/t3570-r5-bpser-lg-aux (2,520; Calc!A100, owner T-3937);
  - gamma quantile: probes-numeric/t3311-gamma-inv-lower-tail-cdf (899; S!A1, owners T-3925, T-3403, T-3412, T-3659);
  - Student t quantile: probes-numeric/t3349-t-inv-2t-sweep (300; Calc!A14, owners T-3061, T-3386, T-3414, T-3415);
  - IRR: probes-numeric/t3478-irr-r4-wide (300; Calc!A1, owners T-3411, T-3548);
  - RATE: probes/t3850-rate-same-state (240; Calc!A1, owner T-3963);
  - CUMPRINC/CUMIPMT: probes/fin-cumipmt-tiny-rate-structure (226; Sheet1!A10, owners T-3776, T-3938);
  - control: probes-numeric/t2143-ln-near-ties-fill1 (380; Calc!B10, owner T-2651), whose instability is known.
- Method: `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite <suite> --case <id> --force`. That
  overwrites the committed sidecar, so copy each fill's sidecar to docs/research/T-3992/ and restore the committed one
  with `git checkout`. Space at least two fills apart in time (separate sessions, ideally separate days), because
  server or CPU differences are the leading hypothesis for decision 0112. Compare with
  `rustsheet-eval snapshot`/`compare`; tools never parse xlsx values themselves.
- excel-graph only. Nothing here needs excel-desktop.

## Acceptance criteria
- [ ] At least three fills of each chosen case (the committed fill counts as one) are compared cell by cell. For each
      kernel family, docs/research/T-3992/README.md gives: cells compared, cells whose bits differ in any fill, the
      largest difference in ulp, and whether the differing cells cluster (near-ties, one branch, one argument range).
- [ ] The control (T-2143's probe) reproduces decision 0112's instability, or the README explains why it does not.
- [ ] For every open task that cites R4.4 or "reproducibility floor" (rg them; list each id), the README states
      whether its kernel was measured and, if so, how many of its owned cells are fill-unstable. Each such task gets a
      one-line note under its Context that links the README. Adding that note is the only edit to other tasks.
- [ ] A docs/excel-semantics/ entry (numbers.md, beside "LN is FLDLN2 then FYL2X", or the kernel's own file) records each
      kernel found fill-unstable, citing cells. A kernel found stable is recorded as "stable over N fills" with the
      dates.
- [ ] No committed `.oracle.json` sidecar changes: `git diff --stat origin/main -- eval/` is empty, and `make eval-baseline`
      records nothing.
- [ ] The Outcome recommends input to T-3610 (for example, "a cell is a target when N fills agree; kernels X and Y
      measured stable"). It does not make that decision itself.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The scoring decision itself (T-3610) and any change to R4.4 (a decision record that needs an owner ruling).
- Any engine or fit change.
- Web against desktop comparisons (T-1075, M14).
