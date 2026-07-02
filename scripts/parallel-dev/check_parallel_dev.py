#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fnmatch
import json
from pathlib import Path
import re
import sys

try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None


ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".devinfra/hopium/parallel-dev.toml"
REPORT = ROOT / ".agent-state/parallel-dev/parallel-dev-report.json"
SKIP_DIRS = {".git", ".agent-state", ".devinfra", "target", ".venv", "node_modules", "artifacts", "tmp"}
SCAN_SUFFIXES = {".rs", ".toml", ".md", ".yaml", ".yml", ".sh", ".py"}
SCAN_SKIP_FILES = {
    "scripts/verify-architecture.py",
    "scripts/parallel-dev/check_parallel_dev.py",
}


def load_toml(path: Path) -> dict:
    if tomllib is not None:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    raise RuntimeError("Python 3.11+ tomllib is required for Hopium parallel-dev checks")


def iter_files(root: Path):
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(root)
        if any(part in SKIP_DIRS for part in rel.parts):
            continue
        if str(rel) in SCAN_SKIP_FILES:
            continue
        yield rel, path


def cargo_dependency_lines(root: Path):
    for rel, path in iter_files(root):
        if rel.name != "Cargo.toml":
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            if "=" not in stripped or "git" not in stripped:
                continue
            yield str(rel), lineno, stripped


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scan-bridge-terms", action="store_true")
    parser.add_argument("--format", choices=["human", "json"], default="human")
    args = parser.parse_args()

    cfg = load_toml(CONFIG)
    repo = cfg.get("repo", {})
    forbidden = cfg.get("dependency_policy", {}).get("forbidden_dependency_fragments", [])
    bridge_terms = cfg.get("bridge_policy", {}).get("forbidden_terms", [])
    findings = []

    for file_name, line_no, line in cargo_dependency_lines(ROOT):
        for fragment in forbidden:
            if fragment and fragment in line:
                findings.append(
                    {
                        "kind": "forbidden_dependency",
                        "path": file_name,
                        "line": line_no,
                        "fragment": fragment,
                        "message": f"forbidden dependency fragment {fragment!r} appears in Cargo.toml",
                    }
                )

    if args.scan_bridge_terms:
        term_patterns = [(term, re.compile(re.escape(term))) for term in bridge_terms]
        bridge_exempt_paths = cfg.get("bridge_policy", {}).get("exempt_paths", [])
        for rel, path in iter_files(ROOT):
            if path.suffix not in SCAN_SUFFIXES:
                continue
            if any(fnmatch.fnmatch(str(rel), pattern) for pattern in bridge_exempt_paths):
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for term, pattern in term_patterns:
                match = pattern.search(text)
                if match:
                    line_no = text.count("\n", 0, match.start()) + 1
                    findings.append(
                        {
                            "kind": "forbidden_bridge_term",
                            "path": str(rel),
                            "line": line_no,
                            "fragment": term,
                            "message": f"compatibility bridge term {term!r} is forbidden",
                        }
                    )

    report = {
        "schema_version": 1,
        "repo": repo.get("id", "unknown"),
        "role": repo.get("role", "unknown"),
        "status": "pass" if not findings else "fail",
        "finding_count": len(findings),
        "findings": findings,
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"parallel-dev: {report['status']} repo={report['repo']} findings={len(findings)}")
        for finding in findings:
            print(f"{finding['path']}:{finding['line']}: {finding['message']}")
    return 0 if not findings else 1


if __name__ == "__main__":
    raise SystemExit(main())
