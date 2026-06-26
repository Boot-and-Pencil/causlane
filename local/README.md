# `local/` — machine-local, non-git working files

Files placed here are **deliberately not committed to git**. The `.gitignore` in
this directory ignores everything except itself and this `README.md`.

## Why this exists

Machine-specific settings, notes and scratch should not travel with the
repository via git. But putting them *outside* the repo would force every
backup/export tool to be pointed at an extra path. Keeping them here — inside the
repo tree, but git-ignored — means existing repo backups capture them with **no
reconfiguration**, while git stays clean.

## Rules

- Anything you drop here stays local (not pushed, not shared via git).
- Do **not** rely on this for secrets: contents are still plain files on disk and
  may be backed up. Use it for configuration snapshots and notes, not secrets.
- Do not move shared documentation here — it won't reach other clones.

## Typical contents

- Per-host CI/setup notes (e.g. `ci-dispatcher.local.md`).
- Local environment snapshots, scratch logs, throwaway scripts.
