---
id: T-3938
title: "CUMPRINC one-period last digits on T-3552's evidence probes (387 gated probes cells, Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3776]
defer: phase-1b
---

## Goal
CUMPRINC returns Excel's bits on the 387 one- and few-period principals of T-3552's two evidence probes, so these
known-failure lines leave eval/baseline/probes.txt with no new ones.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (CUMPRINC/CUMIPMT) measured on fin-cumipmt-tiny-rate-structure, 0 of 226 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
- Gap triage 2026-11-03m (`make eval` on origin/main ea84dadb9, excel-graph evidence; per-cell `owned` run over every
  failing cell against every open task, deferred ones included, and every unmerged branch's task files). T-3552 and
  T-3731 (done) built these probes as evidence for the CUMIPMT fit and named them only in Context; T-3776 (blocked,
  time-boxed, decision 0056) owns the 37 visibly-off CUMIPMT cells of probes/fin-cumipmt-tiny-rate-structure and
  needs the same one-period form. No open task owns these cells.
- probes/fin-cumprinc-single-period-rate-scan (276 failing) and probes/fin-cumprinc-dyadic-rates (111 failing):
  all NumericUlp, 1-3 ulp, within 15 digits, gated in eval/baseline/probes.txt.
- Examples (Excel vs ours), `make eval-show CASE=probes/<case>`:
  - rate-scan Sheet1!A10 `=CUMPRINC(0.0000152587890625,100,100000,12,12,0)`: Excel -999.4126166786187, ours ...189.
  - dyadic Sheet1!B3 `=CUMPRINC(0.0000152587890625,100,100000,3,5,0)`: Excel -2997.8718795668774, ours ...765.
  - dyadic Sheet1!J1 `=CUMPRINC(0.00006103515625,100,100000,3,3,1)`: Excel -997.0427147109465, ours ...466.
- Root cause (from financial.md "CUMIPMT and CUMPRINC over many periods", T-3552 and T-3731 attempts): over one
  period Excel's CUMPRINC is PPMT·(1+δL), δL the relative error of `growth_log`'s L, with at least two
  period-dependent roundings; no candidate family reached more than 34% bit-exact. The 2^-16 rate cells above show
  the gap does not vanish at every dyadic rate. Code: crates/rustsheet-engine/src/functions/financial/cumulative.rs.
- This task depends on T-3776: when its attempt 3 finds the form, these cells are its tune and hold-out set; until
  then nothing here is visibly off. The misses are below decision 0112's floor, so a form chosen on them needs the
  R4.4 check or an owner ruling.

Cells (gap triage 2026-11-03m; 387 cells):
- probes/fin-cumprinc-dyadic-rates.case.json (111): Sheet1!A4:A5, A8, A10, A12, B3:B9, B11:B12, D1, D3, D5, E1, E3:E9, E11:E12, G2:G3, G6, G9, G11:G12, H3, H5, H8, H12, J1:J3, J5:J11, K1:K2, K4:K5, K7:K8, K12, M1, M3, M5, M7, M11:M12, N1, N4:N8, N11:N12, P3:P4, Q3:Q4, Q6:Q8, Q12, S1, S3, S5:S7, S9, S12, T1, T3:T10, T12, V1:V7, V10:V12, W1:W2, W4, W6:W8, W10:W11
- probes/fin-cumprinc-single-period-rate-scan.case.json (276): Sheet1!A1:A2, A4:A8, A10:A12, A14:A16, A18:A20, A24:A33, A35:A36, A38, A40:A43, A45:A48, A53:A54, A56, A58, A60, C1:C2, C4:C7, C10:C12, C14:C18, C20, C24, C26:C33, C35:C38, C40, C42:C48, C50:C51, C54:C60, E1:E2, E4, E6, E10:E11, E14:E20, E24:E33, E35:E40, E42:E44, E46:E51, E53:E54, E56:E59, G1:G8, G10:G12, G14:G21, G23:G27, G29:G33, G35:G38, G40:G48, G50:G51, G54, G56:G58, G60, I1:I8, I10:I13, I15:I22, I24:I28, I30:I36, I38, I40, I42:I43, I45:I49, I51, I54:I60, K1:K20, K23:K33, K35:K38, K40:K42, K44, K46:K48, K51, K54:K56, K58

## Acceptance criteria
- [ ] These 387 known-failure lines disappear from eval/baseline/probes.txt with no new ones, or the Outcome lists
      the ones still open with the form tried.
- [ ] No CUMPRINC, CUMIPMT, PPMT or IPMT cell of any suite regresses.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), in the Outcome:
  - hold-out counts: train against hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and
    the correctly rounded value, from the fit lab's split, with the margin stated before scoring;
  - a fresh log-uniform sweep over the owned domain, generated and oracle-filled after the model is frozen, with its
    result recorded;
  - the Gate A envelope result on the fresh sweep and on every owned cell, every point outside listed;
  - every fitted constant or cut carries `FITTED(T-3938, <n> cells, <range>)` and applies only inside that range,
    with accurate math outside it;
  - no fit below the oracle's reproducibility floor (4 ulp web against desktop, decision 0112) without a
    desktop-backed reproducibility check or an owner ruling (R4.4);
  - no keep-matched branch that special-cases evidenced points.
- [ ] financial.md's CUMPRINC entry records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 37 visibly-off CUMIPMT cells (T-3776) and T-3413's cells.

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
