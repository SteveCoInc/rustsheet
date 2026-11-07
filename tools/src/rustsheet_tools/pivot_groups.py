"""Discrete (manual) and date-level groups of a defined pivot (T-4170; case builder v2, pivot_definition.py).

Two `fields` entry keys group a source field the way Excel saves the grouping. The shapes are read off
Excel-authored caches: ClosedXML's pivot-table/shared-group-field.xlsx and shared-nested-dategroup.xlsx
(Excel 15.0300) and corpus-bench 52379 and 59932 (Excel 16.0300).

    "Product": {"discrete": {"name": "Product2",
                             "groups": [{"name": "Group1", "items": ["A", "B"]}, ...]}}
    "Date": {"date_group": {"levels": ["months", "quarters", "years"], "start": 43466, "end": 44196}}

A discrete group adds a cache field (`databaseField="0"`) named `name`. Its `<fieldGroup base>` names the
source field and its `<discretePr>` gives each of the source field's shared items its group item. The
source field's own `<fieldGroup par>` names the added field. The group items are the source items no group
lists, each its own item in the source field's item order and written as its value, then the groups in the
order given, as Excel lists them. The added field's pivotField lists them in the order of their first source
item, as Excel does.

A date group groups the field in place by its first level (`<fieldGroup par base>`, `rangePr groupBy`).
Each further level adds a cache field, named as Excel names it (`Quarters`, `Years`, ...), with its own
`<fieldGroup base>`. The source field's `par` names the last added field, as in 52379, 59932 and
shared-nested-dategroup (48357 writes none). Levels run from the finest to the coarsest. The builder writes
Excel's group labels (`<1/1/2019`, `Jan` ... `Dec`, `>12/31/2020`). `start` and `end` are date serials
from 1900-03-01 on, written with `autoStart="0" autoEnd="0"` so Excel keeps them. The field holds date
serials (give its cells a date format: Excel groups a field by date only when its cells are dates) and is
cached as dates, as pivot.py's date groups are (T-4236).

The added fields come after the source fields, in source-field order. A definition names them in `rows`,
`columns`, `pages` and `fields`, whose entry for one takes only `attrs`, `hidden`, `collapsed`,
`subtotals`, `item_attrs`, `xml` and `item_order` (pivot_definition.py's, T-4164: its group items to list
first in its pivotField, by label, then the rest in the group's own order). Every field of a group lists its
`<items>`, even off the axes, as Excel writes them. Calculated fields and items do not combine with these groups, and an added field is not
a data field.

The eval adapter mirrors this module in crates/rustsheet-eval/src/pivot_definition/groups.rs: keep them in
step (the golden parts in tests/pivot_parts_golden/ pin both).
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from datetime import date, timedelta
from typing import TYPE_CHECKING, Any
from xml.sax.saxutils import quoteattr

from rustsheet_tools.pivot_xml import DATE_GROUPS, OLE_EPOCH, iso_date

if TYPE_CHECKING:
    from rustsheet_tools.pivot_definition import Value

# Most groups of a discrete group, and most items of one group (the Rust mirror's `MAX_LIST`).
MAX_LIST = 256
# The name Excel gives the field it adds for a date level.
LEVEL_NAMES = {by: by.capitalize() for by in DATE_GROUPS}
# The keys of a field entry that group the field (one at most; `group` is pivot_definition.py's).
GROUPINGS = ("group", "discrete", "date_group")
# The keys a field entry of an added field may hold.
ADDED_FIELD_KEYS = {"attrs", "hidden", "collapsed", "subtotals", "item_attrs", "xml", "item_order"}
_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")
_FIRST_SERIAL = 61  # 1900-03-01: earlier serials are on the 1900 calendar, which the labels do not follow
_LAST_SERIAL = 2958465  # 9999-12-31


@dataclass
class Discrete:
    """A discrete group: the added field's name, its group items, each source item's group item, and the
    added pivotField's item order."""

    name: str
    labels: list[Value]
    groups: list[int]
    order: list[int]


@dataclass
class DateGroup:
    """A date group: its levels (finest first) and its `startDate` and `endDate` serials."""

    levels: list[str]
    start: float
    end: float


def _day(serial: float) -> date:
    return (OLE_EPOCH + timedelta(days=int(serial))).date()


def date_labels(by: str, start: float, end: float) -> list[str]:
    """Excel's group items for level `by` from `start` to `end` (web.md "Grouped pivot fields import as
    their group items"): the item below `start`, the calendar part, and the item past `end`."""
    first, last = _day(start), _day(end)
    if by in ("seconds", "minutes"):
        inner = [f":{k:02d}" for k in range(60)]
    elif by == "hours":
        inner = [f"{(h % 12) or 12} {'AM' if h < 12 else 'PM'}" for h in range(24)]
    elif by == "days":  # a leap year's days: 29-Feb is always there
        days = (date(2000, 1, 1) + timedelta(days=k) for k in range(366))
        inner = [f"{d.day}-{_MONTHS[d.month - 1]}" for d in days]
    elif by == "months":
        inner = list(_MONTHS)
    elif by == "quarters":
        inner = [f"Qtr{q}" for q in range(1, 5)]
    else:
        inner = [str(y) for y in range(first.year, last.year + 1)]
    return [f"<{first.month}/{first.day}/{first.year}", *inner, f">{last.month}/{last.day}/{last.year}"]


