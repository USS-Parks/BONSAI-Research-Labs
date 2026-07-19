"""Validate the frozen BONSAI terminology and numeric-field registry."""

from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "schemas" / "registry" / "terminology-v1.json"


def validate_registry(registry: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    canonical: dict[str, str] = {}
    aliases: dict[str, str] = {}
    for term in registry.get("terms", []):
        name = term.get("name", "")
        definition = term.get("definition", "")
        if not name or name in canonical:
            errors.append(f"duplicate or empty canonical term: {name!r}")
        else:
            canonical[name] = definition
        if not definition:
            errors.append(f"{name!r}: ambiguous empty definition")
        for alias in term.get("aliases", []):
            if not alias or alias in aliases:
                errors.append(f"duplicate or empty alias: {alias!r}")
            else:
                aliases[alias] = name

    collisions = sorted(set(canonical) & set(aliases))
    if collisions:
        errors.append(f"aliases collide with canonical terms: {collisions}")

    identifiers = [identifier.get("name", "") for identifier in registry.get("identifiers", [])]
    if len(identifiers) != len(set(identifiers)) or any(not name.endswith("_id") for name in identifiers):
        errors.append("identifiers must be unique and end in _id")
    for identifier in registry.get("identifiers", []):
        if not identifier.get("representation") or not identifier.get("scope") or not identifier.get("stability"):
            errors.append(f"{identifier.get('name', '<unknown>')}: incomplete identifier definition")

    numeric_names: set[str] = set()
    for field in registry.get("numeric_fields", []):
        name = field.get("name", "")
        if not name or name in numeric_names:
            errors.append(f"duplicate or empty numeric field: {name!r}")
        numeric_names.add(name)
        for required in ("unit", "representation", "missingness"):
            if not field.get(required):
                errors.append(f"{name!r}: missing {required}")

    enum_values: dict[str, str] = {}
    for enum_name, values in registry.get("enums", {}).items():
        for value in values:
            if value in enum_values:
                errors.append(f"enum value {value!r} is ambiguous between {enum_values[value]!r} and {enum_name!r}")
            enum_values[value] = enum_name
    return errors


def self_test(registry: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    duplicate = copy.deepcopy(registry)
    duplicate["terms"].append(copy.deepcopy(duplicate["terms"][0]))
    if not any("duplicate" in error for error in validate_registry(duplicate)):
        errors.append("duplicate-term negative fixture was accepted")

    unitless = copy.deepcopy(registry)
    unitless["numeric_fields"][0]["unit"] = ""
    if not any("missing unit" in error for error in validate_registry(unitless)):
        errors.append("unitless-numeric negative fixture was accepted")
    return errors


def main() -> int:
    registry: dict[str, Any] = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    errors = validate_registry(registry) + self_test(registry)
    if errors:
        print("terminology registry check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(
        f"terminology registry check passed: {len(registry['terms'])} terms, "
        f"{len(registry['identifiers'])} identifiers, {len(registry['numeric_fields'])} numeric fields; "
        "negative fixtures rejected"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
