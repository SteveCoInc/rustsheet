# 0326 — A deferred task never owns a counted visibly-off cell; the owner of one with no math value attaches it

- **Status:** accepted (2026-11-06, T-3994). Amends decision 0245 (its new-case gate no longer passes a cell
  owned only by an open `defer:` task).
- **Context:**
  - Gap triage 2026-11-03 (origin/main eda15c023): `make phase1-exit` reported "owners not met: 61 visibly-off
    cells with no open owner (61 owned only by a deferred task)". Most were probe cells (CHIDIST, CHISQ.DIST.RT,
    POISSON, F.DIST.RT, FDIST, GAMMALN and 45 nested `LN(GAMMA(x))`) that decision 0188 R1(c) counted only because
    they had no math value; 50-digit mpmath put every sampled one inside 1e-9 for both Excel and the engine.
  - Three rules disagreed on the same cell:
    - decision 0245's new-case gate (`scripts/new_case_owners.py`) passed a cell owned only by a `defer:` task;
    - `scripts/phase1_exit.py`'s `owners` line and `scripts/exit_owners.py --all-off` counted it as unowned;
    - decision 0114 item 3 reserves `defer: phase-1b` for cells already within 15 digits, so a visibly-off cell
      should never have had a deferred owner only.
  - Gap triage re-decided each case: sometimes it filed `visible` fit tasks (T-3954, T-3952 took gen-m9 cells out of
    deferred T-2221), for probe cells a math value would have made Phase 1b evidence.
  - Since then the cells' math values landed: T-4100 to T-4106 (decision 0272's tables), T-3925 and T-3993's
    composite sidecars (decision 0297). On origin/main 9d793da6 (`make eval`, excel-graph; calipers, corpus-bench
    and poi not fetched) the line reads "2 visibly-off cells with no open owner (2 owned only by a deferred
    task)": gen-m9/abc2939ebd1e2365 Calc!A11 `CHIDIST(F.INV.RT(1E-200,1,1.999),10)` (Excel 0, ours #NUM!) and
    gen-m9/01523d247594952e Calc!A377 (`SYD(..F.INV.RT(1E-20,9,3)..)`), both owned only by T-4662, which T-2808
    filed deferred.
- **Options weighed:**
  - (a) Keep the owners line: a deferred task never owns a counted cell.
  - (b) Let a deferred owner suffice while an open task will attach the cell's math value: it needs a mechanical
    "will attach" test (a function in a table task's list, a formula shape in a sidecar task's scope) that
    `phase1_exit.py` would have to parse from prose.
  - (c) Leave such cells out of the owners line and count them apart: a weaker gate, and it hides the cell until
    someone attaches its math.
  - (a) is chosen: it needs no new parser, weakens nothing, and is what the Phase 1 exit already measures. (b) and
    (c) would only relabel the missing math value, which is the real work.
- **Decision:**
  1. A task with `defer:` never owns a visibly-off cell that decision 0188 R1(c) (or Gate B, for gen-* and the other
     R1(b) suites) counts. `phase1_exit.py`, `exit_owners.py --all-off` and `new_case_owners.py` all read such a
     cell as unowned; the new-case gate now fails it (`unowned_new` drops its deferred exemption).
  2. A visibly-off number counted only because it has no math value is owned by an open, non-deferred task that
     attaches its math value: a Gate A table task (decision 0272) or a math-sidecar task (decision 0297 for nested
     formulas). Gap triage names the case there (amending or filing that task), not in a `visible` fit task. Once
     the math value lands, R1(c) either leaves the cell out as Phase 1b evidence or it gets a `campaign:` task.
  3. Any other counted visibly-off cell gets a `campaign:` task (decision 0114). A deferred task that owns one loses
     its `defer:` key and gets the stage instead, or the cell moves to a campaign task.
- **Applied (T-3994):** T-4662 loses `defer: phase-1b` and becomes `campaign: visible` (its two gen-m9 cells are
  visibly off in a Gate B suite). After: "0 visibly-off cells with no open owner (0 owned only by a deferred
  task); 0 milestone exit cells with no owner of their milestone" on the same run; `make unowned-off ALL=1` lists 0.
- **Consequences:**
  - The three scripts agree: one owner rule, no deferred exemption anywhere.
  - A probe or gen PR whose added or refilled case has a visibly-off cell owned only by a deferred task now fails
    `make check`; it files or amends a non-deferred owner (the excel-probe skill's step 4 says which).
  - .claude/agents/gap-triager.md, the steward skill's Phase 1 exit item, docs/process/task-format.md and
    docs/process/quality-gates.md state the rule.
