---
id: T-4643
title: "F.INV and F.INV.RT last digits on T-4497's df 2 a = 1 tail sweeps: 366 quantile and 78 F.TEST cells within 15 digits no task names (Phase 1b, after T-4498)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4498, T-3485]
defer: phase-1b
---

## Goal
The F quantile calls of T-4497's sweeps, `F.INV(p,2,df2)` and `F.INV.RT(p,df1,2)`, give Excel's bits: the engine's
search stops where Excel's does. Whatever F.TEST residue T-4498 leaves on the same two cases is fixed or named. Today
these cells are known failures that no open task names. They are all within 15 digits, so this is Phase 1b work.

## Context
- Gap triage 2026-11-06 (report of `make eval` at 37389d18, calipers and poi fetched). `make unowned-off ALL=1` lists
  0 unowned visibly-off cells. A scan of every failing cell against open-task ownership (exit_owners `owned`) leaves
  584 unowned. 444 of them are on T-4497's two probes-numeric cases, which no task names cell by cell. T-3485 names only
  the visibly-off ones and T-4498 names neither case.
- Quantile cells, 366 (F.INV column B, F.INV.RT column C; 160 ulp1, 173 within 4 ulp, 33 digits15). Examples
  (`rustsheet-eval show probes-numeric/t4497-f-a1-cut-dense.case.json`), Excel vs ours:
  - dense C15 `F.INV.RT(0.390194931650792,9,2)`: 1.912694633594892 vs 1.9126946335948936 (digits15);
  - dense B1 `F.INV(0.191075619214687,2,4)`: 0.22369916551127794 vs 0.22369916551127822 (digits15);
  - sweep C122 `F.INV.RT(0.0628502742388917,1,2)`: 14.427051899076796 vs 14.427051899076776 (digits15).
- F.TEST cells, 78 (column A; 63 ulp1, 15 within 4 ulp), e.g. dense A1 `F.TEST({-1,0,1},{...})`:
  0.843773042720988 vs 0.8437730427209882. T-4497 finding 2: with Excel's expm1/log1p below the 0.1 cut, all 220
  F.TEST cells are bit-exact (71 of 71 held out). T-4498 makes that change, so this task depends on it and only owns
  what it leaves.
