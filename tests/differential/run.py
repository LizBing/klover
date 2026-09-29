#!/usr/bin/env python3
"""Compare deterministic primitive static entries in HotSpot and Klover (stdlib only)."""
import argparse
import fnmatch
import json
import math
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys

from cases import CASES, FEATURES

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CLASSES = ROOT / "build/test-classes"
REPORT = ROOT / "build/differential/report.json"
RANGES = {"Z": (0, 1), "B": (-128, 127), "C": (0, 65535),
          "S": (-32768, 32767), "I": (-(2**31), 2**31 - 1),
          "J": (-(2**63), 2**63 - 1)}


def signature(desc):
    if not isinstance(desc, str) or not re.fullmatch(r"\(([ZBCSIJFD]*)\)([ZBCSIJFDV])", desc):
        raise ValueError(f"unsupported or invalid primitive descriptor: {desc!r}")
    return desc[1:desc.index(")")], desc[-1]


def scalar(kind, value, result=False):
    if not isinstance(value, str):
        raise ValueError("values must be strings (including integer arguments)")
    if kind in RANGES:
        if not re.fullmatch(r"0|-?[1-9][0-9]*", value):
            raise ValueError(f"noncanonical {kind} integer: {value!r}")
        low, high = RANGES[kind]
        if not low <= int(value) <= high:
            raise ValueError(f"{kind} integer out of range: {value}")
    elif kind in "FD":
        if result and value == "nan":
            return value
        width = 8 if kind == "F" else 16
        if not re.fullmatch(f"[0-9a-f]{{{width}}}", value):
            raise ValueError(f"{kind} requires {width} lowercase IEEE hex digits")
        bits = int(value, 16)
        exponent, fraction = ((0x7f800000, 0x007fffff) if kind == "F"
                              else (0x7ff0000000000000, 0x000fffffffffffff))
        if result and bits & exponent == exponent and bits & fraction:
            raise ValueError("NaN result must use canonical 'nan'")
    elif kind != "V" or value != "void" or not result:
        raise ValueError(f"invalid {kind} value: {value!r}")
    return value


def validate_cases(cases, features):
    names = set()
    for case in cases:
        if case["name"] in names:
            raise ValueError(f"duplicate case: {case['name']}")
        names.add(case["name"])
        unknown = set(case["requires"]) - features.keys()
        if unknown:
            raise ValueError(f"{case['name']}: unknown features: {sorted(unknown)}")
        params, _ = signature(case["descriptor"])
        if len(case["args"]) != len(params):
            raise ValueError(f"{case['name']}: argument count does not match descriptor")
        for kind, value in zip(params, case["args"]):
            scalar(kind, value)


def execute(command, timeout):
    record = {"command": command, "exit_status": None, "stdout": "", "stderr": "", "error": None}
    try:
        proc = subprocess.run(command, capture_output=True, text=True, encoding="utf-8",
                              errors="replace", timeout=timeout)
        record.update(exit_status=proc.returncode, stdout=proc.stdout, stderr=proc.stderr)
        if proc.returncode:
            record["error"] = f"process exited with status {proc.returncode}"
    except subprocess.TimeoutExpired as error:
        def decoded(value):
            return value.decode("utf-8", errors="replace") if isinstance(value, bytes) else value or ""
        record.update(stdout=decoded(error.stdout), stderr=decoded(error.stderr),
                      error=f"process timed out after {timeout:g}s")
    except OSError as error:
        record["error"] = str(error)
    return record


def normalize(record, return_type):
    record["result"] = None
    if record["error"]:
        return
    line = record["stdout"]
    if line.endswith("\n"):
        line = line[:-1]
    parts = line.split("\t")
    try:
        if len(parts) != 3 or parts[:2] != ["KLOVER_DIFF_V1", return_type]:
            raise ValueError("expected exactly one matching KLOVER_DIFF_V1 result line")
        record["result"] = {"type": return_type, "value": scalar(return_type, parts[2], result=True)}
    except ValueError as error:
        record["error"] = f"invalid result protocol: {error}"


