# T-3850: narrowing T-3698's flat-secant restart against probes/t3906-rate-restart-oos

T-3906 found that T-3698's restart moves two points of probes/t3906-rate-restart-oos out of the Gate A envelope
(A152, A194). T-3850's binding criterion asks to narrow the rule so that none leaves, or to revert it. This note
records the search for a narrowing (2026-11-03). No existing case was rescored to pick a rule, and nothing here
changes the engine.

## Method

`narrow_grid.py` (its docstring gives the temporary switch and how to run it) scores 48 variants of the rule:
- restarts per call: 1 (today) or 2;
- when the restarted run is an error: return it (today) or continue with the hold;
- last restarting step K: 8, 12 or 16 (today);
- |f1| cut: 1e-4 (today) or 2e-5;
- |f1| floor: 1.3e-7 (today) or 1e-6.

Each variant is scored on every `=RATE(` cell of probes and probes-numeric where the rule and the hold differ (294
cells, 240 of them the t3906 case):
- **regressions**: cells outside the t3906 case that the merged rule gets bit-exact and the variant does not;
- **inside**: t3906 points inside the Gate A envelope (within half a unit of Excel's 15th digit, or 1e-9 relative
  of the case's math.tsv root; an error against a number is outside);
- **left_vs_hold**: t3906 points the hold has inside and the variant has outside.

`grid.tsv` has all 48 rows. Those with 0 regressions:

| restarts | on error | K | inside | bit-exact | left vs hold |
|---|---|---|---|---|---|
| 1 | return it (today) | 16 | 207 | 48 | A152, A194 |
| 1 | hold | 16 | 207 | 47 | A152 |
| 2 | return it | 16 | 207 | 51 | A194, A204 |
| 2 | hold | 16 | 207 | 49 | A204 |

Every variant with no point leaving (2 restarts with K = 8) regresses 9 to 16 existing bit-exact cells.

## Findings

1. **No narrowing in the probed shape meets the criterion.** Every variant either moves a t3906 point out or
   regresses existing cells (the T-3698 gains in t3579-rate-residual-nper1, t3698-rate-flat-restart, -fresh,
   t3348-rate-fv-nonzero and t3348-rate-gate-a-fresh).
2. **Falling back to the hold on an error is not free either.** It brings A194 back, but A26 then leaves: Excel is
   `#NUM!` there, the restart is `#NUM!` and the hold is a number. The regressions column does not count A26, since
   the t3906 case is excluded from it. Once the case is baselined, A26 is a regression.
   - A26's restarted run converges to r = -1 + 3e-12, which the 1 + r floor refuses (T-3585).
   - A194's restarted run wanders for 92 steps, then meets a second flat secant with |f| 1.65e-4. That is above the
     cap's 1e-4, so the run is `#NUM!`. Its last iterate, 0.01268668355857748, is within 1e-10 of Excel's
     0.012686683557386938. The hold, 0.012686683564913244, is 6e-10 away.
   - The two cells fail in different ways. Separating them would rest on two points, a fit with no evidence behind it.
3. **A152 leaves inside the root's own rounding noise.** For RATE(1, -71487002991.5263, 71487005769.4476,
   -10881.0480439314, 1) the residual's terms are about 7e10, and its slope at the root is pv + pmt, about 2.8e3.
   - The width of the root's rounding noise is eps·Σ|terms| / |f'(r)| / |r| = 7.7e-9 relative, almost 8 times
     ε = 1e-9.
   - The rule is 1.36e-9 from the math root and the hold 7.9e-10, so which of them lands inside is chance.
   - T-3906 already saw this: with roots from the decimal literals the leaving point is A144, not A152.
   - Across the case, 23 of the 234 numeric points have a noise width above 1e-9. Of the 23 points where the rule is
     more than 1e-9 off the math, 21 have a width of 3.3e-10 or more.
   - Excel's own value at A144 is 1.4e-8 off the math.
4. **A revert also moves points out.** Against the merged rule, the hold moves A15, A26, A64, A76 and A121 out of the
   envelope (T-3906's table). It also loses the 22 bit-exact cells T-3698 gained and drops 48 bit-exact t3906
   cells to 12. So neither option in the criterion keeps every point inside.

## What is open

Choosing between the two options depends on how decision 0188 R2/R4 applies here: "a fit that moves a point out of the
envelope fails this item", at points whose root noise width exceeds ε. Agents cannot rule on that (decision 0188
is the owner's), so the choice is listed under "Needs owner" in docs/status.md and tracked by its own task. Unless
the owner rules otherwise, the rule stays as merged.
