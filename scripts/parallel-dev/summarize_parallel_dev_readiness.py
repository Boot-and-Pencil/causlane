#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / ".agent-state/parallel-dev"
REPORT = STATE / "readiness-summary.json"
INPUTS = {
    "parallel_dev": "parallel-dev-report.json",
    "version_set": "version-set-report.json",
    "ownership": "ownership-report.json",
    "long_lanes": "long-lanes-report.json",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--format", choices=["human", "json"], default="human")
    args = parser.parse_args()
    findings = []
    inputs = {}
    for key, file_name in INPUTS.items():
        path = STATE / file_name
        if not path.exists():
            findings.append({"kind": "missing_report", "report": str(path.relative_to(ROOT))})
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        inputs[key] = {"status": data.get("status"), "path": str(path.relative_to(ROOT))}
        if data.get("status") != "pass":
            findings.append({"kind": "failed_report", "report": str(path.relative_to(ROOT)), "status": data.get("status")})
    summary = {
        "schema_version": 1,
        "status": "pass" if not findings else "fail",
        "inputs": inputs,
        "finding_count": len(findings),
        "findings": findings,
    }
    STATE.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if args.format == "json":
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(f"parallel-dev-readiness: {summary['status']} findings={len(findings)}")
        for finding in findings:
            print(json.dumps(finding, sort_keys=True))
    return 0 if not findings else 1


if __name__ == "__main__":
    raise SystemExit(main())
