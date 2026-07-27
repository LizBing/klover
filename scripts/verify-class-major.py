#!/usr/bin/env python3
"""Verify .class files have the expected major_version (JVMS u2 at offset 6)."""

from __future__ import annotations

import struct
import sys
from pathlib import Path


def major_of(path: Path) -> int:
    data = path.read_bytes()
    if len(data) < 8:
        raise ValueError(f"{path}: too short")
    magic, _minor, major = struct.unpack_from(">IHH", data, 0)
    if magic != 0xCAFEBABE:
        raise ValueError(f"{path}: bad magic {magic:#x}")
    return major


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print(f"usage: {argv[0]} EXPECTED_MAJOR classfile...", file=sys.stderr)
        return 2

    expected = int(argv[1])
    failed = 0

    for raw in argv[2:]:
        path = Path(raw)
        if not path.is_file():
            print(f"FAIL {path}: missing (run: make classes)", file=sys.stderr)
            failed += 1
            continue
        try:
            major = major_of(path)
        except ValueError as e:
            print(f"FAIL {e}", file=sys.stderr)
            failed += 1
            continue
        if major != expected:
            print(f"FAIL {path}: major={major}, expected {expected}", file=sys.stderr)
            failed += 1
        else:
            print(f"OK   {path}: major={major}")

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
