import importlib

import pytest
import rustsheet as ic
from rustsheet import legacy


def test_xlsx_roundtrip_user_model(um, tmp_path):
    um.set_user_input(0, 1, 1, "=6*7")
    um.update_range_style(0, 1, 1, 1, 1, "font.b", "true")
    file = str(tmp_path / "book.xlsx")
    um.save_to_xlsx(file)

    model = legacy.UserModel.from_xlsx(file)
    assert model.get_formatted_cell_value(0, 1, 1) == "42"
    assert model.get_cell_style(0, 1, 1)["font"]["b"] is True


def test_xlsx_roundtrip_raw_model(rm, tmp_path):
    rm.set_user_input(0, 1, 1, "=6*7")
    rm.evaluate()
    file = str(tmp_path / "book.xlsx")
    rm.save_to_xlsx(file)

    model = legacy.load_from_xlsx(file)
    model.evaluate()
    assert model.get_formatted_cell_value(0, 1, 1) == "42"


REMOVED_SYMBOLS = [
    "load_from_bytes",
    "load_from_icalc",
    "create_user_model_from_bytes",
    "create_user_model_from_icalc",
    "test_panic",
]
REMOVED_METHODS = [
    "from_bytes",
    "from_icalc",
    "apply_external_diffs",
    "flush_send_queue",
    "save_to_icalc",
    "to_bytes",
    "test_panic",
]


@pytest.mark.parametrize("name", REMOVED_SYMBOLS)
def test_removed_module_symbol(name):
    assert not hasattr(ic, name)
    assert name not in ic.__all__


FACADE_ALL = {
    "Workbook",
    "Transaction",
    "Formula",
    "XlError",
    "RustSheetError",
    "WorkbookError",
    "LoadError",
    "SaveError",
    "EditError",
    "RefError",
    "open",
    "__version__",
    "column_name_from_number",
    "column_number_from_name",
    "quote_name",
    "get_all_timezones",
    "get_supported_locales",
}
LEGACY_SYMBOLS = [
    "Model",
    "UserModel",
    "CellType",
    "create",
    "create_user_model",
    "load_from_xlsx",
    "create_user_model_from_xlsx",
]


def test_package_exports_only_the_facade_and_helpers():
    # SPEC-0026 decision 2: `import rustsheet` is the facade; the legacy classes moved out.
    assert set(ic.__all__) == FACADE_ALL
    assert len(ic.__all__) == len(FACADE_ALL)
    for name in ic.__all__:
        assert hasattr(ic, name)


@pytest.mark.parametrize("name", LEGACY_SYMBOLS)
def test_legacy_symbol_moved(name):
    assert not hasattr(ic, name)
    assert name in legacy.__all__
    assert hasattr(legacy, name)


def test_legacy_import_warns():
    # SPEC-0026 decision 7: the only warning step, naming the v1 freeze as the removal point.
    with pytest.warns(DeprecationWarning, match="removed at the API v1 freeze"):
        importlib.reload(legacy)


@pytest.mark.parametrize("name", REMOVED_METHODS)
def test_removed_class_method(name):
    assert not hasattr(legacy.UserModel, name)
    assert not hasattr(legacy.Model, name)


def test_workbook_name(um):
    assert um.get_name() == "workbook"
    um.set_name("renamed")
    assert um.get_name() == "renamed"


def test_locale_timezone_language(um):
    assert um.get_timezone() == "UTC"
    assert um.get_locale() == "en"
    assert um.get_language() == "en"
    um.set_timezone("Europe/Berlin")
    assert um.get_timezone() == "Europe/Berlin"


def test_fmt_settings(um):
    settings = um.get_fmt_settings()
    assert "currency" in settings
    assert "short_date" in settings


def test_version():
    assert ic.__version__