def run_case(case, features, java, classes, klover, fuel, quantum, timeout):
    record = {"case": case, "status": "SKIP", "reason": "", "hotspot": None, "klover": None}
    disabled = [name for name in case["requires"] if not features[name]]
    if disabled:
        record["reason"] = "unsupported requirements: " + ", ".join(disabled)
        return record
    target = [case["class"], case["method"], case["descriptor"]]
    hot = execute([java, "-cp", str(classes), "HotSpotRunner", *target, *case["args"]], timeout)
    kl = execute([klover, str(classes), *target, str(fuel), str(quantum), *case["args"]], timeout)
    normalize(hot, case["descriptor"][-1])
    normalize(kl, case["descriptor"][-1])
    record.update(hotspot=hot, klover=kl, status="FAIL")
    if hot["error"] or kl["error"]:
        record["reason"] = "; ".join(f"{label}: {r['error']}" for label, r in
                                     [("HotSpot", hot), ("Klover", kl)] if r["error"])
    elif hot["result"] != kl["result"]:
        record["reason"] = "return values differ"
    else:
        record["status"] = "PASS"
    return record


def display(record):
    case = record["case"]
    print(f"{record['status']:4} {case['name']}" + (f": {record['reason']}" if record["reason"] else ""), flush=True)
    if record["status"] != "FAIL":
        return
    print(f"  entry: {case['class']}.{case['method']}{case['descriptor']} args={case['args']}")
    if "seed" in case:
        print(f"  seed: {case['seed']}")
    for label in ("hotspot", "klover"):
        process = record[label]
        print(f"  {label}: result={process['result']} exit={process['exit_status']}")
        print(f"    reproduce: {shlex.join(process['command'])}")
        for stream in ("stdout", "stderr"):
            if process[stream]:
                print(f"    {stream}: {process[stream].rstrip()}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=["debug", "release"], default="debug")
    parser.add_argument("--java", default="java")
    parser.add_argument("--filter", default="*", help="case-name glob; quote shell wildcards")
    parser.add_argument("--fuel", type=int, default=1_000_000)
    parser.add_argument("--quantum", type=int, default=1000)
    parser.add_argument("--timeout", type=float, default=10.0, help="seconds per process")
    args = parser.parse_args(argv)
    report = {"cases": [], "features": FEATURES, "summary": {}, "infrastructure_error": None}
    try:
        if not math.isfinite(args.timeout) or args.timeout <= 0 or not 0 < args.fuel < 2**64 or not 0 < args.quantum < 2**64:
            raise ValueError("timeout, fuel and quantum must be positive finite values (fuel/quantum: u64)")
        report["limits"] = {"fuel": args.fuel, "quantum": args.quantum, "timeout": args.timeout}
        validate_cases(CASES, FEATURES)
        cases = [case for case in CASES if fnmatch.fnmatchcase(case["name"], args.filter)]
        if not cases:
            raise ValueError(f"no cases match {args.filter!r}")
        java = args.java
        klover = str(ROOT / f"build/cargo/{args.profile}/examples/differential_invoke")
        if any(all(FEATURES[r] for r in case["requires"]) for case in cases):
            java, klover = shutil.which(java), shutil.which(klover)
            if not java or not klover:
                raise ValueError("java or differential_invoke not found; run make build-differential")
            java, klover = str(Path(java).resolve()), str(Path(klover).resolve())
            info = execute([java, "-cp", str(CLASSES), "HotSpotRunner", "--vm-info"], args.timeout)
            report["reference_vm"] = info
            name = info["stdout"].splitlines()[0] if info["stdout"] else ""
            if info["error"] or not ("HotSpot" in name or re.fullmatch(r"OpenJDK (64-Bit )?(Server|Client) VM", name)):
                raise ValueError(f"HotSpot probe failed (run make build-differential): {info}")
        for case in cases:
            record = run_case(case, FEATURES, java, CLASSES, klover, args.fuel, args.quantum, args.timeout)
            report["cases"].append(record)
            display(record)
    except (ValueError, OSError) as error:
        report["infrastructure_error"] = str(error)
        print(f"ERROR: {error}", file=sys.stderr)
    report["summary"] = {status: sum(r["status"] == status for r in report["cases"])
                         for status in ("PASS", "FAIL", "SKIP")}
    try:
        REPORT.parent.mkdir(parents=True, exist_ok=True)
        REPORT.write_text(json.dumps(report, indent=2) + "\n")
    except OSError as error:
        print(f"ERROR: cannot write report {REPORT}: {error}", file=sys.stderr)
        return 2
    print(" / ".join(f"{n} {status}" for status, n in report["summary"].items()))
    print(f"Report: {REPORT}")
    return 2 if report["infrastructure_error"] else int(report["summary"]["FAIL"] > 0)


if __name__ == "__main__":
    sys.exit(main())
