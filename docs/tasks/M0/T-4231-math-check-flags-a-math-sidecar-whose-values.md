---
id: T-4231
title: "math --check flags a math sidecar whose values main's code would no longer write"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math --check` (run by `make check`) fails when a committed math sidecar holds values or skips
that the current generator code would no longer write, not only when a candidate cell is missing.

## Context
- Follow-up of T-3680 (PR #3068), raised by its excel-fidelity review. `check_case`
  (tools/src/rustsheet_tools/accuracy/eval_math.py) checks coverage only: every candidate cell is held or listed as
  skipped, and every entry's `f` matches the cell. It never notices a definition or reader change that alters values.
- Found stale on main by #3068's regeneration, both from code already merged:
  - probes-numeric/t2586-verify-r5-f-edges-hold.math.json: B61, B62, B65, B66, B69 and B70 held tiny values
    (e.g. 7.39e-465) that are 0 since T-3675's subnormal flush in `fitlab/sweep.arithmetic` (#2916); B53 and B54 are
    "no single value".
  - probes-numeric/t1543-inverse-cdf-tiny-p.math.json: A91 and A92 (`BETA.INV(1E-300,0.5,100)`,
    `BETA.INV(1E-200,0.5,100)`) are "no single value" since T-3578 (#2873).
- `--check` must stay cheap (no mpmath; it runs in CI). Candidate designs to weigh: a per-definition version or
  source hash stored in each sidecar entry (or per sidecar) and compared against the current code; storing the parsed
  arguments so a reader change (like the subnormal flush) is caught; a sampled recompute in a nightly job instead.
- A full regeneration of probes takes hours on 4 cores (T-3680's Outcome), so the design should let a change name
  the sidecars it invalidates rather than force a full rerun.

## Acceptance criteria
- [x] A change to a Gate A definition or to the argument reader that alters a committed math value makes
      `math --check` fail on the affected sidecars, with a test that edits a definition and sees the failure.
- [x] `math --check` stays mpmath-free and its CI time does not grow by more than 10%.
- [x] The committed sidecars pass the new check (regenerate only what it flags).
- [x] `make check` green; reviewers approve.

## Out of scope
- Speeding up the generator.

## Outcome
- New `tools/src/rustsheet_tools/accuracy/fingerprint.py`: every eval math sidecar's `source` records `definitions`
  (function → 16-hex hash of the code computing its definition: the modules of its `value`, `domain` and `tail`
  and every `rustsheet_tools` module they import, transitively, each hashed as its syntax tree without docstrings,
  so comments and docstrings do not count) and `arguments_sha256` (every computable candidate's sheet, cell,
  formula and arguments as the definitions receive them, by exact `repr`; a composite by its tree).
  `fingerprint.MACHINERY` (model, literal, sampler, tables, the fit lab's sweep reader and scoring modules,
  snapshot) is not hashed: nearly every definition imports it, the reader is covered by the argument hash, and an
  edit there that alters values bumps `tables.VERSION`, which the check now compares too.
- `math --check` (`eval_math.check_case` → `stale_inputs`) fails on: another generator version; a definition whose
  code changed, naming it (`regenerate with --function F`, T-4483, so a change names the sidecars it invalidates);
  arguments the reader now gives otherwise (reported only when no per-cell staleness already explains it); a
  sidecar without fingerprints. Still mpmath-free.
- Tests (tools/tests/test_accuracy_eval_math.py): editing T.DIST's module source fails the check naming T.DIST; a
  reader that reads 1.5 as 1.25 fails it on the arguments; a sidecar without fingerprints or of another version
  fails; the source hash ignores comments and docstrings only; T.DIST's digest covers defs/_common.py and the fit
  lab's beta kernel and no machinery module.
- Committed sidecars: all 984 (probes 577, probes-numeric 406, gen-m9 1) stamped with the fingerprints of the
  current code without recomputing. Before stamping, each file's bytes were rebuilt from its own entries and
  matched the committed bytes exactly, except probes/t2728-irr-guess-ref-mark.math.json, whose case file T-2766
  changed without a regeneration (`input_sha256` only); that one case was regenerated (its XIRR value is
  unchanged). A seeded spot recompute with mpmath of 80 random small cases (677 entries, values and skips) matched
  the committed sidecars exactly. The two stale sidecars named in Context were already regenerated on main.
- CI time of `math --check --suite probes --suite probes-numeric` on 4 cores, three runs each: main 15.0 to 15.1 s,
  this branch 16.2 to 16.4 s (+7%, within the 10% bound). The fingerprint costs about 1.3 s of serial work over
  both suites, against about 44 s for parsing the candidates.
- A PR that adds or regenerates a sidecar with older tools code, or edits a definition module, after this merges
  gets "no input fingerprints" or "computed by other code" from `make check` and regenerates the named sidecars.
- SPEC-0029 ("Math references for eval cells") describes the new `source` fields.
- Eval delta: none (sidecar `source` fields only; values unchanged).
