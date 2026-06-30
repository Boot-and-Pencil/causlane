#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path
import sys
import tomllib
from typing import Any, Iterable

SKIP_DIRS = {".git", ".agent-state", "target", "node_modules", ".venv", "venv", "vendor"}
DEPENDENCY_SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        data = tomllib.load(handle)
    if not isinstance(data, dict):
        return {}
    return data


def iter_cargo_manifests(root: Path) -> Iterable[Path]:
    for path in sorted(root.rglob("Cargo.toml")):
        if any(part in SKIP_DIRS for part in path.relative_to(root).parts):
            continue
        yield path


def iter_dependency_specs(table: dict[str, Any], context: str) -> Iterable[tuple[str, Any, str]]:
    for section in DEPENDENCY_SECTIONS:
        deps = table.get(section)
        if isinstance(deps, dict):
            for name, spec in deps.items():
                yield name, spec, f"{context}.{section}"

    workspace = table.get("workspace")
    if isinstance(workspace, dict):
        yield from iter_dependency_specs(workspace, f"{context}.workspace")

    targets = table.get("target")
    if isinstance(targets, dict):
        for target_name, target_table in targets.items():
            if isinstance(target_table, dict):
                yield from iter_dependency_specs(target_table, f"{context}.target.{target_name}")


def normalize_git_remote(remote: str) -> str:
    if remote.startswith("git@github.com:"):
        return "ssh://git@github.com/" + remote.removeprefix("git@github.com:")
    return remote


def check_manifest(root: Path, manifest: Path, policy: dict[str, Any]) -> list[str]:
    findings: list[str] = []
    data = load_toml(manifest)
    allowed_remotes = set(policy.get("allowed_remotes", []))
    allowed_external_remotes = set(policy.get("allowed_external_git_remotes", []))
    rel_manifest = manifest.relative_to(root)

    for dep_name, spec, context in iter_dependency_specs(data, str(rel_manifest)):
        if not isinstance(spec, dict):
            continue

        path_value = spec.get("path")
        if isinstance(path_value, str):
            resolved = (manifest.parent / path_value).resolve()
            try:
                resolved.relative_to(root.resolve())
            except ValueError:
                findings.append(
                    f"CROSS_REPO_PATH_DEP {rel_manifest}: {context}.{dep_name} path={path_value!r} resolves outside repo"
                )

        git_value = spec.get("git")
        if isinstance(git_value, str):
            remote = normalize_git_remote(git_value)
            if allowed_remotes and remote not in allowed_remotes and remote not in allowed_external_remotes:
                findings.append(
                    f"UNAPPROVED_GIT_REMOTE {rel_manifest}: {context}.{dep_name} git={git_value!r}"
                )
            if "branch" in spec:
                findings.append(
                    f"BRANCH_GIT_DEP {rel_manifest}: {context}.{dep_name} uses branch={spec['branch']!r}"
                )
            if "tag" not in spec and "rev" not in spec:
                findings.append(
                    f"UNPINNED_GIT_DEP {rel_manifest}: {context}.{dep_name} needs tag or rev"
                )

    return findings


def check_cargo_config(root: Path, policy: dict[str, Any]) -> list[str]:
    findings: list[str] = []
    required_key = policy.get("required_cargo_config_key")
    if required_key != "net.git-fetch-with-cli":
        return findings
    config_path = root / ".cargo" / "config.toml"
    if not config_path.exists():
        findings.append("MISSING_CARGO_CONFIG .cargo/config.toml is required for private git deps")
        return findings
    data = load_toml(config_path)
    net = data.get("net")
    if not isinstance(net, dict) or net.get("git-fetch-with-cli") is not True:
        findings.append("CARGO_CONFIG_GIT_FETCH .cargo/config.toml must set net.git-fetch-with-cli = true")
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Check Hopium private git dependency policy.")
    parser.add_argument("--root", default=".")
    parser.add_argument("--policy", default=".devinfra/policy/private-dependency-policy.toml")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    policy_path = root / args.policy
    if not policy_path.exists():
        print(f"PRIVATE-DEPS: missing policy {policy_path.relative_to(root)}", file=sys.stderr)
        return 1

    policy = load_toml(policy_path)
    findings: list[str] = []
    findings.extend(check_cargo_config(root, policy))
    for manifest in iter_cargo_manifests(root):
        findings.extend(check_manifest(root, manifest, policy))

    if findings:
        print("PRIVATE-DEPS: failed")
        for finding in findings:
            print(f"- {finding}")
        return 1

    print("PRIVATE-DEPS: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
