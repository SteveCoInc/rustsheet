# Probe hand-off: fill the 15 `val2-*` probes with the Excel oracle (2026-11-08)

**For:** whoever runs the Excel oracle (`.github/workflows/oracle.yml`, environment `oracle`).
**Why:** these probes decide the open questions behind the six functions still below the Gate B rate (NORM.S.INV,
NORM.INV, LOGINV, F.INV, HYPGEOMDIST, FVSCHEDULE) and IRR's error-only classifier. The questions, hypotheses and
predictions are in [probe-requests-2026-11-08.md](probe-requests-2026-11-08.md).

## What this change adds
The 15 drafts from `probe-cases-2026-11-08/` as real cases of the `probes` suite, so the oracle fills them like any
other pending case: `eval/suites/probes/cases/val2-*.case.json`. Each now has a `check` list (every formula cell) and
no `*.oracle.json` sidecar, so `rustsheet-oracle fill` picks them up. `rustsheet-eval cases probes` lists all 15
(3,224 cases in the suite). Inputs are unchanged from the drafts.

## How to run
1. **Excel for the web is enough for 13 of 15** (and is the corpus's oracle of record):
   Actions → *Excel oracle* → `suite = probes`, `backend = excel-graph`, `limit = 0`.
   `fill` only touches cases without a sidecar, so this fills the 15 (plus the suite's two existing pending cases)
   and opens a PR with the `*.oracle.json` sidecars.
2. **Desktop only** (`backend = excel-desktop`, the self-hosted Windows runner), if one is available:
   FVS-01's live and seeded steps and FVS-02's isolated variants (see their sections). Without a desktop runner, the
   web run still answers FVS-01's web-carrier question and every other probe.
3. **Run CAL-01 first** in whichever build is used: it checks that the build reproduces the corpus's cached web values
   and that low-bit residuals survive capture. If it fails, stop and report its sidecar: the other results would not be
   trustworthy.

## What happens next
Once the sidecars are on main, the research continues per function family (one session each: inverse normal,
F.INV, HYPGEOMDIST, FVSCHEDULE, IRR), scoring each hypothesis against the filled values. No engine change lands until a
probe confirms its rule; no fitted special cases.

## Results (2026-11-09, excel-graph)
All 15 `val2-*` cases filled in one Excel for the web run (`rustsheet-oracle fill -b excel-graph --suite probes`,
CAL-01 first). The suite's two older pending cases (`t0481-curly-quote-names-c`, `typed-if-one-argument`) were
refused again by Graph (`FileCorruptTryRepair`) and stay pending; they are not part of this batch.

**CAL-01.** Every anchor H2:H30, H32, H33 and H37 equals its cached web value bit for bit, so this build reproduces the
corpus. H1 (desktop equals web) is untested, as no desktop runner was used, and so is repeatability across a cold
reopen. The canary does **not** pass as written: column D (`(B-1)*2^52`) gives exactly -3..3, so low bits
survive capture, but columns C and E (`SUM(B,-1)*2^52`, `SUM(1,-B)*2^52`) are 0 in every row. That is the known
SUM near-zero snap (docs/excel-semantics/numbers.md, "a SUM call snaps wherever it sits"), not a capture loss. The
consequence for the later sheets is the one CAL-01 names: in-sheet `SUM(x,-y)/x*2^52` residual columns (HYP-01, HYP-02,
HYP-03) snap to 0 whenever x and y are within the snap window, so read those probes from the raw value cells, whose
bits are captured, not from the residual columns. The NSI sheets build their inputs from integer limbs and do not
depend on SUM.

**Two more fingerprints are unreadable on the web, for the same kind of reason.** FVS-02 column G (`B*2^F` with
F = -1023) is 0 for every finite B, because `^` flushes a subnormal result and `(2^-1023)=0` is TRUE in Excel (numbers.md,
"EXP is an x87 2^(a*log2 e) evaluation, not correctly rounded (T-1778)": "POWER flushes a subnormal result"), so rows with F = -1023 carry no information there. FVS-03's MOD fingerprint columns (60 cells) are
all `#NUM!` (C, E and G in rows 2 to 9; H, I and J in rows 12 to 23). Read FVS-03 from its raw values instead:
B, D and F in rows 2 to 9, and C, D and E in rows 12 to 23, where Excel's FVSCHEDULE, `*` product and PRODUCT are
equal to the bit.

**FVS-02 was filled as one sheet** although its protocol allows that only once FVS-01 has shown no carried state,
and FVS-01's carried-state steps (desktop) were not run. Its results are therefore unconfirmed as per-call behaviour
until the per-row isolated runs are made (T-4250).

**New Excel behaviour recorded with this fill:** FVSCHEDULE's overflow is a live 0 in some calls and `#NUM!` in
others, and a subnormal running product is 0 (financial.md, "FVSCHEDULE whose running product overflows or
underflows"); `*` near DBL_MAX does not overflow where the binary64 product does (numbers.md, "A product at the
edge of the double range"); FORMULATEXT keeps an exponent literal written `1E+300` (syntax.md).
HYPGEOMDIST and HYPGEOM.DIST are 0 in all of hyp-02 rows 37 to 50, exactly where Excel's own BINOM.DIST factor at
p = MAX(x, n-x)/N underflows (statistics.md, "HYPGEOM.DIST of an empty population is 1, and a deep-tail pmf can be 0").

**Tier note (no gate changes state):** the fill adds IFERROR, ISNUMBER, ABS and ISERROR to the foundational tier's
below-bar list (54 wrong type, 26 outside) and IRR, F.INV, F.INV.RT, BETA.DIST, FVSCHEDULE and FINV to the expert
tier's; both tiers were already unmet on main. The foundational misses are observers of FVSCHEDULE and HYPGEOMDIST
cells, so T-5108 and T-5109 own foundational-tier cells too.

**Owners:** scoring per family is T-5107 (F.INV), T-5108 (FVSCHEDULE), T-5109 (HYPGEOMDIST), T-5110 (IRR) and T-5112
(inverse normal); T-5111 is the FORMULATEXT spelling.

**Not answered here (desktop only):** FVS-01's live and seeded steps and FVS-02's isolated variants.
