# PUB5 v0.0.1 Tag Evidence

**Status:** signed `v0.0.1` tag created and pushed.

This hand-maintained evidence records the release tag for the first public
`0.0.1` pre-alpha workspace release. It is release evidence only; PUB6
post-publication stabilization remains a follow-up.

## Tag Scope

Reviewed source baseline:

```text
main_commit: 53d927b3fd47994b9e2be2486421c689d1d4d492
date: 2026-06-29
host: dispatcher
runner: local repository workspace
tag: v0.0.1
tag_message: Causlane 0.0.1
```

Pre-tag checks:

```text
git status --short --branch: clean, main synced with origin/main
GitHub CI run 28318987697: success
git tag --list 'v0.0.1': local signed tag present before push
git ls-remote --tags origin 'v0.0.1*': no remote tag before push
```

Local tag verification:

```bash
git tag -v v0.0.1
git rev-parse v0.0.1^{}
```

Result:

```text
signature: good
signing_key: 0AC6D616765B1C729A60479090576F9A767B2AEA
signer: Vitalii Lobanov <lobanov@bootandpencil.com>
tagger_time: 2026-06-29T00:57:48+04:00
target_commit: 53d927b3fd47994b9e2be2486421c689d1d4d492
```

## Push Result

Command:

```bash
git push origin refs/tags/v0.0.1
```

Result:

```text
push: success
remote_tag_object: 8f94a65eb9152c5422507455178da964d0376347
remote_tag_target: 53d927b3fd47994b9e2be2486421c689d1d4d492
verified_at: 2026-06-28T21:01:52Z
```

Remote verification:

```text
git ls-remote --tags origin 'v0.0.1*'
  8f94a65eb9152c5422507455178da964d0376347 refs/tags/v0.0.1
  53d927b3fd47994b9e2be2486421c689d1d4d492 refs/tags/v0.0.1^{}
```

## Next State

The publication state machine moves to:

```text
Tagged(v0.0.1)
```

The next runbook action is PUB6 post-publication stabilization.
