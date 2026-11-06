---
id: T-4604
title: "engineering.md: settle four stale notes (IMPRODUCT blank in a range, IMCOSH tiny imaginary part, IMABS near DBL_MAX, OCT2BIN of a blank) from the probes that answered them"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Four engineering.md notes no longer mark as UNVERIFIED or unprobed what committed Excel cells already answer. Each one
cites those cells, or points to the section that does. The duplicate OCT2BIN section is folded into the base-conversion
rule.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage checked the semantics notes against the probes committed since they were written.
  Every case cited below passes (a `C` line and no `F` line in eval/baseline/probes.txt).
1. "IM* results that overflow (T-1827)", the IMPRODUCT bullet: "(a blank inside a range is UNVERIFIED, T-2231)".
   probes/eng-im-overflow-edges-2 A55:A57 (`IMPRODUCT(C1:C3)` with C2 blank, `IMPRODUCT(C2:C3)`, `IMPRODUCT(C2)`) answer
   it. The rule is already stated in "IMCOSH and IMSINH past |x| = 709 work in log space (T-2231)" ("IMPRODUCT of a
   range holding a blank ... is "0"").
2. The same section, the IMCOSH/IMSINH bullet: "A tiny nonzero imaginary part with a large real one is UNVERIFIED".
   probes/eng-im-overflow-edges-2 A9, A10, A21, A22 (`IMCOSH("710.4+1E-300i")`, `"710.4+1E-10i"`, and the IMSINH
   twins) and A113 (`IMSINH("709.5+1E-200i")`) answer it (T-2231's Outcome: "a tiny imaginary part").
3. "IM* reciprocal kernels, IMABS and IMARGUMENT (T-2044)", the IMABS bullet: "A part near DBL_MAX overflows the
   scaled form to infinity as `hypot` does (`IMABS("1E308+1E308i")`); that edge is UNVERIFIED". That input never
   reaches the kernel. "An input part of `1E308` or more is `#NUM!` in every IM* function" (`IMREAL("1E308")`; "IM* results
   that overflow (T-1827)", probes/eng-complex-overflow, excel-graph), so |z| stays below sqrt(2)·1E308, which is finite. Ours gives `IMABS("9E307+9E307i")` =
   1.2727922061357857E+308 (scratch case on this branch). The note should say the overflow is unreachable and cite the
   parse rule. A probe of `IMABS("9.99999999999999E+307+9.99999999999999E+307i")` is optional.
4. "OCT2BIN of a blank cell is 0" (T-1413) still says "The other X2Y conversions of a blank cell, and of typed empty
   text, are unprobed", "Ours: `#NUM!` for OCT2BIN, OCT2DEC and HEX2BIN of a blank cell" and "the rest UNVERIFIED".
   T-1413 then probed all of them: probes/eng-base-conv-blank, 46 cells. "Base conversions read a blank cell and empty
   text as 0" (VERIFIED, T-1413) states the general rule. The OCT2BIN section is a second definition of it.

## Acceptance criteria
- [ ] Notes 1 and 2 cite the eng-im-overflow-edges-2 cells above (or point to the T-2231 section), and their
      UNVERIFIED wording is gone.
- [ ] Note 3 says why the IMABS overflow is unreachable, citing the 1E308 parse rule, and no longer says UNVERIFIED.
- [ ] "OCT2BIN of a blank cell is 0" is folded into "Base conversions read a blank cell and empty text as 0" (its
      gen-m9/d244411dd90f7fbd A15 citation kept) and deleted, so the rule has one definition.
- [ ] Docs only, unless the optional IMABS probe is added (then its case and sidecar under eval/suites/probes, all
      cells passing). `make check` green; reviewers approve.

## Out of scope
- The IMSEC/IMCSC/IMSECH/IMCSCH tiny-argument bullet in the same section: T-4603.
- "Engineering edge-pool smoke check (T-2249)" and the IMLOG2 "UNVERIFIED until T-2044" note: T-3306 items 4 and 5.
- IMARGUMENT's overflow and subnormal edges: T-2736 probes them next to IMPOWER.
