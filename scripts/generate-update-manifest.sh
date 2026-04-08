#!/bin/bash

set -euo pipefail

VERSION="${1:?need release version}"
IS_PRERELEASE="${2:-false}"
EXISTING_MANIFEST="${3:-}"

python3 - "$VERSION" "$IS_PRERELEASE" "$EXISTING_MANIFEST" <<'PY'
import json
import pathlib
import re
import sys
from datetime import datetime, timezone

version = sys.argv[1]
is_prerelease = sys.argv[2].lower() == "true"
existing_path = sys.argv[3]

repo_base = f"https://github.com/yelog/rust-redis-desktop/releases/download/v{version}"
channel_name = "beta" if is_prerelease else "stable"
published_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def load_existing_manifest(path_str: str) -> dict:
    if not path_str:
        return {"channels": {}}

    path = pathlib.Path(path_str)
    if not path.exists():
        return {"channels": {}}

    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {"channels": {}}


def release_notes_for(version_str: str) -> str:
    changelog = pathlib.Path("CHANGELOG.md")
    if not changelog.exists():
        return f"Version {version_str} release"

    content = changelog.read_text(encoding="utf-8")
    pattern = rf"## \[{re.escape(version_str)}\].*?\n(.*?)(?=\n## \[|\Z)"
    match = re.search(pattern, content, re.DOTALL)
    if not match:
        return f"Version {version_str} release"

    lines = []
    for raw_line in match.group(1).splitlines():
        line = raw_line.strip()
        if not line:
            continue
        lines.append(line)

    return "\n".join(lines) if lines else f"Version {version_str} release"


manifest = load_existing_manifest(existing_path)
channels = manifest.get("channels")
if not isinstance(channels, dict):
    channels = {}

channels[channel_name] = {
    "version": version,
    "published_at": published_at,
    "release_notes": release_notes_for(version),
    "platforms": {
        "macos-x86_64": {
            "url": f"{repo_base}/rust-redis-desktop-x86_64.dmg",
            "asset_name": "rust-redis-desktop-x86_64.dmg",
        },
        "macos-aarch64": {
            "url": f"{repo_base}/rust-redis-desktop-aarch64.dmg",
            "asset_name": "rust-redis-desktop-aarch64.dmg",
        },
        "windows-x86_64": {
            "url": f"{repo_base}/rust-redis-desktop-x86_64-setup.exe",
            "asset_name": "rust-redis-desktop-x86_64-setup.exe",
        },
        "linux-x86_64": {
            "url": f"{repo_base}/rust-redis-desktop-x86_64.AppImage",
            "asset_name": "rust-redis-desktop-x86_64.AppImage",
        },
    },
}

output = {"channels": channels}
print(json.dumps(output, indent=2, ensure_ascii=False))
PY
