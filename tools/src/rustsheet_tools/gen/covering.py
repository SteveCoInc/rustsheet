"""Seeded t-wise covering arrays with constraints (T-4132, the gen-pivot policy).

A covering array over factors (each a name with a list of levels) is a list of rows, each one level per
factor, such that every combination of `t` levels of every `t` factors of a group appears in some row. Each
group has its own strength, so one array can be pairwise over every factor and 3-wise over a few that interact.
A constraint (`allowed`) rules out combinations that cannot be built: it is called with a partial row and
answers False only when the levels it sees already conflict, so a combination it refuses is not required.

The construction is AETG's greedy one (Cohen et al., 1997): each new row starts from an uncovered combination
drawn from the seeded stream, fills the other factors one at a time in a random order with the level that covers the most uncovered
combinations (ties drawn from the seeded stream), and the best of `CANDIDATES` such rows is kept. It is not
minimal, but it is small (within a few rows of the largest group's product for pivot-sized factor sets) and,
for a seed, the same every run. `uncovered` recomputes coverage from the rows alone, so tests check the array
rather than trust the construction.
"""

from __future__ import annotations

import itertools
import random
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass

# Rows tried per added row; the one covering the most new combinations is kept.
CANDIDATES = 24
# A safety bound on rows (directive 5 spirit for tools): a constraint that refuses every completion of a
# combination it allowed alone would otherwise loop.
MAX_ROWS = 10_000

Row = dict[str, str]
Allowed = Callable[[Mapping[str, str]], bool]
# One required combination: (factor, level) pairs, sorted by factor.
Combo = tuple[tuple[str, str], ...]


@dataclass(frozen=True)
class Group:
    """Factors whose every `strength`-combination of levels must appear in some row."""

    factors: tuple[str, ...]
    strength: int


def required(factors: Mapping[str, Sequence[str]], groups: Sequence[Group], allowed: Allowed) -> set[Combo]:
    """Every combination the groups ask for that `allowed` does not refuse on its own."""
    out: set[Combo] = set()
    for group in groups:
        if group.strength > len(group.factors):
            raise ValueError(f"strength {group.strength} exceeds the {len(group.factors)} factors of a group")
        for names in itertools.combinations(sorted(group.factors), group.strength):
            for levels in itertools.product(*(factors[n] for n in names)):
                combo = tuple(zip(names, levels, strict=True))
                if allowed(dict(combo)):
                    out.add(combo)
    return out


def _combos_with(row: Mapping[str, str], factor: str, level: str, sizes: set[int]) -> list[Combo]:
    """The combinations of sizes `sizes` that `row` plus `factor=level` holds and that contain `factor`."""
    others = sorted(row)
    out: list[Combo] = []
    for size in sizes:
        for names in itertools.combinations(others, size - 1):
            out.append(tuple(sorted([*((n, row[n]) for n in names), (factor, level)])))
    return out


def build(
    factors: Mapping[str, Sequence[str]],
    groups: Sequence[Group],
    allowed: Allowed,
    rng: random.Random,
) -> list[Row]:
    """A covering array (module docstring) over `factors`; every row satisfies `allowed`."""
    for name, levels in factors.items():
        if not levels:
            raise ValueError(f"factor {name!r} has no level")
    todo = required(factors, groups, allowed)
    sizes = {g.strength for g in groups}
    names = list(factors)
    rows: list[Row] = []
    while todo:
        if len(rows) >= MAX_ROWS:
            raise ValueError(f"no covering array within {MAX_ROWS} rows: a constraint refuses completions")
        seed = rng.choice(sorted(todo))
        best: tuple[int, Row] | None = None
        for _ in range(CANDIDATES):
            row = _candidate(factors, names, dict(seed), todo=todo, sizes=sizes, allowed=allowed, rng=rng)
            if row is None:
                continue
            gain = len(covered_by(row, todo, sizes))
            if best is None or gain > best[0]:
                best = (gain, row)
        if best is None:
            # Every candidate dead-ended: the seed combination cannot be completed, so it is not coverable.
            todo.discard(seed)
            continue
        rows.append(best[1])
        todo -= covered_by(best[1], todo, sizes)
    return rows


def _candidate(
    factors: Mapping[str, Sequence[str]],
    names: list[str],
    row: Row,
    *,
    todo: set[Combo],
    sizes: set[int],
    allowed: Allowed,
    rng: random.Random,
) -> Row | None:
    rest = [n for n in names if n not in row]
    rng.shuffle(rest)
    for name in rest:
        scored: list[tuple[int, float, str]] = []
        for level in factors[name]:
            if not allowed({**row, name: level}):
                continue
            gain = sum(1 for c in _combos_with(row, name, level, sizes) if c in todo)
            scored.append((gain, rng.random(), level))
        if not scored:
            return None
        row[name] = max(scored)[2]
    return {n: row[n] for n in names}


def covered_by(row: Mapping[str, str], combos: set[Combo], sizes: set[int]) -> set[Combo]:
    """The combinations of `combos` that `row` holds."""
    out: set[Combo] = set()
    for size in sizes:
        for names in itertools.combinations(sorted(row), size):
            combo = tuple((n, row[n]) for n in names)
            if combo in combos:
                out.add(combo)
    return out


def uncovered(
    rows: Sequence[Mapping[str, str]],
    factors: Mapping[str, Sequence[str]],
    groups: Sequence[Group],
    allowed: Allowed,
) -> set[Combo]:
    """The required combinations no row holds (empty for a covering array)."""
    todo = required(factors, groups, allowed)
    sizes = {g.strength for g in groups}
    for row in rows:
        todo -= covered_by(row, todo, sizes)
    return todo
