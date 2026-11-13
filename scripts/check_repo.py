"""Check the mod catalog and the paths contributors use to build each mod."""

import json
import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMIT = re.compile(r"[0-9a-f]{40}")
VERSION = re.compile(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?")
SLUG = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")
MOD_ID = re.compile(r"[a-z][a-z0-9_]*")


def read_toml(path):
    with path.open("rb") as file:
        return tomllib.load(file)


def check_mod(mod, catalog):
    metadata = read_toml(mod / "mod.toml")
    required = ("name", "id", "version", "license", "minecraft", "horizon", "horizon_mixins", "rust")
    for key in required:
        if not isinstance(metadata.get(key), str) or not metadata[key].strip():
            raise ValueError(f"missing or empty {key!r} in mod.toml")
    if not SLUG.fullmatch(mod.name) or not MOD_ID.fullmatch(metadata["id"]):
        raise ValueError("expected a kebab-case folder and a snake_case mod id")
    for key in ("version", "rust"):
        if not VERSION.fullmatch(metadata[key]):
            raise ValueError(f"{key} must be a version such as 1.2.3")
    for key in ("horizon", "horizon_mixins"):
        if not COMMIT.fullmatch(metadata[key]):
            raise ValueError(f"{key} must pin a full Git commit")
    if not (mod / "README.md").is_file() or f"](mods/{mod.name})" not in catalog:
        raise ValueError("add a mod README and link it from the root catalog")
    if not (mod / "LICENSE").is_file() and metadata["license"] != "MIT":
        raise ValueError("include the mod's license text")

    manifest = mod / "mixins" / metadata["id"] / "horizon-mixin.toml"
    if manifest.is_file():
        mixin = read_toml(manifest)
        expected = {"id": metadata["id"], "version": metadata["version"]}
        if any(mixin["mod"].get(key) != value for key, value in expected.items()):
            raise ValueError("mixin id and version must match mod.toml")
        if mixin.get("compatibility", {}).get("horizon_revision") != metadata["horizon"]:
            raise ValueError("mixin Horizon pin must match mod.toml")
    provenance = mod / "native-mods.json"
    if provenance.is_file() and expected_record(metadata) not in json.loads(provenance.read_text())["mods"]:
        raise ValueError("native-mods.json must include the current id and version")
    return metadata["id"]


def expected_record(metadata):
    return {key: metadata[key] for key in ("id", "version")}


def main():
    catalog = (ROOT / "README.md").read_text()
    ids = set()
    mods = sorted(path for path in (ROOT / "mods").iterdir() if path.is_dir())
    if not mods:
        raise SystemExit("The catalog must contain at least one mod.")
    for mod in mods:
        try:
            mod_id = check_mod(mod, catalog)
            if mod_id in ids:
                raise ValueError(f"duplicate mod id {mod_id}")
            ids.add(mod_id)
        except (KeyError, OSError, ValueError) as error:
            raise SystemExit(f"{mod.relative_to(ROOT)}: {error}") from error
    print(f"Validated {len(mods)} mod(s).")


if __name__ == "__main__":
    main()
