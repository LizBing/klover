#!/usr/bin/env python3
"""Incrementally build Java 8 fixtures, replacing stale outputs after success."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--javac", default="javac")
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    source_roots = [ROOT / name for name in ("core", "rust", "java", "test_data", "scripts", ".git", ".agents", ".codex")]
    if output == ROOT or output in ROOT.parents or any(
        output == source or output in source.parents or source in output.parents
        for source in source_roots
    ):
        parser.error("output must be a generated directory, not the source tree")
    javac = shutil.which(args.javac)
    if not javac:
        parser.error(f"javac not found: {args.javac}")
    version = subprocess.check_output([javac, "-version"], stderr=subprocess.STDOUT, text=True)
    sources = sorted((ROOT / "test_data/classes").rglob("*.java"))
    sources.append(ROOT / "java/java.base/java/lang/Object.java")
    inputs = {
        "javac": str(Path(javac).resolve()),
        "version": version.strip(),
        "flags": ["--release", "8"],
        "sources": {str(p.relative_to(ROOT)): digest(p) for p in sources},
        "builder": digest(Path(__file__)),
        "verifier": digest(ROOT / "scripts/verify-class-major.py"),
    }
    stamp = output / ".build-state.json"
    try:
        previous = json.loads(stamp.read_text())
    except (OSError, ValueError):
        previous = {}
    existing = {str(p.relative_to(output)): digest(p) for p in sorted(output.rglob("*.class"))}
    if previous.get("inputs") == inputs and existing and previous.get("outputs") == existing:
        print("Java test classes are up to date.")
        return

    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".test-classes-", dir=output.parent) as tmp:
        stage = Path(tmp) / "classes"
        stage.mkdir()
        subprocess.run([javac, "--release", "8", "-d", str(stage), *map(str, sources)], check=True)
        subprocess.run([sys.executable, str(ROOT / "scripts/verify-class-major.py"), "52", str(stage)], check=True)
        outputs = {str(p.relative_to(stage)): digest(p) for p in sorted(stage.rglob("*.class"))}
        (stage / stamp.name).write_text(json.dumps({"inputs": inputs, "outputs": outputs}, indent=2) + "\n")
        if output.exists():
            output.rename(Path(tmp) / "previous")
        stage.rename(output)
    print(f"Built {len(outputs)} Java test classes in {output}")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
