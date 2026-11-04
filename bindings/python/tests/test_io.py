import pytest
import rustsheet as ic


def test_xlsx_roundtrip_user_model(um, tmp_path):
    um.set_user_input(0, 1, 1, "=6*7")
    um.update_range_style(0, 1, 1, 1, 1, "font.b", "true")
    file = str(tmp_path / "book.xlsx")
    um.save_to_xlsx(file)

    model = ic.UserModel.from_xlsx(file)
    assert model.get_formatted_cell_value(0, 1, 1) == "42"
    assert model.get_cell_style(0, 1, 1)["font"]["b"] is True


def test_xlsx_roundtrip_raw_model(rm, tmp_path):
    rm.set_user_input(0, 1, 1, "=6*7")
    rm.evaluate()
    file = str(tmp_path / "book.xlsx")
    rm.save_to_xlsx(file)

    model = ic.load_from_xlsx(file)
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


@pytest.mark.parametrize("name", REMOVED_METHODS)
def test_removed_class_method(name):
    assert not hasattr(ic.UserModel, name)
    assert not hasattr(ic.Model, name)


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
