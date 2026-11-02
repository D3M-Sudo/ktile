#!/usr/bin/env python3
"""Validate that every resolved Cargo package declares license metadata.

This is an inventory completeness check, not a license-compatibility verdict.
Run from any directory inside the repository.
"""
from __future__ import annotations

import json
import subprocess
import sys


def main() -> int:
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as exc:
        print(f"error: cargo metadata failed: {exc}", file=sys.stderr)
        if isinstance(exc, subprocess.CalledProcessError) and exc.stderr:
            print(exc.stderr, file=sys.stderr)
        return 2

    metadata = json.loads(result.stdout)
    workspace_members = set(metadata["workspace_members"])
    packages = sorted(
        (package for package in metadata["packages"] if package["id"] not in workspace_members),
        key=lambda package: (package["name"].casefold(), package["version"]),
    )
    missing: list[str] = []
    print("Resolved third-party Cargo dependency license inventory")
    print("Package | Version | License expression / license file")
    for package in packages:
        license_expression = package.get("license")
        license_file = package.get("license_file")
        declaration = license_expression or (
            f"license-file: {license_file}" if license_file else ""
        )
        if not declaration:
            missing.append(f'{package["name"]} {package["version"]}')
            declaration = "MISSING LICENSE METADATA"
        print(f'{package["name"]} | {package["version"]} | {declaration}')

    if missing:
        print("\nerror: packages without declared license metadata:", file=sys.stderr)
        for package in missing:
            print(f"  - {package}", file=sys.stderr)
        return 1

    print(f"\nAll {len(packages)} resolved third-party packages declare license metadata.")
    print("This does not replace compatibility review or bundling required license texts.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
