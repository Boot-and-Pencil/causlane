#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fnmatch
import json
from pathlib import Path
import subprocess

try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None


ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".devinfra/hopium/ownership.toml"
REPORT = ROOT / ".agent-state/parallel-dev/ownership-report.json"


def load_toml(path: Path) -> dict:
    if tomllib is not None:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    raise RuntimeError("Python 3.11+ tomllib is required for Hopium ownership checks")


def changed_files(base: str) -> list[str]:
    proc = subprocess.run(
        ["git", "diff", "--name-only", f"{base}...HEAD"],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode == 0:
        return [line for line in proc.stdout.splitlines() if line]
    proc = subprocess.run(
        ["git", "diff", "--name-only", base, "HEAD"],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    return [line for line in proc.stdout.splitlines() if line]


def matches(patterns: list[str], path: str) -> bool:
    return any(fnmatch.fnmatch(path, pattern) or fnmatch.fnmatch("/" + path, pattern) for pattern in patterns)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default="origin/main")
    parser.add_argument("--change-class", default="code")
    parser.add_argument("--format", choices=["human", "json"], default="human")
    args = parser.parse_args()

    cfg = load_toml(CONFIG)
    owners = cfg.get("owners", [])
    default_owner = cfg.get("default_owner")
    files = changed_files(args.base)
    findings = []
    assignments = []

    for path in files:
        matched = [owner for owner in owners if matches(owner.get("paths", []), path)]
        if not matched and not default_owner:
            findings.append({"kind": "unowned_path", "path": path, "message": "changed path has no owner rule"})
            continue
        assignments.append({"path": path, "owners": [owner.get("owner") for owner in matched] or [default_owner]})

    report = {
        "schema_version": 1,
        "status": "pass" if not findings else "fail",
        "base": args.base,
        "change_class": args.change_class,
        "changed_count": len(files),
        "assignments": assignments,
        "finding_count": len(findings),
        "findings": findings,
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"touch-ownership: {report['status']} changed={len(files)} findings={len(findings)}")
        for finding in findings:
            print(f"{finding['path']}: {finding['message']}")
    return 0 if not findings else 1


if __name__ == "__main__":
    raise SystemExit(main())
