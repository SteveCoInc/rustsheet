"""What the two pivot builders share (pivot.py, pivot_definition.py; T-4122): the OOXML namespaces and how
a cached number is written."""

from __future__ import annotations

from datetime import datetime, timedelta

MAIN_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL_NS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PACKAGE_REL_NS = "http://schemas.openxmlformats.org/package/2006/relationships"


def number_text(value: float) -> str:
    """A cached number's text: a whole number below 1e15 without a fraction, else the shortest repr."""
    return str(int(value)) if float(value).is_integer() and abs(value) < 1e15 else repr(float(value))


# ST_GroupBy's date levels, finest first (T-4236; pivot_groups.py's levels, T-4170).
DATE_GROUPS = ("seconds", "minutes", "hours", "days", "months", "quarters", "years")
# Pivot caches write a date before March 1900 on the OLE epoch (1899-12-30 is 0), so the same
# epoch serves every serial the builders write (crates/rustsheet-xlsx `pivot_cache_date_serial`).
OLE_EPOCH = datetime(1899, 12, 30)


def iso_date(serial: float) -> str:
    """A date serial as a pivot cache writes it, to the whole second (truncated; the cache is
    refreshed on load, so it only has to parse)."""
    if not 0 <= serial < 2958466:
        msg = f"a pivot date serial must be 0 to 9999-12-31: {serial!r}"
        raise ValueError(msg)
    return (OLE_EPOCH + timedelta(seconds=int(serial * 86400))).strftime("%Y-%m-%dT%H:%M:%S")


def date_shared_items(numbers: list[float], *, blank: bool) -> str:
    """The `<sharedItems>` of a date field cached inline (no list): Excel's date flags and range."""
    return (
        '<sharedItems containsSemiMixedTypes="0" containsNonDate="0" containsDate="1" containsString="0"'
        + (' containsBlank="1"' if blank else "")
        + f' minDate="{iso_date(min(numbers))}" maxDate="{iso_date(max(numbers))}"/>'
    )
