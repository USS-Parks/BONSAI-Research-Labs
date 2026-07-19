from bonsai_reference import __all__


def test_scaffold_imports_without_domain_behavior() -> None:
    assert __all__ == ()