- Root cause (docs/research/t4497/README.md, finding 3): the df 2 quantiles go through the crossing search
  (`fisher.rs` `f_quantile_tail` → `unoriented_shape_one_tail`). It stops a few ulps from the crossing, and the stop
  moves with the tail's last bits in no consistent direction. Only about 14% of the 440 quantile cells are bit-exact
  under any of the three tail forms tried. At df1 = 2 (or df2 = 2 for the upper tail) the a = 1 tail has a closed form,
  1 - (1 - x)^b, so its inverse is closed-form too. Excel's bits are not that inverse's: the cells it misses look like
  the stop of Excel's search. Which search, which stop rule and which start is the open question. T-3485 has the same
  question at p = 0.5 (Excel's y 2+ ulps from any crossing). Its visibly-off cells come first, so this task depends
  on it.
- Two of the dense cells are visibly off in this report and are listed in T-3485, not here: B21 `F.INV(0.420180003002338,2,10)`
  0.5758536525446335 vs ...327, and B34 `F.INV(0.483291248020393,2,12)` 0.6979764172164359 vs ...364. T-4497 found
  B9, B41, B58, C9, C27 and C44 off instead, so which cells are off moves with engine changes.
- Needs an oracle sweep for the R4 fresh-sweep guard only. The 660 cells of T-4497 are the training data, with its
  fixed split (sha256(case!cell) mod 100 < 30 held out, margin 5 points). `points.tsv` has 50-digit math for every
  point.
- Cells this task owns (all on the a = 1 tail, x 4E-4..0.2):
  - probes-numeric/t4497-f-a1-cut-dense.case.json (8 F.TEST): Calc!A1, A17, A29, A31, A43, A44, A49, A58
  - probes-numeric/t4497-f-a1-cut-sweep.case.json (70 F.TEST): Calc!A2, A4, A6, A11, A12, A13, A14, A17, A19, A22, A25, A26, A27, A32, A33, A34, A35, A37, A38, A42, A43, A46, A47, A48, A51, A55, A56, A57, A59, A60, A63, A66, A67, A68, A71, A72, A76, A78, A82, A83, A85, A86, A88, A89, A91, A94, A98, A100, A101, A105, A106, A107, A108, A111, A114, A117, A118, A120, A121, A122, A125, A129, A139, A145, A146, A148, A150, A152, A153, A157
  - probes-numeric/t4497-f-a1-cut-dense.case.json (94 quantiles): Calc!B1, B3, B4, B5, B7, B8, B10, B11, B12, B14, B16, B17, B18, B19, B22, B23, B24, B25, B26, B27, B28, B30, B31, B32, B33, B35, B36, B37, B40, B42, B43, B45, B46, B47, B48, B49, B50, B51, B52, B53, B54, B55, B56, B57, B60, C1, C3, C4, C7, C8, C10, C12, C13, C14, C15, C16, C17, C18, C19, C20, C22, C23, C24, C25, C26, C28, C29, C30, C31, C32, C33, C34, C35, C36, C37, C38, C40, C41, C42, C45, C46, C47, C48, C49, C50, C51, C53, C54, C55, C56, C57, C58, C59, C60
  - probes-numeric/t4497-f-a1-cut-sweep.case.json (272 quantiles): Calc!B1, B2, B3, B5, B6, B7, B8, B9, B10, B11, B12, B14, B15, B16, B17, B18, B19, B20, B21, B22, B24, B25, B26, B27, B28, B29, B31, B32, B33, B34, B35, B36, B37, B38, B40, B42, B43, B44, B46, B47, B48, B50, B51, B52, B53, B54, B55, B57, B58, B59, B61, B62, B63, B64, B66, B67, B68, B69, B70, B71, B72, B73, B74, B76, B77, B79, B80, B82, B83, B84, B85, B86, B87, B88, B89, B91, B92, B95, B97, B98, B99, B100, B102, B103, B104, B105, B106, B108, B110, B111, B112, B113, B114, B115, B116, B117, B119, B120, B121, B122, B123, B124, B125, B126, B127, B128, B129, B130, B131, B132, B133, B134, B135, B137, B138, B139, B140, B141, B142, B143, B144, B145, B146, B148, B149, B150, B151, B152, B153, B154, B156, B158, B159, B160, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C12, C14, C15, C16, C17, C18, C19, C20, C21, C23, C24, C25, C26, C27, C28, C29, C30, C32, C33, C34, C35, C36, C38, C40, C42, C44, C45, C47, C48, C49, C50, C51, C53, C54, C56, C58, C60, C61, C62, C63, C64, C65, C66, C67, C68, C69, C70, C71, C72, C73, C74, C75, C76, C77, C78, C80, C81, C82, C83, C84, C85, C86, C87, C88, C89, C90, C91, C92, C93, C94, C95, C96, C97, C98, C99, C100, C101, C103, C106, C107, C108, C109, C110, C111, C112, C113, C114, C115, C116, C117, C118, C119, C120, C121, C122, C124, C126, C127, C128, C130, C131, C132, C133, C134, C135, C136, C137, C138, C139, C140, C141, C143, C144, C145, C146, C147, C149, C150, C151, C152, C153, C154, C155, C156, C157, C158, C159, C160

## Acceptance criteria
- [ ] The bit-exact count of the quantile cells above rises on T-4497's hold-out (194 cells) as well as on its training
      cells, under a stop rule chosen on the training cells only. If no candidate beats today's on the hold-out, the
      engine is unchanged and the Outcome lists the candidates tried with their scores.
- [ ] R4 guards (decision 0188; reviewer-excel-fidelity check 7), each shown in the Outcome:
  - [ ] Hold-out: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        stop rule and for the correctly rounded quantile, on T-4497's split with its 5-point margin. The candidate
        beats correctly rounded on the hold-out, and its hold-out exact rate is within the margin of its train rate.
  - [ ] Fresh sweep: after the rule is frozen, a new oracle-filled probes-numeric case (or an `r4-T-4643` batch of
        the F.INV Gate A table) of at least 150 df 2 quantile calls over p 1E-6..0.5, never used for fitting.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope (`rustsheet-accuracy`), with every point outside
        it listed.
  - [ ] `FITTED` tag and range: any constant or rule fitted here carries `FITTED(T-4643, n_cells, range)` with the
        sweep's p and df range, and applies only inside it; outside, the code keeps today's search.
        `quality/fitted.json` is regenerated by its script, never hand-edited.
  - [ ] Reproducibility floor: no step fits differences below the oracle's floor (web against desktop up to 4 ulp,
        decision 0112).
  - [ ] No keep-matched branch: no branch that keeps the old stop only where it happened to match (decisions 0056,
        0082).
- [ ] No cell becomes visibly off: the quantile cells and F.TEST cells listed above that leave the failing set do so
      with no new failing cell in eval/baseline/gen-m9.txt, eval/baseline/probes.txt or eval/baseline/ironcalc-calc.txt
      (`make eval-baseline` refuses regressions). probes-numeric is ungated: its before and after counts are in the
      Outcome.
- [ ] statistics.md gains or updates the entry for the df 2 F quantile's stop, citing the cases above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The a = 1 tail cut and its kernels (T-4498).
- Visibly-off F quantile cells (T-3485), the p = 0.5 mapping cells and the general-df quantile last digits (T-3225,
  T-3233, T-2767).
