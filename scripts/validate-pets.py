#!/usr/bin/env python3
"""Validate QPets runtime packages and print a machine-readable JSON report."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path, PurePosixPath
from typing import Any

from PIL import Image

ATLAS_SIZE = (1536, 2288)
CELL_SIZE = (192, 208)
FRAME_COUNTS = (7, 8, 8, 4, 5, 8, 6, 6, 6, 8, 8)
REQUIRED_DIALOGUE_KEYS = ("idle", "working", "success", "error")
MIN_NONTRANSPARENT_PIXELS = 256
PET_ID_RE = re.compile(
    r"^(?:qpet-[a-z0-9][a-z0-9._-]{1,59}|user\.[a-z0-9][a-z0-9._-]{1,59})$"
)


def load_json(path: Path, errors: list[str]) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(f"invalid JSON {path.name}: {exc}")
        return None


def safe_relative_path(value: Any) -> bool:
    if not isinstance(value, str) or not value or "\\" in value:
        return False
    path = PurePosixPath(value)
    return not path.is_absolute() and ".." not in path.parts and path.parts != (".",)


def alpha_extrema(image: Image.Image) -> tuple[int, int]:
    alpha = image.getchannel("A")
    return alpha.getextrema()


def transparent_rgb_residue_pixels(image: Image.Image) -> int:
    """Count non-zero RGB values hidden beneath fully transparent pixels."""
    return sum(
        1
        for red, green, blue, alpha in image.get_flattened_data()
        if alpha == 0 and (red != 0 or green != 0 or blue != 0)
    )


def validate_png(path: Path, errors: list[str], label: str) -> None:
    try:
        with Image.open(path) as image:
            image.load()
            if image.format != "PNG" or "A" not in image.getbands():
                errors.append(f"{label} must be an alpha PNG")
            elif alpha_extrema(image)[0] == 255:
                errors.append(f"{label} must contain transparent background pixels")
    except (OSError, ValueError) as exc:
        errors.append(f"cannot decode {label}: {exc}")


def validate_dialogues(path: Path, errors: list[str]) -> None:
    data = load_json(path, errors)
    if not isinstance(data, dict) or data.get("schemaVersion") != 1:
        errors.append("dialogues.json must have schemaVersion 1")
        return
    lines = data.get("lines")
    if not isinstance(lines, dict):
        errors.append("dialogues.json lines must be an object")
        return
    for key in REQUIRED_DIALOGUE_KEYS:
        values = lines.get(key)
        if not isinstance(values, list) or not values or not all(isinstance(v, str) and v.strip() for v in values):
            errors.append(f"dialogues.json lines.{key} must be a non-empty string array")


def validate_v2_sheet(path: Path, errors: list[str], details: dict[str, Any]) -> None:
    try:
        with Image.open(path) as original:
            original.load()
            if original.format != "WEBP" or "A" not in original.getbands():
                errors.append("spritesheet must be WEBP")
            image = original.convert("RGBA")
    except (OSError, ValueError) as exc:
        errors.append(f"cannot decode spritesheet: {exc}")
        return
    details["image"] = {"format": "WEBP", "size": list(image.size), "mode": "RGBA"}
    if image.size != ATLAS_SIZE:
        errors.append(f"spritesheet must be {ATLAS_SIZE[0]}x{ATLAS_SIZE[1]}")
        return
    residue_count = transparent_rgb_residue_pixels(image)
    details["transparentRgbResiduePixels"] = residue_count
    if residue_count:
        errors.append(f"spritesheet contains {residue_count} transparent RGB residue pixels")
    for row, used_count in enumerate(FRAME_COUNTS):
        for col in range(8):
            left, top = col * CELL_SIZE[0], row * CELL_SIZE[1]
            cell = image.crop((left, top, left + CELL_SIZE[0], top + CELL_SIZE[1]))
            alpha_min, alpha_max = alpha_extrema(cell)
            nontransparent_count = sum(alpha > 0 for alpha in cell.getchannel("A").get_flattened_data())
            if col < used_count and nontransparent_count < MIN_NONTRANSPARENT_PIXELS:
                errors.append(
                    f"required cell row {row}, column {col} has only "
                    f"{nontransparent_count} non-transparent pixels (minimum {MIN_NONTRANSPARENT_PIXELS})"
                )
            if col >= used_count and alpha_max != 0:
                errors.append(f"non-transparent unused cell row {row}, column {col}")
            if col >= used_count and alpha_min != 0:
                errors.append(f"partially opaque unused cell row {row}, column {col}")


def validate_package(root: Path, entry: dict[str, Any]) -> dict[str, Any]:
    package_id = entry.get("id") if isinstance(entry.get("id"), str) else "<invalid-index-id>"
    result: dict[str, Any] = {"id": package_id, "path": entry.get("packagePath"), "ok": False, "errors": [], "details": {}}
    errors: list[str] = result["errors"]
    package_path = entry.get("packagePath")
    if not safe_relative_path(package_path):
        errors.append("invalid packagePath")
        return result
    package_dir = root / package_path
    manifest_path = package_dir / "pet.json"
    manifest = load_json(manifest_path, errors)
    if not isinstance(manifest, dict):
        return result
    if manifest.get("schemaVersion") != 1:
        errors.append("pet.json schemaVersion must be 1")
    if manifest.get("id") != package_id:
        errors.append("pet.json id must match index id")
    if not isinstance(package_id, str) or not PET_ID_RE.fullmatch(package_id):
        errors.append("pet id must use qpet- (builtin) or user. (user) namespace and portable lowercase characters")
    elif not package_id.startswith("qpet-"):
        errors.append("builtin index entries must use the qpet- namespace")
    if not isinstance(manifest.get("displayName"), str) or not manifest["displayName"].strip():
        errors.append("pet.json displayName must be a non-empty string")
    for key, expected in (("thumbnailPath", "thumbnail.png"), ("dialoguesPath", "dialogues.json")):
        if manifest.get(key) != expected:
            errors.append(f"pet.json {key} must be {expected}")
    allowed = {"pet.json", "dialogues.json", "thumbnail.png"}
    render_type = manifest.get("renderType")
    if render_type == "sprite-atlas-v2":
        if manifest.get("spriteVersionNumber") != 2 or manifest.get("spritesheetPath") != "spritesheet.webp":
            errors.append("sprite-atlas-v2 requires spriteVersionNumber 2 and spritesheetPath spritesheet.webp")
        allowed.add("spritesheet.webp")
        validate_v2_sheet(package_dir / "spritesheet.webp", errors, result["details"])
    elif render_type == "static-image-v1":
        if manifest.get("characterPath") != "character.png":
            errors.append("static-image-v1 requires characterPath character.png")
        allowed.add("character.png")
        validate_png(package_dir / "character.png", errors, "character.png")
    else:
        errors.append("pet.json renderType must be sprite-atlas-v2 or static-image-v1")
    validate_png(package_dir / "thumbnail.png", errors, "thumbnail.png")
    validate_dialogues(package_dir / "dialogues.json", errors)
    try:
        children = list(package_dir.iterdir())
        actual = {child.name for child in children if child.is_file()}
        unexpected = sorted(
            child.name
            for child in children
            if child.is_symlink() or not child.is_file() or child.name not in allowed
        )
        missing = sorted(allowed - actual)
        if unexpected:
            errors.append("unexpected runtime files: " + ", ".join(unexpected))
        if missing:
            errors.append("missing runtime files: " + ", ".join(missing))
    except OSError as exc:
        errors.append(f"cannot inspect package directory: {exc}")
    result["ok"] = not errors
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("src-tauri/resources/pets"))
    args = parser.parse_args()
    root = args.root.resolve()
    index_errors: list[str] = []
    index = load_json(root / "index.json", index_errors)
    results: list[dict[str, Any]] = []
    if not isinstance(index, dict) or index.get("schemaVersion") != 1 or not isinstance(index.get("pets"), list):
        index_errors.append("index.json must have schemaVersion 1 and a pets array")
    else:
        seen_ids: set[str] = set()
        for entry in index["pets"]:
            if not isinstance(entry, dict):
                index_errors.append("every index pet must be an object")
                continue
            if entry.get("id") in seen_ids:
                index_errors.append(f"duplicate pet id: {entry.get('id')}")
            seen_ids.add(entry.get("id"))
            results.append(validate_package(root, entry))
        if index.get("defaultPetId") not in seen_ids:
            index_errors.append("defaultPetId must reference an indexed pet")
    report = {"schemaVersion": 1, "root": str(root), "ok": not index_errors and all(item["ok"] for item in results), "indexErrors": index_errors, "pets": results}
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
