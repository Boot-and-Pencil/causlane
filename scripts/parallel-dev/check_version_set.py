#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re

try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None


ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".devinfra/hopium/version-set.toml"
REPORT = ROOT / ".agent-state/parallel-dev/version-set-report.json"


def load_toml(path: Path) -> dict:
    if tomllib is not None:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    raise RuntimeError("Python 3.11+ tomllib is required for Hopium version-set checks")


def cargo_specs():
    specs = []
    for path in ROOT.rglob("Cargo.toml"):
        rel = path.relative_to(ROOT)
        if any(part in {".git", ".agent-state", "target", "artifacts", "tmp"} for part in rel.parts):
            continue
        text = path.read_text(encoding="utf-8")
        for line_no, line in enumerate(text.splitlines(), start=1):
            if not re.search(r"\bgit\s*=", line) or "Boot-and-Pencil" not in line:
                continue
            specs.append((str(rel), line_no, line.strip()))
    return specs


def field(line: str, name: str) -> str | None:
    match = re.search(rf"\b{name}\s*=\s*[\"']([^\"']+)[\"']", line)
    return match.group(1) if match else None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--format", choices=["human", "json"], default="human")
    args = parser.parse_args()

    cfg = load_toml(CONFIG)
    pins = cfg.get("pins", [])
    specs = cargo_specs()
    findings = []
    checked = []

    for pin in pins:
        repo = pin["repo"]
        expected_tag = pin.get("tag")
        expected_rev = pin.get("rev")
        matched = [(path, line_no, line) for path, line_no, line in specs if repo in line]
        if not matched:
            checked.append({"repo": repo, "status": "not_used_here"})
            continue
        for path, line_no, line in matched:
            actual_tag = field(line, "tag")
            actual_rev = field(line, "rev")
            if expected_tag and actual_tag != expected_tag:
                findings.append(
                    {
                        "kind": "tag_mismatch",
                        "path": path,
                        "line": line_no,
                        "repo": repo,
                        "expected": expected_tag,
                        "actual": actual_tag,
                    }
                )
            if expected_rev and actual_rev != expected_rev:
                findings.append(
                    {
                        "kind": "rev_mismatch",
                        "path": path,
                        "line": line_no,
                        "repo": repo,
                        "expected": expected_rev,
                        "actual": actual_rev,
                    }
                )
            checked.append(
                {
                    "repo": repo,
                    "path": path,
                    "line": line_no,
                    "expected_tag": expected_tag,
                    "expected_rev": expected_rev,
                    "actual_tag": actual_tag,
                    "actual_rev": actual_rev,
                    "status": "checked",
                }
            )

    report = {
        "schema_version": 1,
        "version_set": cfg.get("version_set", "unknown"),
        "status": "pass" if not findings else "fail",
        "checked": checked,
        "finding_count": len(findings),
        "findings": findings,
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"version-set: {report['status']} version_set={report['version_set']} findings={len(findings)}")
        for finding in findings:
            print(
                f"{finding['path']}:{finding['line']}: {finding['repo']} expected "
                f"{finding.get('expected')} actual {finding.get('actual')}"
            )
    return 0 if not findings else 1


if __name__ == "__main__":
    raise SystemExit(main())
