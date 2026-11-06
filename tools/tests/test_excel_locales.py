"""The committed locale and calendar tables are exactly what `rustsheet-excel-locales` makes from the
probes, so a refilled probe or an edited generator cannot leave them stale."""

from __future__ import annotations

from rustsheet_tools.excel_locales import generate


def test_tables_are_generated_from_the_probes() -> None:
    for path, text in generate().items():
        assert path.read_text(encoding="utf-8") == text, f"{path.name} is stale: run rustsheet-excel-locales"
