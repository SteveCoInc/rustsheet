---
id: T-3974
title: "Probe typed IFS, LET and SWITCH argument counts past 6: whole pairs accepted, a broken pair refused"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The parser's typed-input arity rule for IFS, LET and SWITCH (`arg_counts::takes_count` with `PAIRED`) has Excel
evidence beyond the 2-6 argument range T-2682 probed, including at the table's maxima. The rule is then either
confirmed in syntax.md or fixed.

## Context
- Gap triage 2026-11-03 (opus, `make eval` on origin/main 69b7d87b). Every failing cell has an open owner, so this
  triage turned to UNVERIFIED engine rules that no open task names. docs/excel-semantics/syntax.md "Maxima and pairs,
  typed" (T-2682) says: "UNVERIFIED: ... the pair rule above 6 arguments (probed IFS 2 to 5, LET 3 to 6)".
- Engine: crates/rustsheet-engine/src/functions/kinds/arg_counts.rs `PAIRED` = IFS even, LET odd, and
  arg_counts.txt gives `IFS 2 255`, `LET 3 255`, `SWITCH 3 255`. So the engine accepts typed IFS up to 254 (255 is
  odd and refused by the pair rule), LET up to 255, and SWITCH at any count from 3 to 255. None of these maxima is
  probed for the three functions. T-2709 probed 255/256 for SUM-like functions and 254/255 for CONCAT and TEXTJOIN,
  but not these. SWITCH has no pair rule in the engine: `SWITCH(1,2,3,4)` (a default after the pairs) is accepted,
  so every count is a whole pair or a pair plus a default.
- Method: the T-2682 probe (probes/t2682-typed-max-arg-counts; excel-probe skill, excel-graph, typed via Graph PATCH).
  Refused inputs (400 `InvalidArgument`) cannot be committed, so log each refusal in the case description and in
  syntax.md, with the date, as T-2682 did. Accepted calls go in column A of a new probe case with their values. Use
  values that tell the arms apart (`=IFS(FALSE,1,FALSE,2,FALSE,3,TRUE,4)` is 4).
- Counts to type: IFS 6, 7, 8, 9, 252, 253, 254, 255; LET 7, 8, 9, 253, 254, 255 (as `LET(a1,1,a2,2,...,a1)`);
  SWITCH 5, 6, 7, 253, 254, 255, 256.

## Acceptance criteria
- [ ] A probe case (for example probes/t3974-typed-pair-counts, tagged M5) holds the accepted calls with excel-graph
      values. Its description and syntax.md list each count Excel refused, with the Graph error and date.
- [ ] syntax.md replaces the UNVERIFIED pair-rule sentence with the observed rule for all three functions and cites
      the case cells.
- [ ] Where Excel differs from `takes_count`, this PR fixes `PAIRED` or arg_counts.txt (the generated table's
      tooling and tools/tests/test_catalog_arity.py `MAX_OVERRIDES` stay consistent) and adds a parser unit test. A
      larger change gets an engine task filed with `make new-task`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other repeating-group functions (SORTBY, the *IFS aggregates, MAP), which syntax.md already covers.
- Stored-file arity, which the importer reads as written (syntax.md "stored text keeps such a call").
