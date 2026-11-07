"""Case-spec defined names are written as workbook-scoped names (T-0043)."""

import io
import zipfile

import pytest

from rustsheet_tools.spec2xlsx import build_xlsx


def test_build_xlsx_writes_workbook_names() -> None:
    xlsx = build_xlsx(
        {
            "sheets": [{"name": "Sheet1", "cells": {"A1": 1}}],
            "names": {"isect": "=Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2"},
        }
    )
    workbook = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/workbook.xml").decode()
    assert '<definedName name="isect">Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2</definedName>' in workbook


def test_build_xlsx_without_names_has_none() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"A1": 1}}]})
    workbook = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/workbook.xml").decode()
    assert "definedName" not in workbook


def test_build_xlsx_prefixes_let_variables_and_future_functions_in_names() -> None:
    xlsx = build_xlsx(
        {
            "sheets": [{"name": "Sheet1", "cells": {"A1": 1}}],
            "names": {"letn": "=LET(x,5,x)", "fut": "=DAYS(2,1)"},
        }
    )
    workbook = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/workbook.xml").decode()
    assert '<definedName name="letn">_xlfn.LET(_xlpm.x,5,_xlpm.x)</definedName>' in workbook
    assert '<definedName name="fut">_xlfn.DAYS(2,1)</definedName>' in workbook


def test_build_xlsx_writes_names_xlsxwriter_refuses_by_their_characters() -> None:
    # XlsxWriter's `\w` check drops these with a warning; T-4446 probes them in Excel.
    xlsx = build_xlsx(
        {
            "sheets": [{"name": "Sheet1", "cells": {"A1": 1}}, {"name": "S2", "cells": {}}],
            "names": {"ⓐ": "=Sheet1!$A$1", "Ⓐx": "=1", "́a": "=2", "S2!नाम": "=Sheet1!$A$1", "ok": "=3"},
        }
    )
    workbook = zipfile.ZipFile(io.BytesIO(xlsx)).read("xl/workbook.xml").decode()
    assert '<definedName name="ⓐ">Sheet1!$A$1</definedName>' in workbook
    assert '<definedName name="Ⓐx">1</definedName>' in workbook
    assert '<definedName name="́a">2</definedName>' in workbook
    assert '<definedName name="नाम" localSheetId="1">Sheet1!$A$1</definedName>' in workbook
    assert '<definedName name="ok">3</definedName>' in workbook


def test_build_xlsx_refuses_a_hand_written_name_on_an_unknown_sheet() -> None:
    with pytest.raises(ValueError, match="no sheet"):
        build_xlsx({"sheets": [{"name": "Sheet1", "cells": {"A1": 1}}], "names": {"Nope!ⓐ": "=1"}})
