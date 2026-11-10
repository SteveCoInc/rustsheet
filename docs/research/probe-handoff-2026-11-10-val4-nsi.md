# Probe hand-off: the seven `val4-nsi-*` probes (2026-11-10)

**For:** the Excel oracle owner / the Excel-for-the-web oracle agent.
**Why:** measure the first piece of Excel's central inverse-normal kernel (NORM.S.INV, NORM.INV, LOGINV; owner
task T-5112). The val3 values showed what the kernel is in structure; its coefficients can only be measured.

## PROBE-REQUEST

Seven pending workbooks, **106,970 checked formula cells**, in `eval/suites/probes/cases/val4-nsi-0N.case.json`
(`rustsheet-case/v1`, the val3 layout: `sheets[].cells` and a qualified `check` list holding every formula cell).
No expected values, no oracle sidecars; the engine, evaluator, tests, baselines and gates are untouched. The
files are the output of `python3 docs/research/t5112/next_val4_nsi.py eval/suites/probes/cases` and regenerate
byte-identically.

Every data cell is `=GAMMA.INV(x,0.5,1)` with x written as an exact dyadic expression
`(hi*67108864+lo)/2^k` (integer limbs below 2^53, a power-of-two divisor: no 15-digit literal truncation, no
subtraction, no cell reference). Why GAMMA.INV: `SQRT(GAMMA.INV(x,0.5,1))` is the hidden binary64 inverse-erf
core w that NORM.S.INV multiplies by sqrt(2) (val3-nsi-01: 192/192 rows, val3-nsi-02: 9/9, val2-nsi-01: 12/12),
and squaring is injective on doubles, so one cell per row reads the kernel's output exactly.

| Case | Checked cells | Sheet and rows | What it pins | What we expect |
|---|---:|---|---|---|
| `val4-nsi-01` .. `04` | 4 x 16,390 | `R!A1:A16384`: 16 runs of 1,024 consecutive doubles each; global run i = 0..63 starts at x0 = fl(sqrt(0.08 + 0.0065137 i + 0.0031415)), so t = x*x covers 0.083 to 0.494 | degrees, normalisation and variable of the first piece (x*x < 0.5); the phases of every stored Horner step, i.e. the data the coefficient solve needs | a lattice: the core moves as c*i + Q*k_i along a run, with w/Q within 3 percent of the denominator of a (7,8) or (8,7) relative-minimax rational in t = x*x with Q(0) = 1 (F3); a spectral match with one candidate if Excel's doubles are near a minimax fit (F5); no reset at x = 0.5 (F4) |
| `val4-nsi-05` | 16,390 | `R!A1:A8192` the 8,192 doubles just below fl(sqrt(0.5)) = 0.7071067811865476; `R!A8193:A16384` the 8,192 doubles just below 0.5 | stage values where t is a power of two minus epsilon; the end of the first piece | spread about 12 ulp up to the last double below sqrt(0.5); smooth behaviour up to x = 0.5 (F4) |
| `val4-nsi-06` | 8,630 | `E!A1:A7168`: 28 ladders of 256 consecutive doubles from fl(sqrt(T(1-2^-e))), T = 1/2 then 1/4, e = 4, 6, .., 30. `E!B`: `=-NORM.S.INV((1-x)/2)` on the first 64 rows of the first 8 ladders (512 cells). `S!A1:A944`: 16 doubles at x = 0.71, 0.72, .., 0.99, 16 around 1-2^-k for k = 7..20, 64 around each of 0.85, sqrt(0.75), 0.75, 0.9375 | unwrapping of the stage phases; that NORM.S.INV still takes the same core; where the core changes piece above sqrt(0.5) | z = RN53(RN64(sqrt(E!A)*sqrt2)) on all 512 controls (F2); the piece map is an open question (F6) |
| `val4-nsi-07` | 16,390 | `R!A1:A16384`: 16 runs of 1,024 consecutive doubles, x0 = 0.7080 + 0.0088137 j + 0.0003141 (0.708 to 0.841) | the second piece's lattice | spread near 1.5 ulp at 0.708, growing with x (F6, qualitative) |

Each workbook also has the val3 `Precision` sheet (6 checked cells, included in the counts above).

Frozen predictions: [val4-nsi-predictions.json](val4-nsi-predictions.json) (rules F1 to F6 with what each
failure would mean, and the per-run denominator, spread and lattice of the (7,7), (7,8), (8,7) and (8,8)
candidates). This is research metadata, **not an oracle or math sidecar**; never copy it into expected values.

**Size choice.** The solve needs many runs at distinct t (each gives one equation per stored step) and runs long
enough to lock a step's phase; 64 x 1,024 is the smallest design that passed the synthetic control below. The
three 64-row ladders of val3 give three function values for sixteen unknown doubles and cannot decide the degree
(spectral overlap 0.2 to 0.7 for right and wrong candidates alike).

## How to run

From `tools/`, one workbook per case:

```bash
args=()
for n in 01 02 03 04 05 06 07; do args+=(--case "probes/val4-nsi-$n.case.json"); done
uv run rustsheet-oracle fill -b excel-graph --suite probes "${args[@]}"
```

In every workbook `Precision!B2:B4` must be **0.9999999999999998, 1, 1.0000000000000002** and `Precision!C2:C4`
**-1, 0, 1**. A capture failure means stop. Return the seven `val4-nsi-0N.oracle.json` sidecars with the full
binary64 values; do not transcribe displayed numbers. Excel for the web is sufficient.

Expect about a third of the GAMMA.INV cells to differ visibly from the engine inside the envelope (the engine
has the accurate value, Excel's core is up to a few tens of ulp off for 0.6 < x < 0.7071), as on val3-nsi-01;
the owner is T-5112.

## How it will be scored

`python3 docs/research/t5112/nsi_kernel_lab.py score-val4` (needs mpmath) prints per run: how many cores square
back (F1), the lattice (Q, c, coherence) of the first 64 rows, the error mean and spread, and the phase-free
spectral overlap with a stored-Horner simulation of each candidate of `docs/research/t5112/remez-t-0-0.5.json`.
Controls run before this request: on synthetic sidecars generated from the (8,7) candidate the scorer returns
1024/1024 cores, overlap 1.00 for (8,7) and 0.07 to 0.54 for the other eight candidates; a (7,8) truth whose
coefficients are 3.5E-7 away from the candidate's still scores 0.96 to 0.99 against 0.13 to 0.65 for the wrong
degrees. A truth far from every minimax fit (a Chebyshev-node interpolant, 0.10 away in its top coefficient)
is attributed to the nearest structure, which is why F5 only decides when one candidate is above 0.9.

## What happens next

A confirmed F3 and F5 fix the structure and the generator; the 64 runs then give the stage phases for the
coefficient solve (excel-kernel-playbook.md, sections 3.4 to 3.7), verified on held-out runs. The fix is the
first piece in x87-discipline Horner plus stored-z scaling in NORM.INV in the same change. If F3 fails, the
measured cores still give the rational's values on 64 runs for a direct fit of its degrees. No engine fix, newly
passing function or Gate B change is claimed by this request.
