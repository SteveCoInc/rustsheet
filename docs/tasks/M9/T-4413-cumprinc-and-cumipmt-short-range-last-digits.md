---
id: T-4413
title: "CUMPRINC and CUMIPMT short-range last digits at rates 1E-12 to 1E-4: 190 T-3524 tiny-rate-sweep cells within 15 digits, after T-4390 (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4390]
defer: phase-1b
---

## Goal
Each listed short-range cell (32 periods or fewer) of the tiny-rate sweep returns Excel's
double once T-4390 has settled the short-range forms, or the Outcome lists it with Excel, ours, the 60-digit reference
(fit lab `kernels/cumulative.py`) and the reason Excel's last bits cannot be derived. The cells are CUMPRINC, and the
CUMIPMT built on it, at rates 1E-12 to 1E-4. Every one is already within half a unit of the 15th significant digit, so
this is Phase 1b work only (decision 0114).

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 952336450). Ownership was checked with `scripts/exit_owners.py`'s
  `owned` over every failing cell, not only the visibly-off ones.
  - The sweep's rows 1-250 are short ranges: rows 1-125 type 0, rows 126-250 type 1 (`docs/research/t3524/gen_sweep.py`,
    seed 35241). 190 of their failing cells have no owner: 148 CUMPRINC (column A) and 42 CUMIPMT (column B).
  - T-4390 owns the visibly-off cells of this case (140) and names them, so this task shares none of them (split note,
    T-3229).
  - None of the 190 cells is visibly off. Distance from Excel: 95 cells at 1 ulp, 63 at 2, 18 at 3-4, 14 at 8-16. The
    cells past 4 ulp are CUMIPMT, which T-3118 builds as `PMT·m - CUMPRINC`.
- Examples (Excel vs ours):
  - A10 `CUMPRINC(3.16465044916096E-07,44471,15887.03498,69,76,0)`: -2.837959739224426 vs -2.8379597392244267.
  - A100 `CUMPRINC(8.47942311297401E-10,6316619,6.601225785,34,48,0)`: -1.5633909419709034E-05 vs
    -1.563390941970903E-05.
  - B94 `CUMIPMT(3.48727931011995E-06,26480,104852.1445,1,4,0)`: -1.4625157717925958 vs -1.4625157717925994.
- Root cause: Excel's short-range form at a tiny rate. Neither the per-period sum below `GEOMETRIC_MIN_LEAD` nor the
  geometric sum in decision 0296's band is Excel's there: each is bit-exact on only about 30% of the cells where they
  differ (T-4349). T-4390 hunts for Excel's own form on the visibly-off cells. T-3776 (blocked) and T-3413 (blocked) own
  the same question at a tiny rate and in CUMIPMT's amplification, on other probes. Any form T-4390 lands moves these
  cells, hence the dependency: score them after it.
- Reproducibility floor (R4.4): most differences are 1-2 ulp, inside the 4-ulp web/desktop floor
  (docs/research/oracle-calibration.md), and the sweep has one excel-graph fill (2026-11-05). Refill it before fitting.
- The evidence is already cached: 250 short-range rows, both functions. A new fit needs a fresh sweep after its freeze
  (R4.2), drawn with `gen_sweep.py` on a new seed and oracle-filled inside this task, so no separate research sweep
  task is needed first.
- Code: `crates/rustsheet-engine/src/functions/financial/cumulative.rs` (`short_takes_geometric`,
  `GEOMETRIC_MIN_LEAD`). Decision 0056 applies: after two attempts that do not raise the bit-exact count, time-box.
- Cells:
  - probes-numeric/t3524-cumprinc-tiny-rate-sweep.case.json (190): S!A2, A4, A5, A7, A9, A10, A11, A12, A15, A17, A18, A21, A22, A23, A26, A30, A31, A32, A33, A35, A36, A37, A38, A39, A44, A46, A49, A50, A52, A53, A54, A55, A56, A57, A58, A59, A60, A61, A63, A65, A66, A67, A69, A70, A73, A75, A78, A81, A82, A85, A86, A87, A89, A90, A91, A92, A94, A97, A98, A99, A100, A101, A102, A108, A109, A111, A112, A115, A116, A117, A120, A121, A122, A124, A126, A127, A129, A131, A132, A133, A137, A138, A139, A140, A141, A142, A143, A144, A146, A148, A150, A154, A155, A158, A160, A161, A162, A163, A164, A165, A166, A169, A171, A173, A174, A176, A180, A181, A182, A184, A185, A186, A192, A193, A194, A195, A196, A197, A201, A204, A205, A208, A209, A210, A211, A213, A214, A215, A216, A218, A220, A221, A222, A223, A224, A225, A226, A229, A230, A236, A240, A242, A243, A244, A246, A247, A248, A250, B2, B18, B21, B22, B23, B30, B35, B38, B46, B56, B57, B58, B63, B66, B67, B75, B94, B97, B117, B124, B133, B138, B143, B158, B165, B171, B174, B176, B180, B184, B185, B192, B194, B196, B201, B208, B210, B213, B221, B222, B225, B230

## Acceptance criteria
- [ ] After T-4390, a refill of the sweep (excel-graph) reports how many listed cells changed bits. Cells that vary
      between fills stop being targets (R4.4, decision 0112) and are listed in the Outcome.
- [ ] One of these holds:
      - a short-range form makes the listed CUMPRINC cells bit-exact, with the CUMIPMT cells following through
        `PMT·m - CUMPRINC`, and no cell T-4390 brought within 15 digits leaves it;
      - or the Outcome lists every cell still off, with Excel, ours and the 60-digit value, and ranks the forms tried
        by bit-exact count (fit lab `cumulative-tiny`).
- [ ] Any kept form, band or selector chosen by scoring passes every R4 guard of reviewer-excel-fidelity check 7
      (decision 0188), each shown in the Outcome:
      - hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the
        candidate and for the correctly rounded reference, from the fit lab's T-3360 split, with the margin fixed
        before scoring;
      - a fresh log-uniform sweep, drawn and oracle-filled after the freeze, and its result;
      - the Gate A envelope count on that sweep, with every point outside the envelope listed;
      - a `FITTED(T-4413, n_cells, range)` tag, applied only inside its range;
      - no step that fits differences below the reproducibility floor;
      - no keep-matched branch.
- [ ] The listed cells leave the probes-numeric failing list. Because probes-numeric is gate = false, it has no
      eval/baseline file. No new failing cell appears in the sweep, and no eval/baseline/*.txt file gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells (T-4390); long ranges (T-4412); an underflowed v^nper (T-3141).
