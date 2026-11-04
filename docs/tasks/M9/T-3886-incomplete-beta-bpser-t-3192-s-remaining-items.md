---
id: T-3886
title: "Incomplete beta bpser: T-3192's remaining items (b0 >= 8 L diff, T-2505's 987 moved cells, 6 shape-of-1 cells) and a = b = 1/2 at x = 0.5"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3192]
campaign: accuracy
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The bratio port's bpser items T-3192 did not reach are ported where Excel's form is found, or recorded as negative
results with their scores; and BETA.DIST at a = b = 1/2, x = 0.5 is explained.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (incomplete beta, bpser) measured on t2464-r2-bpser-small-gl, 0 of 5843 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
T-3192 took its fourth item (t1745-beta-inv-roundtrip G1) and found BETA.DIST at a = b = 1/2 is an arctangent closed
form, not bpser (statistics.md "BETA.DIST at a = b = 1/2 is an arctangent closed form, reflected above x = 0.9";
docs/research/t3192). Its other three items carry over unchanged (read T-3192's Context for the detail):
- the bpser criterion's shortfall at b0 >= 8 on probes-numeric/t2464-r2-bpser-small-shapes (engine 2,859 of 2,868
  within 15 digits against the model's 2,861): diff the model's L and the engine's L at the cells
  `docs/research/t2977/score_branches.py` lists (needs the T-2464 harness back from git history, per its docstring);
- the 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the research
  model with the bindings venv (decision 0114): which are the port's and which the model's too;
- the 6 cells T-3170's shape-of-1 w moved off: probes/ftest-decomposition B12, t2220-f-dist-sweep-cdf A191, A195,
  t2220-f-dist-sweep-d2-1 A16, t2220-f-dist-sweep-rt A195 and t1745-beta-inv-roundtrip G145.
New from T-3192: `BETA.DIST(0.5,0.5,0.5,TRUE)` is 0.5000000000000001 in Excel (m9-beta-dist-a0p5 A9,
m9-legacy-betadist A19, A20, t1745-beta-inv-roundtrip I4, I180) and 0.5 in the arctangent form; the form's other
1-2 ulp misses (40 cells of probes-numeric/t3192-beta-half-half, 54 of t3192-beta-half-half-2) have no model.
Whether T.DIST at df 1 takes the same form (it fixed t0969-t-beta-link's df-1 BETA.DIST cells) is T-2930's.
T-3192 ported the direct form only; Excel's reflection above x near 0.9 (the switch: 281 of 321 and 287 of 341
bit-exact against the direct form's 270 and 228) is a 1-ulp cut below decision 0112's floor, so porting it needs a
desktop-backed reproducibility check or an owner ruling (decision 0188 R4.4), the R4.1 split and an `r4-T-3886` Gate A
batch: `rustsheet-accuracy generate BETA.DIST` draws the shapes and cannot pin a = b = 1/2, so add a shape pin to
tools/src/rustsheet_tools/accuracy (defs/beta.py, cli.py) first.
Tools: `docs/research/t3007/cell_diff.py` for full-eval A/B diffs; `docs/research/t3192/score_half.py`.

## Acceptance criteria
- [ ] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with
      its scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.

## Outcome
The two items that could be scored with today's tooling are recorded as negative results. The other two were split
to T-4070. Nothing is ported, and engine code is unchanged.
- **a = b = 1/2 (x = 0.5 and the 1 to 2 ulp misses):** no closed form explains them
  (docs/research/t3886/RESULTS.md section 1, `score_half_x87.py`).
  - Over the three sweeps (962 cells), 10 double forms and 192 variants with 64-bit (x87) intermediates were scored.
    None beats the ported direct form's 737 bit-exact.
  - The 22 variants that give Excel's 0.5000000000000001 at x = 0.5 reach at most 666.
  - Excel's worksheet ATAN form equals the double direct form on all 321 cells of sweep 1, so the misses are not a
    libm difference.
  - The unported 0.9 switch (855 of 962) still needs R4.4 evidence or an owner ruling. It is filed as deferred T-4071
    (phase-1b), with the shape-pin prerequisite.
- **The 6 shape-of-1 cells:** no single form of w fits them (RESULTS.md section 2, `ln1m_tie.py`).
  - With l correctly rounded, Excel's expm1 gives Excel's bits at A191 and G145. A191 is a near-tie of the log1p
    series (0.053 ulp from the tie); G145 is not a tie and stays unexplained.
  - The A195 pair and A16 take libm's expm1, 1 - EXP or a reassociated Kahan form, which conflict with the 71 cells
    T-3170's form fixed.
  - Full-eval A/B of the reassociated Kahan form (`kahan_reassociated.patch`): 78 cells newly bit-exact, 47 lost,
    5 of them gated probes cells, so it was not ported.
  - Its wins include all 47 of T-3764's t3568 column-B cells. That is recorded in T-3764's Context as a lead.
- **Split to T-4070** (opus, functions-stat): the b0 >= 8 L diff and T-2505's 987 moved cells. Each first needs the
  T-2464 harness ported back from git history, or the bindings venv built, before anything can be scored.
- statistics.md: the a = b = 1/2 entry and "Incomplete beta at a shape of 1 outside the closed forms" carry the
  negative results.
- Eval delta: none (docs and research scripts only).
