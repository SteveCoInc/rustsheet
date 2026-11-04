---
id: T-3970
title: "RATE: why Excel still restarts on noise-only nper-1/2 residuals under the keep-r0 rule (after T-3963)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4239]
campaign: accuracy
---

## Goal
RATE's cells off Excel since T-3963's keep-r0 rule are explained, by a probe-backed rule or a recorded negative
finding.

## Context
- Applies only if the owner adopts keep-r0 (T-4239, option 1). T-3963 (financial.md "RATE keeps its older iterate
  when a step leaves the residual unchanged", decision 0260 on PR #3019's branch) replaces T-3698's restart with
  keep-r0 there. It lost 37 gated probes cells, among them nper-1 and nper-2 calls whose
  residual is rounding noise everywhere, where Excel's value is still T-3698's restart:
  `RATE(1,18770705030.2495,-18770705030.2495,0,1,0.120430283703524)` (probes/t3906-rate-restart-oos A15, Excel
  0.12073028370352401, the restart's start offset from the older iterate) and A64.
- A hybrid (keep-r0 plus T-3698's restart in its steps 3..16 and |f| band) scored worse on train, test and the fresh
  sweep (T-3963 Outcome), so the restart condition, if any, is something else (perhaps f2 = f1 = f0 on three points,
  or a residual of exactly 0).
- Tools: docs/research/t3963/ (`trace.diff`, `score.py`, `holdout.py`, the extract of all literal RATE cells).
- Known failures to own once keep-r0 lands (T-4239 owns them until then):
  - every cell of probes/t3698-rate-flat-restart that is off Excel;
  - every cell of probes/t3698-rate-flat-restart-fresh that is off Excel;
  - every cell of probes/t3906-rate-restart-oos that is off Excel;
  - every cell of probes/t3850-rate-same-state that is off Excel;
  - every cell of probes/t3963-rate-keep-r0-fresh that is off Excel.
  - The cases are named here on lines with no cell address, so this task owns every cell of them. It overlaps with
    T-3424 on the error-against-number cells.

## Acceptance criteria
- [ ] A rule, ranked with decision 0188 R4's hold-out and a fresh sweep, that wins back cells of decision 0260 with
      no net loss; or a recorded finding that none on the evidenced inputs does.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- The zero-step `#NUM!` cells (T-3424).
