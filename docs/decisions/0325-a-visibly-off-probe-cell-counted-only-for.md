# 0325 — A visibly-off probe cell counted only for want of a math value is owned by the task that attaches its math

- **Status:** accepted (2026-11-06, T-3994).
- **Context:**
  - Gap triage 2026-11-03 (origin/main eda15c023, excel-graph): `make phase1-exit`'s `owners` line was not met on
    61 visibly-off cells owned only by a `defer: phase-1b` task (decision 0187 item 4 counts such a cell as unowned).
    They were probe and probes-numeric cells that decision 0188 R1(c) counted only because they had no math value:
    one-call cells of functions with no Gate A definition yet (CHIDIST 2, CHISQ.DIST.RT 2, POISSON 3, F.DIST.RT 7,
    FDIST 1, GAMMALN 1) and 45 nested `LN(GAMMA(x))` cells. 50-digit mpmath put every sampled one inside ε = 1e-9 for
    both Excel and the engine (T-3994's Context: `F.DIST.RT(1.61,1,50)` Excel -4.7E-15 relative, ours +6.9E-17), so each
    was Phase 1b evidence waiting for a math value.
  - The rules pulled two ways: decision 0245's new-case gate passes a cell owned only by an open `defer:` task, while
    the exit `owners` line (decision 0187, `exit_owners.unowned_off`) does not; decision 0114 and the gap-triage rule
    keep `defer: phase-1b` for cells already within 15 digits and give a visibly-off cell a `campaign:` task. Triage
    decided case by case, and twice (T-3954, T-3952) filed `visible` fit tasks, which is right for Gate B suites
    (gen-m9: every visibly-off cell counts, decision 0188 R1(b)) but would be fit work on probe cells that a math
    value takes out of the count.
  - What cleared them: the Gate A table tasks T-3798 classified attached the math. T-4103 (F.DIST.RT, FDIST:
    t2220-f-dist-sweep-d1-1.math.json), T-4104 (CHIDIST and the T-3638 subnormal probe:
    t3638-subnormal-upper-tail.math.json), T-4106 (GAMMALN: t3570-r5-bpser-lg-aux.math.json), T-3925
    (CHISQ.DIST.RT) and T-3993 (composite cells such as `LN(GAMMA(x))`, decision 0297). On origin/main e200fa21
    (2026-11-06, `make eval` without calipers, poi or corpus-bench), `make phase1-exit` reports
    "2 visibly-off cells with no open owner (2 owned only by a deferred task)". Both are gen-m9 cells, Gate B, outside
    this record: `gen-m9/01523d247594952e!Calc!A377` (SYD over `F.INV.RT(1E-20,9,3)`) and
    `gen-m9/abc2939ebd1e2365!Calc!A11` (`CHIDIST(F.INV.RT(1E-200,1,1.999),10)`, Excel 0, ours `#NUM!`). No probe cell of
    the kind is left: 61 then, 0 now.
- **Options considered:**
  - *(a) Keep the scripts; such a cell needs a non-deferred owner (chosen, with the owner named below).* No gate
    changes. Which non-deferred task owns it is the open question; a `visible` fit task is the wrong one.
  - *(b) Let a deferred owner suffice while an open task will attach the cell's math.* Needs a mechanical "will attach"
    test in phase1_exit.py (a function in a table task's list, a formula shape in a sidecar task's scope) and weakens
    the owners line for as long as that task is open.
  - *(c) Leave such cells out of the owners line and count them apart.* Weakens decision 0187 item 4 outright; the
    cells would block nothing while no task had them.
- **Decision:**
  1. A visibly-off numeric cell of the probes or probes-numeric suites that R1(c) counts only because it has no math
     value (no `.math.json` value, or a stale one) is not a last-digit cell for ownership. A `defer: phase-1b` task
     alone does not own it for the exit `owners` line; the scripts are unchanged (decision 0187 item 4 stands).
  2. Its owner is the open, non-deferred task that attaches its math value: the Gate A definition or table task of its
     function (SPEC-0029, decision 0188 R2) when the function has none, or a math-sidecar task
     (`rustsheet-accuracy math`, SPEC-0029 "Math references for eval cells") when the function has a definition but the
     cell's formula shape gets no math yet (a composite with an undefined call, an argument the reader leaves unparsed,
     a skipped candidate). Gap triage names the cells in that task (decision 0136's naming rule), filing it when none
     exists; it files no `visible` or `accuracy` fit task for them before the math is attached.
  3. Once the math is attached, the cell is Phase 1b evidence when Excel and the engine are both within ε of it
     (R1(c) leaves it out, so it needs no exit owner and its deferred task keeps it); otherwise it is an ordinary
     visibly-off cell and triage files its `campaign:` task as decision 0114 says (`accuracy` when Excel is the closer).
  4. Gate B suites (gen-*, ironcalc-calc, corpus-bench, calipers, poi, smoke, fixtures) are untouched: every
     visibly-off cell there counts whatever its math, and needs a `campaign:` owner (decision 0188 R1(b)).
- **Consequences:**
  - The three checks agree without a script change: `phase1_exit.py`'s owners line and `exit_owners.py --all-off`
    read the same `unowned_off`, and a cell named in its math task is owned for both. Decision 0245's new-case gate
    still passes a cell owned only by a `defer:` task: it is a per-PR gate that asks that an owner exists, a lower bar
    than the exit's, so a probe PR is not blocked; the exit line then lists the cell until a math task names it, and
    the next triage applies item 2.
  - .claude/agents/gap-triager.md and the steward skill state items 1–3 where they describe `defer: phase-1b`.
  - Counts: 61 such cells on 2026-11-03, 0 on 2026-11-06 (the table and sidecar tasks of item 2 landed); the 2 cells
    the owners line lists today are gen-m9 cells under item 4.
