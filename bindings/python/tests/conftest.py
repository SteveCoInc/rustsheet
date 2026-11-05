from rustsheet import legacy
import pytest


@pytest.fixture
def um() -> legacy.UserModel:
    """An empty workbook with the user API"""
    return legacy.UserModel("workbook")


@pytest.fixture
def rm() -> legacy.Model:
    """An empty workbook with the raw API"""
    return legacy.create("workbook")
