# Security Policy

## Supported Versions

Causlane is currently pre-alpha. Security reports are accepted for the latest
published version and the `main` branch.

## Reporting A Vulnerability

Do not open a public issue for sensitive vulnerabilities.

Report privately to:

```text
Boot and Pencil <lobanov@bootandpencil.com>
```

Include:

- affected crate and version;
- impact;
- reproduction steps;
- whether the issue involves credentials or secrets;
- suggested mitigation if known.

## Secrets And Publication

Publishing to crates.io is permanent for a version. Yanking does not delete
uploaded source. If a secret is published, rotate it immediately.

Before publication, confirm the recorded PUB4 secret scan and PUB5 package
file-list review still match the selected upload baseline. If the baseline or
package contents changed, repeat the affected checks as described in
`PUBLISHING.md`.