def discrete(
    raw: Any,
    items: list[Value],
    key: Callable[[Value], tuple[str, str]],
    cached: Callable[[Any], Value],
    where: str,
) -> Discrete:
    """The discrete group `raw` over a field whose shared items are `items`."""

    def bad(why: str) -> ValueError:
        return ValueError(f"{where}: discrete {why}")

    if not isinstance(raw, dict) or set(raw) != {"name", "groups"}:
        raise bad("is {name, groups}")
    name, groups = raw["name"], raw["groups"]
    if not isinstance(name, str) or not name:
        raise bad("name is a non-empty text")
    if not isinstance(groups, list) or not 0 < len(groups) <= MAX_LIST:
        raise bad(f"groups is a list of 1 to {MAX_LIST} groups")
    index = {key(v): k for k, v in enumerate(items)}
    owner: dict[int, int] = {}  # source item -> its group
    names: list[Value] = []
    for g, group in enumerate(groups):
        if not isinstance(group, dict) or set(group) != {"name", "items"}:
            raise bad("group is {name, items}")
        label, members = group["name"], group["items"]
        if not isinstance(label, str) or not label:
            raise bad("group name is a non-empty text")
        if not isinstance(members, list) or not 0 < len(members) <= MAX_LIST:
            raise bad(f"group {label!r} items is a list of 1 to {MAX_LIST} items")
        for member in members:
            k = index.get(key(cached(member)))
            if k is None:
                raise bad(f"item {member!r} is not in the field")
            if k in owner:
                raise bad(f"item {member!r} is in two groups")
            owner[k] = g
        names.append(label)
    loose = [k for k in range(len(items)) if k not in owner]
    labels = [items[k] for k in loose] + names
    if len({key(v) for v in labels}) != len(labels):
        raise bad("group names must differ from each other and from the ungrouped items")
    at = {k: n for n, k in enumerate(loose)}
    of = [at[k] if k in at else len(loose) + owner[k] for k in range(len(items))]
    return Discrete(name, labels, of, list(dict.fromkeys(of)))


def discrete_xml(base: int, group: Discrete, item_xml: Callable[[Value], str]) -> str:
    """The added field's `<fieldGroup>`: its base, each source item's group, and the group items."""
    xs = "".join(f'<x v="{k}"/>' for k in group.groups)
    items = "".join(item_xml(v) for v in group.labels)
    return (
        f'<fieldGroup base="{base}"><discretePr count="{len(group.groups)}">{xs}</discretePr>'
        f'<groupItems count="{len(group.labels)}">{items}</groupItems></fieldGroup>'
    )


def date_group(raw: Any, where: str) -> DateGroup:
    """The date group `raw`."""

    def bad(why: str) -> ValueError:
        return ValueError(f"{where}: date_group {why}")

    if not isinstance(raw, dict) or set(raw) != {"levels", "start", "end"}:
        raise bad("is {levels, start, end}")
    levels = raw["levels"]
    if not isinstance(levels, list) or not levels or not all(lv in DATE_GROUPS for lv in levels):
        raise bad(f"levels lists some of {list(DATE_GROUPS)}")
    ranks = [DATE_GROUPS.index(lv) for lv in levels]
    if ranks != sorted(set(ranks)):
        raise bad("levels are distinct and run from the finest to the coarsest")
    bounds = (raw["start"], raw["end"])
    if any(isinstance(b, bool) or not isinstance(b, int | float) for b in bounds):
        raise bad("start and end are date serials")
    try:
        start, end = (float(b) for b in bounds)
    except OverflowError:  # an int past the double range (`10**400`)
        raise bad("start and end are date serials") from None
    if not _FIRST_SERIAL <= start < end <= _LAST_SERIAL:
        raise bad(f"needs {_FIRST_SERIAL} <= start < end <= {_LAST_SERIAL} (1900-03-01 to 9999-12-31)")
    return DateGroup([str(lv) for lv in levels], start, end)


def date_xml(by: str, group: DateGroup, base: int, par: int | None) -> str:
    """One level's `<fieldGroup>`: `par` on the field grouped in place, when it has further levels."""
    labels = date_labels(by, group.start, group.end)
    items = "".join(f"<s v={quoteattr(label)}/>" for label in labels)
    parent = "" if par is None else f' par="{par}"'
    return (
        f'<fieldGroup{parent} base="{base}"><rangePr autoStart="0" autoEnd="0" groupBy="{by}"'
        f' startDate="{iso_date(group.start)}" endDate="{iso_date(group.end)}"/>'
        f'<groupItems count="{len(labels)}">{items}</groupItems></fieldGroup>'
    )
