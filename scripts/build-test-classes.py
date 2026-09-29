#!/usr/bin/env python3
"""Recompile the shared Java 8 fixtures into build/test-classes."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--javac", default="javac")
    args = parser.parse_args()
    output = ROOT / "build/test-classes"
    if output.parent.is_symlink() or output.is_symlink():
        raise ValueError("build and test-classes must be real directories")
    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)
    sources = sorted((ROOT / "test_data/classes").rglob("*.java"))
    sources.append(ROOT / "java/java.base/java/lang/Object.java")
    command = [args.javac, "--release", "8", "-d", str(output)]
    try:
        subprocess.run([*command, *map(str, sources)], check=True)
        # Compile the reference-only wrapper against the JDK's Object API.
        subprocess.run([*command, str(ROOT / "tests/differential/HotSpotRunner.java")], check=True)
    except (subprocess.CalledProcessError, OSError):
        shutil.rmtree(output)
        raise
    print(f"Compiled Java fixtures into {output}")


if __name__ == "__main__":
    try:
        main()
    except (subprocess.CalledProcessError, OSError, ValueError) as error:
        print(f"Java compilation failed: {error}", file=sys.stderr)
        sys.exit(1)
