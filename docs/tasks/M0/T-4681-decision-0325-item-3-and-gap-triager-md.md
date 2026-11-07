---
id: T-4681
title: "Decision 0325 item 3 and gap-triager.md: restate R1(c)'s left-out rule exactly (T-3994 review follow-up)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Decision 0325 item 3, and the sentence in .claude/agents/gap-triager.md that restates it, say exactly what decision
0188 R1(c) leaves out once a probe cell has a math value.

## Context
- T-3994's review (excel-fidelity, PR #3727, MINOR): item 3 says the cell is Phase 1b evidence "when Excel and the
  engine are both within ε" of the math, "otherwise an ordinary visibly-off cell". R1(c) is looser: a numeric cell is
  left out when the engine is within 15 significant digits of Excel or within ε of the math (R2's envelope); Excel's
  distance from the math only decides `accuracy` against `visible` for a cell that still counts. The stricter text
  over-owns cells (harmless) but reads as a restatement of R1(c).
- gap-triager.md: "Phase 1b evidence if Excel and the engine are within 1e-9 of it, else a `campaign:` group".

## Acceptance criteria
- [x] Decision 0325 gets an "Amended by" line or an amendment record (`make new-decision`) whose item 3 reads: left
      out when the engine is within ε of the math or within 15 digits of Excel (R1(c)); otherwise a `campaign:` group,
      `accuracy` when Excel is the closer to the math.
- [x] gap-triager.md and the steward skill's "Deferred owners on the exit line" match it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to the scripts or gates.

## Handoff (resumed)
Sonnet run (2026-11-07) stopped before editing: the acceptance criteria change decision 0325 (a protected path,
work-loop step 5b) and need an amendment record via `make new-decision` or an "Amended by" line that reviewers approve.
Nothing was tried. Next: amend 0325 item 3 to R1(c)'s wording (left out when the engine is within epsilon of the math or
within 15 digits of Excel; otherwise a `campaign:` group, `accuracy` when Excel is closer to the math), then match
.claude/agents/gap-triager.md and the steward skill's "Deferred owners on the exit line".

## Outcome
- Decision 0347 (amendment record) restates decision 0325 item 3 as decision 0188 R1(c) reads: once a probe cell's
  math is attached, it is left out (Phase 1b evidence, its deferred task keeps it) when the engine is within ε of the
  math or within 15 significant digits of Excel; otherwise it gets a `campaign:` group, `accuracy` when Excel is the
  closer to the math. Decision 0325 carries an "Amended by decision 0347" line and a pointer on item 3.
- .claude/agents/gap-triager.md and the steward skill's "Deferred owners on the exit line" state the same rule.
- Prose only: no script, gate or eval change; eval delta 0.
