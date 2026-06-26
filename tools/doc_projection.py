"""Shared helpers for generated documentation projections."""

from pathlib import Path
import sys


def markdown_list(items):
    return ", ".join(items) if items else "none"


def write_text(path, content):
    output = Path(path)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8") as handle:
        handle.write(content)


def report_projection_drift(path, expected, source, regenerate_command, stderr=sys.stderr):
    try:
        with open(path, encoding="utf-8") as handle:
            committed = handle.read()
    except FileNotFoundError:
        stderr.write(f"{path} not found: regenerate with `{regenerate_command}`\n")
        return False
    if committed == expected:
        return True
    stderr.write(
        f"{path} drifted from {source}: regenerate with `{regenerate_command}`\n"
    )
    got_lines = committed.splitlines()
    want_lines = expected.splitlines()
    for line_no, (got, want) in enumerate(zip(got_lines, want_lines), start=1):
        if got != want:
            stderr.write(
                f"  first differing line {line_no}: committed={got!r} generated={want!r}\n"
            )
            break
    else:
        stderr.write(
            f"  line count differs: committed={len(got_lines)} generated={len(want_lines)}\n"
        )
    return False
