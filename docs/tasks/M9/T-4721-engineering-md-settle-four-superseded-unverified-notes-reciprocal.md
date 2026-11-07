---
id: T-4721
title: "engineering.md: settle four superseded UNVERIFIED notes (reciprocal IM* kernels past 2^27 on a real argument, the other IM* overflows, T-2420/T-2421's probe-pending status, IMLOG2's T-2044 candidates) and probe what no cell decides"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/engineering.md carries no UNVERIFIED note that a later VERIFIED entry or a committed probe
already answers, and the parts no cell answers yet are probed and matched. This is the sibling of T-4604, which
settles four other notes of the same file; the two tasks share no note.

## Context
- Gap triage 2026-11-06 (opus worker run, `make eval` on origin/main 4def66f7 with calipers, corpus-bench and poi
  fetched). `scripts/exit_owners.py --all-off` reports 0 visibly-off cells with no open owner, so UNVERIFIED notes
  with no open task are the gap left. Every IM* function is M9 in eval/catalog/functions.yaml (prior 0.05).
- The four notes, none cited by an open task (T-4604 covers IMPRODUCT's blank in a range, IMCOSH's tiny imaginary
  part, IMABS near DBL_MAX and OCT2BIN of a blank; T-2631 covers the large-y note near line 1717; T-2797 the
  IMSINH/IMCOSH band cuts):
  1. The IMCSC/IMCOT tiny-imaginary entry, "UNVERIFIED: for |x| ≥ 2^27 (SIN's own `#NUM!` cut) the reciprocal kernels
     take libm's sine and cosine". The later entry "IM* trig and hyperbolic functions past SIN's argument limit
     (T-3642)" (VERIFIED, probes/im-trig-magnitude-grid and probes/im-trig-argument-cut) makes IMTAN, IMSEC, IMCSC
     and IMCOT `#NUM!` from |x| = 2^27 with a nonzero other part, and says a zero other part keeps the real rules
     (`IMCOT("134217728")` is 0). Check which of the note's claims those cells already decide; what is left is the
     real-argument IMSEC, IMCSC and IMTAN at and past 2^27.
  2. "IM* results that overflow (T-1827)": "The other IM* functions are UNVERIFIED." The same file has since
     verified IMSIN, IMCOS, IMSINH, IMCOSH (T-2003), IMSUB (T-2003), IMSQRT (T-1996, T-4461), the input-part cut at
     1E308 for every IM* function, and the trig grid. Left unprobed: IMLN, IMLOG10 and IMLOG2 of a modulus past
     DBL_MAX (`"1E307+1E307i"` and the like, where hypot overflows but the log does not), IMCONJUGATE, IMREAL and
     IMAGINARY of the largest parts, and `IMDIV("1","1E-308")` (the same file: "the cause (subnormal divisor?) is
     unprobed, ours matches").
  3. The edge-pool smoke check entry (T-2249): "Status: UNVERIFIED (probe pending: T-2420 and T-2421 commit probe
     cases for the mismatches and the BESSELI cap cells ...)". T-2420 and T-2421 are done; cite their committed
     probe cases and set the status the cells support.
  4. The IM* last-digit sweep entry (around line 1321): "Candidates the triage checked (UNVERIFIED until T-2044's sweep
     confirms them): IMLOG2's real part is ln|z| / 0.693147180559945 ... IMCSC is conj(s)/|s|² ...". T-2044 is done and
     the IMLOG2 entry near line 778 (T-2231, T-3643) now gives the implemented model (`ln` of the doubly rounded
     modulus times K = 1.4426950408889636). Replace the candidate list with a pointer to the entries that settled it,
     or mark each candidate the later work rejected.
- Code: crates/rustsheet-engine/src/functions/engineering/complex/ (`trig.rs`, `mod.rs`, `finite_complex_result`).

## Acceptance criteria
- [ ] engineering.md: notes 1, 3 and 4 are reworded to cite the committed cells or entries that settle them, or
      kept with the precise part still open; no claim is changed without a cited cell.
- [ ] A probe case `probes/t4721-im-overflow-residue.case.json` is committed with excel-graph oracle values and holds
      at least: `IMSEC`, `IMCSC` and `IMTAN` of `"134217727.99"`, `"134217728"` and `"1E+10"` (real arguments);
      `IMLN`, `IMLOG10` and `IMLOG2` of `"1E307+1E307i"`, `"9.9E307+9.9E307i"` and `"-9E307-9E307i"`; `IMCONJUGATE`,
      `IMREAL` and `IMAGINARY` of `"9.99999999999999E307-9.99999999999999E307i"`; and `IMDIV("1","1E-308")`,
      `IMDIV("1","2.3E-308")`, `IMDIV("1","1E-307")` and `IMDIV("1E-300","1E-308")` to place the IMDIV cut. Wrap
      error results in `ERROR.TYPE` only if excel-graph cannot decode them.
- [ ] Every probe cell passes, or each cell that does not is listed in a new functions-engineering task (`campaign:
      semantics` for a type or error mismatch, with a `## Visible-parity stage` section) filed with `make new-task`
      that names the case and its cells, and those known failures are recorded with `make eval-baseline`.
- [ ] engineering.md note 2 lists the newly observed functions and cites the probe cells.
- [ ] Any engine change has a unit test that pins a probed result.
- [ ] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- The four notes T-4604 settles, the large-y note (T-2631), the IMSINH/IMCOSH band cuts (T-2797) and any last-digit
  work (T-2538, T-4001, T-4012).

## Outcome
New probe probes/t4721-im-overflow-residue (excel-graph, 25 cells, all pass; no engine change, one generated case-count line in eval/baseline/probes.txt (`make eval-baseline`), no
math sidecar since it has no Gate A one-call cell). Findings: IMSEC/IMCSC/IMTAN of a real argument are numbers at
134217727.99 and `#NUM!` at 134217728 and 1E+10; IMLN/IMLOG10/IMLOG2 of an argument whose x²+y² overflows (the modulus is finite) are `#NUM!`;
IMCONJUGATE/IMREAL/IMAGINARY return the largest parts unchanged; the IMDIV cut lies between 1E-308 (`#NUM!`) and
2.3E-308 (a number). engineering.md notes 1-4 now cite these cells or the entries that settled them. Eval delta: one
new passing case, 0 regressions.

## Blocked because
Resumed after the review-round limit (step 5c).

Three fidelity rounds ran (heads bcd6c01a7, 2a81cae4e, 000f53490). Design and security approved bcd6c01a7. The round-3
findings (T-2044 Status line still calling the candidates UNVERIFIED; T-1827 entry Evidence/Tasks lines; IMDIV cause
cross-reference) are fixed in the commit that adds this section, with no reviewer run on it. Resume per work-loop step
5c: `make check-fast`, then the full panel on the current diff.
