"""Golden slicer and timeline cache parts shared with the Rust copy of pivot_slicers.py (T-4188).

The eval adapter writes a spec's slicer and timeline cache parts in Rust
(crates/rustsheet-eval/src/pivot_definition/slicers.rs, which mirrors pivot_slicers.py) so the engine applies the
selections Excel opened. Both must write the same caches: this test pins the Python builder's pivot slicer and
timeline cache parts (a table slicer's drives no pivot and is left out) of the slicer and timeline probes to the
golden files in tests/slicer_parts_golden/, and the Rust module's tests pin its parts to the same files, in the
canonical form of test_pivot_parts_golden.py.

`RUSTSHEET_UPDATE_GOLDEN=1 uv run pytest tests/test_slicer_parts_golden.py` rewrites them from this builder.
"""

import json
import os
import re
from pathlib import Path

import pytest
from spec_parts import spec_parts
from test_pivot_parts_golden import CASES, canonical

GOLDEN = Path(__file__).resolve().parent / "slicer_parts_golden"
STEMS = ["t4767-pivot-slicers", "t4767-pivot-timelines"]
_CACHE = re.compile(r"xl/(slicerCaches/slicerCache|timelineCaches/timelineCache)(\d+)\.xml")


def golden_text(stem: str) -> str:
    spec = json.loads((CASES / f"{stem}.case.json").read_text(encoding="utf-8"))
    parts = spec_parts(spec)
    caches = sorted(
        (int(m[2]), xml)
        for name, xml in parts.items()
        if (m := _CACHE.fullmatch(name)) and "<pivotTables>" in xml
    )
    return "".join(f"== cache\n{canonical(xml.split('?>', 1)[-1])}\n" for _, xml in caches)


@pytest.mark.parametrize("stem", STEMS)
def test_the_builder_writes_the_golden_slicer_parts(stem: str) -> None:
    path = GOLDEN / f"{stem}.txt"
    text = golden_text(stem)
    if os.environ.get("RUSTSHEET_UPDATE_GOLDEN"):
        path.write_text(text, encoding="utf-8")
    assert path.read_text(encoding="utf-8") == text


def test_every_golden_slicer_file_is_listed() -> None:
    assert sorted(p.name for p in GOLDEN.glob("*.txt")) == sorted(f"{s}.txt" for s in STEMS)
