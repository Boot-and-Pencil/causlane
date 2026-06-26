#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


def run_text(argv: list[str], cwd: Path, default: str = "") -> str:
    try:
        result = subprocess.run(
            argv,
            cwd=cwd,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        return default
    return result.stdout.strip()


def repo_root() -> Path:
    start = Path.cwd()
    detected = run_text(["git", "rev-parse", "--show-toplevel"], start)
    if detected:
        return Path(detected)
    return start


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def ensure_state_dirs(root: Path) -> None:
    for relative in [
        ".agent-state",
        ".agent-state/locks",
        ".devinfra/run",
        ".devinfra/logs",
    ]:
        (root / relative).mkdir(parents=True, exist_ok=True)


def sha256_bytes(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    if not path.exists():
        return "sha256:"
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return "sha256:" + digest.hexdigest()


def load_toml(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    with path.open("rb") as handle:
        return tomllib.load(handle)


def atomic_write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        handle.write(text)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp_name, path)


def atomic_write_json(path: Path, data: Any) -> None:
    atomic_write_text(path, json.dumps(data, indent=2, sort_keys=True) + "\n")


def git_head(root: Path) -> str:
    return run_text(["git", "rev-parse", "--short", "HEAD"], root, default="unknown")


def dirty_tree_hash(root: Path) -> str:
    raw = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=root,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    ).stdout
    digest = hashlib.sha256()
    for entry in raw.split(b"\0"):
        if not entry:
            continue
        text = entry.decode("utf-8", errors="replace")
        path_text = text[3:]
        if " -> " in path_text:
            path_text = path_text.split(" -> ", 1)[1]
        if should_ignore_path(path_text):
            continue
        digest.update(text.encode("utf-8"))
        candidate = root / path_text
        if candidate.is_file():
            digest.update(path_text.encode("utf-8"))
            digest.update(candidate.read_bytes())
    return "sha256:" + digest.hexdigest()


def should_ignore_path(path_text: str) -> bool:
    parts = path_text.split("/")
    if "__pycache__" in parts or path_text.endswith(".pyc"):
        return True
    ignored_prefixes = (
        "target/",
        ".agent-state/",
        ".git/",
        "dist/",
        ".devinfra/logs/",
        ".devinfra/run/",
        ".claude/hook-runs/",
        ".codex/hook-runs/",
        "tmp/",
        "coverage/",
    )
    return path_text.startswith(ignored_prefixes) or path_text == ".bacon-locations"


def snapshot(root: Path | None = None) -> dict[str, str]:
    root = root or repo_root()
    return {
        "git_head": git_head(root),
        "dirty_tree_hash": dirty_tree_hash(root),
        "cargo_lock_hash": sha256_file(root / "Cargo.lock"),
        "cargo_config_hash": sha256_file(root / ".cargo/config.toml"),
        "rust_toolchain_hash": sha256_file(root / "rust-toolchain.toml"),
    }


def snapshot_stale_reasons(saved: object, current: dict[str, str]) -> list[str]:
    if not isinstance(saved, dict):
        return ["missing snapshot"]
    reasons = []
    for key, current_value in current.items():
        if saved.get(key) != current_value:
            reasons.append(f"{key} mismatch")
    for key in saved.keys():
        if key not in current:
            reasons.append(f"unexpected snapshot key {key}")
    return reasons


def rustc_vv(root: Path) -> str:
    return run_text(["rustc", "-Vv"], root, default="unknown")


def cargo_vv(root: Path) -> str:
    return run_text(["cargo", "-Vv"], root, default="unknown")


def cache_key(root: Path, cache_class: str) -> dict[str, Any]:
    classes = load_toml(root / ".devinfra/cache-classes.toml")
    class_data = classes.get(cache_class, {})
    if not class_data:
        raise SystemExit(f"unknown cache class: {cache_class}")
    inherited = class_data.get("inherits")
    if inherited:
        inherited_data = classes.get(str(inherited), {})
        merged = dict(inherited_data)
        merged.update(class_data)
        class_data = merged

    features = " ".join(class_data.get("features", []))
    rustflags = class_data.get("rustflags", "")
    rustdocflags = class_data.get("rustdocflags", "")
    rustc_wrapper = class_data.get("rustc_wrapper", "")
    rustc_workspace_wrapper = class_data.get("rustc_workspace_wrapper", "")
    cargo_target_dir = class_data.get("target_dir", "target")
    cargo_build_dir = class_data.get("llvm_cov_build_dir") or class_data.get("build_dir") or cargo_target_dir
    target_dir = class_data.get("llvm_cov_target_dir") or cargo_target_dir
    profile = "dev" if cache_class != "ci_sccache" else "ci"
    if cache_class == "coverage":
        profile = "coverage"
    if "heavy" in cache_class:
        profile = "custom-heavy"

    material = {
        "cache_class": cache_class,
        "rustc_vv": rustc_vv(root),
        "cargo_vv": cargo_vv(root),
        "target_dir": str((root / target_dir).resolve()),
        "cargo_target_dir": str((root / cargo_target_dir).resolve()),
        "cargo_build_dir": str((root / cargo_build_dir).resolve()),
        "target_triple_mode": class_data.get("target_triple_mode", "host-default"),
        "features": features,
        "profile": profile,
        "rustflags": rustflags,
        "rustdocflags": rustdocflags,
        "rustc_wrapper": rustc_wrapper,
        "rustc_workspace_wrapper": rustc_workspace_wrapper,
        "cargo_wrapper": class_data.get("cargo_wrapper", ""),
        "linker_config": class_data.get("linker_config", ""),
        "command_family": class_data.get("command_family", cache_class),
        "cargo_lock_hash": sha256_file(root / "Cargo.lock"),
        "cargo_config_hash": sha256_file(root / ".cargo/config.toml"),
        "rust_toolchain_hash": sha256_file(root / "rust-toolchain.toml"),
        "tool_versions_hash": sha256_file(root / ".devinfra/tool-versions.json"),
        "cache_class_config_hash": sha256_file(root / ".devinfra/cache-classes.toml"),
    }
    encoded = json.dumps(material, sort_keys=True).encode("utf-8")
    material["hash"] = sha256_bytes(encoded)
    material["rustc_vv_hash"] = sha256_bytes(material["rustc_vv"].encode("utf-8"))
    material["cargo_vv_hash"] = sha256_bytes(material["cargo_vv"].encode("utf-8"))
    return material


def normalize_cargo_json(stdout: str, root: Path, snap: dict[str, str], cache_class: str) -> list[dict[str, Any]]:
    diagnostics: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if row.get("reason") != "compiler-message":
            continue
        message = row.get("message") or {}
        spans = message.get("spans") or []
        primary = next((span for span in spans if span.get("is_primary")), spans[0] if spans else {})
        file_name = primary.get("file_name") or ""
        if file_name:
            try:
                file_name = str(Path(file_name).resolve().relative_to(root))
            except ValueError:
                pass
        code = message.get("code") or {}
        suggestions = []
        replacement = primary.get("suggested_replacement")
        if replacement is not None:
            suggestions.append({"message": "suggested replacement", "replacement": replacement})
        diagnostics.append(
            {
                "schema_version": 1,
                "snapshot": snap,
                "cache_class": cache_class,
                "severity": message.get("level", "unknown"),
                "code": code.get("code") if isinstance(code, dict) else None,
                "message": message.get("message", ""),
                "rendered": message.get("rendered"),
                "file": file_name,
                "range": {
                    "start_line": primary.get("line_start", 0),
                    "start_col": primary.get("column_start", 0),
                    "end_line": primary.get("line_end", primary.get("line_start", 0)),
                    "end_col": primary.get("column_end", primary.get("column_start", 0)),
                },
                "suggestions": suggestions,
            }
        )
    return diagnostics


def read_json(path: Path, default: Any = None) -> Any:
    if not path.exists():
        return default
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def tool_path(root: Path, name: str) -> str | None:
    local = root / "tools" / name
    if local.exists():
        return str(local)
    return shutil.which(name)


def print_json(data: Any) -> None:
    json.dump(data, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
