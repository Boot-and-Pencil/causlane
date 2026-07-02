#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None


ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".devinfra/hopium/long-lanes.toml"
REPORT = ROOT / ".agent-state/parallel-dev/long-lanes-report.json"
VALID_DISPOSITIONS = {
    "required",
    "not_applicable_until_executable_targets",
    "declared_pending_execution",
    "manual_only_legacy",
}
REQUIRED_FIELDS = {"id", "owner", "disposition", "reason", "evidence"}


def load_toml(path: Path) -> dict:
    if tomllib is not None:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    raise RuntimeError("Python 3.11+ tomllib is required for Hopium long-lane checks")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--format", choices=["human", "json"], default="human")
    args = parser.parse_args()
    cfg = load_toml(CONFIG)
    lanes = cfg.get("lanes", [])
    findings = []

    for lane in lanes:
        lane_id = lane.get("id", "<missing>")
        missing = sorted(REQUIRED_FIELDS - set(lane))
        if missing:
            findings.append({"kind": "missing_fields", "lane": lane_id, "missing": missing})
        disposition = lane.get("disposition")
        if disposition not in VALID_DISPOSITIONS:
            findings.append({"kind": "invalid_disposition", "lane": lane_id, "disposition": disposition})

    if not lanes:
        findings.append({"kind": "empty_registry", "message": "long-lane registry must declare at least one lane"})

    report = {
        "schema_version": 1,
        "status": "pass" if not findings else "fail",
        "lane_count": len(lanes),
        "lanes": lanes,
        "finding_count": len(findings),
        "findings": findings,
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"long-lanes: {report['status']} lanes={len(lanes)} findings={len(findings)}")
        for finding in findings:
            print(json.dumps(finding, sort_keys=True))
    return 0 if not findings else 1


if __name__ == "__main__":
    raise SystemExit(main())
