"""Tests for the locale-table generator (`rustsheet_tools.excel_locales`)."""

from __future__ import annotations

from rustsheet_tools.excel_locales import extended_hex

# The same pairs as `formatter::locale_tag::tests::EXTENDED_HEX` in the engine, so the
# generator's copy of Excel's extended-hex reading cannot drift from the engine's.
EXTENDED_HEX = [
    ("fr_FR", 0x0010_E00B),
    ("fr_FR-x", 0x10E0_0AF1),
    ("bgc", 0x0C0C),
    ("ceb", 0x0CEB),
    ("tok-001", 0x1E93_D001),
    ("rhg-Rohg-BD", 0xEC91_FDBD),
]


def test_extended_hex_matches_the_engine() -> None:
    for tag, value in EXTENDED_HEX:
        assert extended_hex(tag) == value, tag
