---
id: T-4603
title: "IMCSC, IMCOT and IMSEC of a tiny real, and IMCSCH, IMSECH and IMCSC of a tiny imaginary part: probe the reciprocal cells engineering.md leaves UNVERIFIED, and match"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells answer the tiny-argument reciprocal claims in engineering.md "IM* results that overflow (T-1827)". These
are IMCSC, IMCOT and IMSEC of a tiny real, and IMCSCH, IMSECH and IMCSC of a tiny pure imaginary part. The bullet then
states what Excel does, with the cells, and ours matches every cell.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage read the rules the semantics docs mark UNVERIFIED.
- engineering.md "IM* results that overflow (T-1827)", the bullet on IMSEC, IMCSC, IMSECH and IMCSCH, ends with two
  claims:
  - "a tiny real-only or imaginary-only IMCSCH such as `IMCSCH("1E-200")` also gives `#NUM!` here: UNVERIFIED";
  - "A denominator that underflows to 0 for IMCSC (`IMCSC("1E-200")`) and IMCOT (`IMCOT("1E-310")`) is also `#NUM!`
    here: UNVERIFIED".
  Both are out of date:
  - The real-only half is settled the other way. probes-numeric/t3609-imcsch-tiny-real-probe S!A9 has
    `IMCSCH("1E-200")` = "1E+200" (excel-graph, passing). engineering.md "Engineering functions in the gen-m9
    engineering sweep (T-2258)" already states it (the T-3609 sentence).
  - `IMCOT("1E-310")` is decided by the parse, not by a denominator: a subnormal part is `#NUM!` in every IM*
    function (`IMREAL("1E-310")`, probes/eng-inumber-reading, T-2541).
  - Ours no longer gives `#NUM!` for `IMCSC("1E-200")`.
- Ours (release build of this branch, scratch case):
  - `IMCSC("1E-200")` and `IMCOT("1E-200")` are "1E+200", `IMCSC("1E-307")` is "1E+307" and `IMCOT("2.3E-308")` is
    "4.34782608695652E+307"; `IMSEC("1E-200")` and `IMSECH("1E-200i")` are "1";
  - `IMCSCH("1E-200i")` and `IMCSC("1E-200i")` are `#NUM!`.
  - The real-argument path is SIN/COS's kernel (T-2534, `real_trig`).
  - Excel's `IMCSC("1E-20i")` is `#NUM!` (probes-numeric/im-csc-cot-tiny-imag, T-2531), so `#NUM!` for 1E-200i is
    plausible but unprobed. `IMCSCH("1E-200i")` is csch(i·y) = -i·csc(y), whose true value -1E+200i is finite.
- Code: `imcsc`, `imcot`, `imsec`, `imcsch`, `imsech` and `real_trig` in
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Probe
excel-probe skill, excel-graph, tag M9. Name the case probes/t4603-im-reciprocal-tiny-parts.
- Tiny real: IMCSC, IMCOT, IMSEC, IMCSCH and IMSECH of `"1E-20"`, `"1E-100"`, `"1E-200"`, `"1E-307"`, `"2.3E-308"` and
  `"-1E-200"`.
- Tiny imaginary: the same five functions of `"1E-20i"`, `"1E-100i"`, `"1E-200i"`, `"1E-307i"` and `"-1E-200i"`.
- Controls: `"2.2E-308"`, a subnormal input, should read `#NUM!` from the parse, and `"0"` should give the
  zero-denominator `#NUM!`.

## Acceptance criteria
- [ ] probes/t4603-im-reciprocal-tiny-parts is committed with its excel-graph oracle sidecar.
- [ ] Every probe cell passes, or each visibly-off cell has an owner task.
- [ ] The engineering.md bullet states the tiny real and tiny imaginary results with these cells and the t3609 cells.
      Both UNVERIFIED claims above are gone, and the `IMCOT("1E-310")` example cites the parse rule.
- [ ] If the engine changes, a unit test in trig.rs pins one changed cell.
- [ ] No new known-failure line in eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other notes in the same section (IMPRODUCT's blank in a range, IMCOSH's tiny imaginary part): T-4604.
- Large |x| or |y| past the overflow edges (T-2631, T-4425).
