# Addendum 3 to the 2026-11-07 Excel-parity findings: IRR (and NPV through IRR)

Status: research hand-off, **docs only**. In a prototype combining every fix in the earlier three documents with the change
below, the full eval reached **Gate B 500/506** with all gates clean:
- IRR 298 → 302/302 and NPV 227 → 228/228 (NPV's last cell, ironcalc IRR.xlsx General!M17, only consumes IRR L17);
- Gate A exit 0, after removing 68 known-failure exemptions that the gate itself reports as stale (accuracy/IRR points now
  inside the envelope), which takes the known failures from 244 to 176;
- `check` showing only the three pre-existing probe cases, with probe known failures down to 4807;
- engine (5763) and xlsx (583) tests, workspace clippy `-D warnings` and fmt all clean.

This is independent of, and consistent with, the T-4835 lineage table and the T-4836 hypothesis bank ("RATE's driver plus
the Newton fallback leads"), with one difference: the evidence says the last fallback is *not* Newton. As before,
re-implement through the work loop and re-measure on current main.

## H. IRR: one uniform secant in d = 1 − v, then two retries

**Root cause.** The current IRR picks its iteration budget from a coverage rectangle (`IRR_LONG_RUN_RANGE`: flows 6..54,
largest flow 5e-5..2e8, a guess range) and a `guess == 0.1` classifier: 200 updates inside, 50 outside. It then falls back
to a Newton polish. Those are proxies (decisions 0316/0320). Two consequences show they are not Excel's mechanism:
- appending a single zero flow, which does not change the polynomial, flips a root into the NUM error;
- t3478-irr-r4-wide Calc!A206 converges after all flows are divided by 4.

**Rule** (one secant kernel, `irr_secant`, used by `compute_irr`):
1. **200 updates per attempt, for every input.** No dependence on flow count, magnitude or guess. The initial NPV
   evaluations and flat-slope resampling do not count as updates.
2. **Flat slope.** When the two residuals are equal, replace the *older* endpoint with `d0 + (d1 <= d0 ? −0.001 : +0.001)`,
   re-evaluate its NPV, and take the usual secant step with `d1` unchanged. If the residuals are still equal, the attempt
   fails. Otherwise keep iterating, even at NPV's rounding floor, rather than returning a zero update.
3. **Stopping.** Both |step| and |residual| must be below 1e-7. At the 200th update the final check uses 1e-4 for both.
   Return the *new* iterate, not an older point or a Newton polish.
4. **Arithmetic.** The NPV running product and the secant expression use the shared twice-rounded (x87-to-double)
   operations, including `1 − d`.
5. **Retries.** After the supplied guess fails, retry from the default guess, i.e. d = 0.1/1.1. Then retry from the literal
   coordinate **d = −0.1**. This is *not* a rate guess of −0.1 and *not* Newton. Its signed initial offset explains the
   fallback's negation-sensitive digits.
6. **Domain.** Intermediate discounts may be negative. The final discount must be positive and satisfy the existing
   far-root bound (findings item 11, T-2928).

**Evidence.**
- *Long runs past the old box:* gen-m9/31474c20fc1dafd6 Calc!A173 and gen-m9/ba4ddb817a8e700e Calc!A373 reach Excel after
  updates 51 and 68, although a 1e15 coefficient puts them outside the old coverage box.
- *The d = −0.1 retry:*
  - ironcalc IRR.xlsx General!L17 returns −0.6737366751808405 at update 125. Newton gives …8422, and the continued primary
    secant gives …8431 at update 622.
  - `{42,1,1,−1}` returns −0.7467044766537371 at update 47; its negation returns …7407
    (probes-numeric/t2927-irr-fallback-sweep Calc!A80).
- *Update-200 check:* probes-numeric/t3347-irr-conventional-holdout P!A25 returns −6.722756777233219E-5 at update 200;
  update 199 is a different grid point. P!A118 distinguishes the twice-rounded `1 − d`.
- *Former box failures:* probes-numeric/t3478-irr-r4-wide Calc!A206 (−0.03181940929601346) and
  t3347-irr-r4-fresh Calc!A52 (−0.09859364881581734) are reached without any coverage dispatch.
- *Validation:* all 900 IRR accuracy-table cells and every case with an IRR call were inventoried. The cells were split into
  train and holdout by a fixed SHA-256 hash of the input family, which grouped duplicates, power-of-two scalings and
  trailing-zero variants together. The mechanism matches every correctly extracted numeric return on train and then on
  holdout. This validates against cached Excel evidence; it is not a fresh Excel sweep.

**Change.**
- Remove the iteration rectangle, the guess classifier, and the Newton/residual coverage box.
- Implement the kernel above.
- Keep decision 0316's *error-only* classifier (which error Excel returns when no root is found), but evaluate it only
  after all real searches have failed, so it can no longer pre-empt a valid root such as t3347 A52.
- Introduce no new fitted thresholds or restart seeds.
- Tests: the evidence cells above as bit-exact controls, plus a trailing-zero invariance test.
- Remove the stale IRR accuracy exemptions as Gate A instructs.

**Still open.** The error-only classifier: which of the DIV and NUM errors Excel returns when no attempt converges.
Current evidence:
- It is *path-dependent*, not a threshold. Along an annuity series it is non-monotonic: t3406 n=60 gives A81/A82 DIV, A83
  NUM, A84… DIV.
- Two cells with the same modelled trajectory get different results in Excel, which points to NPV's residual in the noise
  floor rather than to the iteration.

Excel experiments that would decide it:
- append a trailing zero flow (54 → 55 flows);
- rescale all flows by powers of two (the T-3117 sweep);
- negate the flows;
- record which error is returned along those paths.
