"""The task id format, as one definition (decision 0360).

Ids this repository allocates are five digits from T-90000 up. Older ids, and another repository's,
are four digits below them. Every pattern that reads a task id builds on `TASK_ID`, so none can read
the four-digit prefix of a five-digit id as an id of its own. Standard library only.
"""

from __future__ import annotations

TASK_ID = r"T-\d{4,5}"
