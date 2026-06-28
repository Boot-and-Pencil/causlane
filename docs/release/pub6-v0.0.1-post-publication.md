# PUB6 v0.0.1 Post-publication Evidence

**Status:** PUB6 release-notes and GitHub Release stabilization complete.

This hand-maintained evidence records the repository-local post-publication
stabilization work for the `0.0.1` pre-alpha workspace release. It is release
evidence only; public follow-up issues were intentionally not created in this
slice.

## Release Scope

Reviewed source baseline:

```text
main_commit: 2ddff8abc00c28f39f904d12cbd423e31b58f146
date: 2026-06-29
host: dispatcher
runner: local repository workspace
tag: v0.0.1
```

Pre-PUB6 checks:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28335946380: success
git tag -v v0.0.1: good signature
tag_signing_key: 0AC6D616765B1C729A60479090576F9A767B2AEA
tag_target_commit: 53d927b3fd47994b9e2be2486421c689d1d4d492
remote_tag_object: 8f94a65eb9152c5422507455178da964d0376347
```

## Published Crates

| Crate | Version | Published At | Yanked | Checksum |
|---|---|---|---|---|
| `causlane-core` | `0.0.1` | 2026-06-26T19:51:01.389588Z | false | `3e703e438396eca36608bd69705354d77f73eab05ccdf46e907b28c2b954ae9c` |
| `causlane-formal` | `0.0.1` | 2026-06-26T22:03:55.863866Z | false | `9ff45311acec7e7e3fa23b05ffeb25e5edff0bc32c82c5dd2d9b2806b326bc1c` |
| `causlane-contracts` | `0.0.1` | 2026-06-27T17:08:10.324056Z | false | `4570be9472ea7c39da58ad3ad298998e27e9780f6c999155894eaa324aa83286` |
| `causlane-runtime` | `0.0.1` | 2026-06-27T23:59:47.881445Z | false | `2657b41237c090855df0ea6809fc5bf52569f9669081b9228c0776988e50a5be` |
| `causlane-replay` | `0.0.1` | 2026-06-28T01:13:20.764456Z | false | `4213e654052b36c9214fa3b7ea0620988615c1a3b93084a0520807556b62b6d8` |
| `causlane-codegen` | `0.0.1` | 2026-06-28T02:12:17.908438Z | false | `36c643419715d4d417c3a4d4377e887ec9f366fbc8574c2eb64a8a037de49e9e` |
| `causlane` | `0.0.1` | 2026-06-28T09:40:20.252158Z | false | `1441c014b854832be9de97de63506543df9ab7195dc73b4aa46d4679bba8b60d` |
| `causlane-cli` | `0.0.1` | 2026-06-28T10:10:10.592697Z | false | `2617fe7db29129a582e558c4b7a667cad1e3c34069de2f678db8f78132062265` |

## Downstream Smoke

Command pattern:

```bash
/workspace/repo/tools/cargo-dev new <tmp>/downstream
cd <tmp>/downstream
/workspace/repo/tools/cargo-dev add causlane@0.0.1
/workspace/repo/tools/cargo-dev check --locked
```

Result:

```text
tmp_project: /tmp/causlane-pub6-smoke.iU6cz6/downstream
cargo_add: resolved causlane v0.0.1 and causlane-core v0.0.1
cargo_check_locked: pass
exit_code: 0
```

The downstream smoke check used the checked-in `tools/cargo-dev` wrapper because
this devinfra host blocks direct Cargo invocation. The wrapper emitted
temporary-project `tools/devctl` lookup warnings before and after Cargo, but
the wrapped `cargo check --locked` completed successfully.

## GitHub Release

GitHub Release metadata:

```text
release_url: https://github.com/Boot-and-Pencil/causlane/releases/tag/v0.0.1
tag_name: v0.0.1
name: Causlane 0.0.1
published_at: 2026-06-28T21:25:16Z
is_draft: false
is_prerelease: true
target_commitish: main
```

The release is attached to the existing signed `v0.0.1` tag. The tag target was
verified separately as `53d927b3fd47994b9e2be2486421c689d1d4d492`.

## Release Notes

Repository release notes are recorded in `CHANGELOG.md` and `RELEASE.md`.

Known limitations for `0.0.1`:

- Causlane is not a workflow engine, scheduler or job queue.
- Public APIs and serialized shapes remain pre-alpha and may change before
  `0.1`.
- Formal and replay evidence is receipt-backed pre-alpha evidence, not a
  complete formal proof.
- Workspace-wide all-features Rust `1.85` compatibility is not claimed because
  the optional Restate runtime dependency chain declares higher MSRVs.
- `cargo-deny` duplicate-version warnings remain tracked as convergence
  backlog.

Security and provenance notes:

- The public repository baseline and package file-list review are recorded.
- The crates.io package sequence was executed one crate at a time in dependency
  order.
- Signed tag `v0.0.1` was created with maintainer key
  `0AC6D616765B1C729A60479090576F9A767B2AEA`.
- GitHub Release `v0.0.1` was created as a public pre-release.
- No public follow-up issues were created in this selected slice.

## Next State

The publication state moves to:

```text
GitHubReleasePublished(v0.0.1)
```

Public follow-up issues for known limitations remain optional/deferred. The next
product-roadmap action is M11.4 Examples.
