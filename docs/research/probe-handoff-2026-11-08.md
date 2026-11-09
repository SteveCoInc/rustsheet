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
