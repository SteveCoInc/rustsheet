# 0255 — T-3698: fitted lock 8 to 9 in financial_util.rs for RATE_RESTART_STEPS

- **Status:** accepted (2026-11-03, T-3698). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3698 makes RATE restart its secant once, from the older iterate, on an early flat secant
  (`rate_secant` in crates/rustsheet-engine/src/functions/financial/financial_util.rs; docs/excel-semantics/
  financial.md, "RATE restarts once on an early flat secant"). The step bound `RATE_RESTART_STEPS` = 16 was chosen
  by scoring oracle cells, so it is a fit (decision 0188 R4):
  - it carries `FITTED(T-3698, 1477 cells, RATE flat secant at steps 3..16 with |f1| 1.3E-7..1E-4)` and applies only
    in that range (`RATE_RESTART_FIRST`, `RATE_RESTART_F_MIN`, the cap's bound);
  - its train-only choice on the fit lab's hash split, the fresh Gate A batch accuracy/RATE/r4-T-3698 (300 of 300
    inside the envelope) and the scorers are in T-3698's Outcome and docs/research/t3698.
  - The |f1| cut reuses the existing `RATE_CAP_TOLERANCE`, and the two range ends are evidence floors, not fits; none
    of them adds a tag.
- **Decision:** quality/fitted.json rises by one for financial_util.rs (8 to 9), for this one new fit, written by
  `scripts/check_fitted.py --update`.
- **Consequences:** none beyond the one counted tag. The fitted ratchet stays one-way for new fits, each with its own
  record. The PR loses no gated cell (eval delta in T-3698's Outcome).
