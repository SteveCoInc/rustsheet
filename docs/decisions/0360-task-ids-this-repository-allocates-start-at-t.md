# 0360 — Task ids this repository allocates start at T-9000

- **Status:** accepted (2026-11-10, owner request; recorded in the PR that changed `tasks.next_id`). Amends how
  `make new-task` and `make renumber` pick an id (decisions 0215, 0224). The id registry and its race handling are
  unchanged.
- **Context:**
  - The owner runs a second agent that files tasks in another git repository. That repository uses the same `T-NNNN`
    ids, so tasks from the two repositories collide.
  - `tasks.next_id` took the next id inside the hundred-block its milestone used most, for example T-51xx for M9.
    Both repositories allocate from the same blocks, so collisions are certain, not occasional. The id registry
    (decision 0215) only sees this repository's claims.
- **Decision:**
  1. A task id this repository allocates is the next id above every taken id at or above T-9000 (`tasks.ID_FLOOR`).
     Ids still only grow, and the registry and branch scan still decide what is taken.
  2. Milestone blocks no longer choose the id. A task's milestone is its `milestone:` field and its directory.
  3. Existing ids below T-9000 stay as they are. The one task this repository had filed since the collision began,
     T-5113, is renumbered to T-9000.
  4. The range ends at T-9999, the last id `T-NNNN` can name. The allocator refuses an id past it rather than
     write a five-digit one that the validators would reject.
- **Consequences:**
  - New tasks here are T-9000, T-9001, and so on, whatever their milestone. The other repository keeps its ids
    below T-9000.
  - The thousand ids are about two weeks of filing at the current rate. Running out is a refusal, and it needs a new
    decision (a wider id pattern, or a new floor agreed with the other repository).
