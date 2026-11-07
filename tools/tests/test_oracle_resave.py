"""`rustsheet-oracle resave` (T-4890): each workbook is saved by the backend, a refusal recorded."""

from __future__ import annotations

from pathlib import Path

from rustsheet_tools.oracle.backends import OracleError, OracleInfo
from rustsheet_tools.oracle.cli import resave_all


class FakeBackend:
    name = "fake"

    def recalc(self, xlsx: bytes) -> tuple[bytes, OracleInfo]:
        if xlsx == b"refuse":
            raise OracleError("the workbook cannot be opened")
        return b"saved:" + xlsx, OracleInfo(kind="fake", application="fake")


def test_each_workbook_is_saved_and_a_refusal_is_recorded(tmp_path: Path) -> None:
    inputs = tmp_path / "in"
    inputs.mkdir()
    (inputs / "a.xlsx").write_bytes(b"one")
    (inputs / "b.xlsx").write_bytes(b"refuse")
    out = tmp_path / "out"
    results = resave_all(FakeBackend(), sorted(inputs.glob("*.xlsx")), out)
    assert results == {
        "a.xlsx": {"status": "saved"},
        "b.xlsx": {"status": "refused", "error": "the workbook cannot be opened"},
    }
    assert (out / "a.xlsx").read_bytes() == b"saved:one"
    assert not (out / "b.xlsx").exists()
