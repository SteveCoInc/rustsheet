# 0360 — Task ids this repository allocates start at T-90000

- **Status:** accepted (2026-11-10, owner request; recorded in the PR that changed `tasks.next_id`). Amends how
  `make new-task` and `make renumber` pick an id (decisions 0215, 0224) and widens the task id format from four
  digits to four or five. The id registry and its race handling are unchanged.
- **Context:**
  - The owner runs a second agent that files tasks in another git repository. That repository uses the same `T-NNNN`
    ids, so tasks from the two repositories collide.
  - `tasks.next_id` took the next id inside the hundred-block its milestone used most, for example T-51xx for M9.
    Both repositories allocate from the same blocks, so collisions are certain, not occasional. The id registry
    (decision 0215) only sees this repository's claims.
  - A floor inside four digits (T-9000) was considered first. It leaves only a thousand ids, about two weeks of
    filing, and a T-9xxx id still looks like the other repository's. The owner chose five digits instead.
- **Decision:**
  1. A task id is `T-` and four or five digits. The pattern has one definition, `ids.TASK_ID` (scripts/ids.py),
     which every validator and allocator pattern builds on. A pattern that reads four digits alone would take
     T-90001 for T-9000, so no task id pattern may hard-code the width. The progress report's citation check
     (tools/, which cannot import scripts/) spells the same `\d{4,5}`.
  2. A task id this repository allocates is the next id above every taken id at or above T-90000 (`tasks.ID_FLOOR`),
     whatever the milestone. Ids still only grow, and the registry and branch scan still decide what is taken. The
     allocator refuses past T-99999 rather than write an id the pattern rejects.
  3. Existing four-digit ids stay as they are. The one task this repository had filed since the collision began,
     T-5113, is renumbered to T-90000.
- **Consequences:**
  - New tasks here are T-90000, T-90001, and so on. The other repository keeps its four-digit ids.
  - A new script that reads task ids imports `ids.TASK_ID`.
  - Ids sort as numbers, not strings: as text, T-90000 sorts before T-9100.
