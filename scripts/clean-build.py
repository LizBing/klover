#!/usr/bin/env python3
"""Remove the configured build roots and their compile database symlink."""

from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parent.parent


def main():
    paths = [Path(raw).resolve() for raw in sys.argv[1:]]
    # Validate all paths before deleting anything. Protect sources and Git data.
    tracked_roots = [ROOT / name for name in ("core", "rust", "java", "test_data", "scripts", ".git", ".agents", ".codex")]
    for path in paths:
        if path == ROOT or path in ROOT.parents or any(
            path == source or path in source.parents or source in path.parents
            for source in tracked_roots
        ):
            raise SystemExit(f"Refusing to clean source directory: {path}")
    link = ROOT / "compile_commands.json"
    if link.is_symlink():
        target = link.resolve()
        if any(target == path or path in target.parents for path in paths):
            link.unlink()
    for path in paths:
        if path.exists():
            shutil.rmtree(path)
            print(f"Removed {path}")


if __name__ == "__main__":
    main()
