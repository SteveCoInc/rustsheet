# 0330 — A ready P0 bypasses the busy-lane rule, and next says when a busy lane hides ready work

- **Status:** accepted (2026-11-07, T-3558). Amends decisions 0022 (one task in flight per lane) and 0023 (P0 first).
- **Context:**
  - Decision 0022 lets each lane have one task in flight (three in engine, decision 0269). `make next-task`
    enforces this in `available` (scripts/tasks.py): a lane with a pushed agent branch that has a commit in the
    last 2 h is busy, and every candidate in it is dropped. P0s are dropped too.
  - On 2026-11-01 (origin/main b39e1eae) T-3488, a P0 opus task with no dependencies, was hidden this way. Two
    harness branches were in flight, so `make next-task LANE=harness TIER=opus` printed `no ready task`. Decision
    0023's promise that a P0 "is never delayed by an escalation" did not cover a busy lane.
  - The same run printed `no ready task` for the engine and research lanes, which held 11 and 24 ready tasks. The
    work-loop skill's step 1 sends a worker to the planner on `no ready task`, so each busy lane started a planner
    run, and the busy rule hid that run's new tasks as well.
- **Options considered:**
  - *A P0 bypasses the busy rule only when no P0 of its lane is in flight.* T-3358, in flight on 2026-11-01, was
    itself a P0, so this rule would still have hidden T-3488. It also needs the priorities of branch tasks that
    may not be on main.
  - *A P0 bypasses the busy rule outright* (chosen). The busy rule guards against two tasks of one lane
    conflicting in the same files. A P0 is urgent by definition (a broken gate, a stranded PR, prime directive 5).
    Conflicts are resolved at merge: the queue tests each PR against the PRs ahead of it (decision 0247), and the
    steward skill resolves conflicts first. P0s are a small share of the work (127 of the 4,036 task files on
    2026-11-07), so the extra parallelism is small.
- **Decision:**
  1. `next` and `ready` offer a ready, unclaimed P0 whatever its lane is doing. A claimed P0 (a pushed branch of
     another worker) is still skipped, and a P0's own branch still makes its lane busy for non-P0 tasks.
  2. When `next` finds no candidate, but ready, unclaimed tasks of the tier are hidden only by the busy rule, it
     prints one line per such lane, `lane <lane> busy: <task id> in flight on <branch>`, and exits 3. The task named
     is one in flight there (the first by lane, id and branch). An empty queue still prints `no ready task` and
     exits 1. `ready`'s output is unchanged apart from item 1.
  3. A `lane ... busy` result is not an empty queue. The worker moves on to the next lane, and it does not launch
     the planner or the gap-triager for that lane (work-loop step 1).
- **Consequences:**
  - The `active` claims in `branch_claims` now carry the branch name, so the message can name it.
  - Routine prompts that read `no ready task` keep working: they see the new line where they would have planned
    for nothing. `make next-task` fails in both cases, as before.
  - Decision 0022's slot counts (LANE_SLOTS) are unchanged for non-P0 work.
