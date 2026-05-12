# Dependabot Security Triage

## Alert 1: `glib` GHSA-wrw7-89jp-8q8g

Status: triaged as not used by supported production targets.

Severity: medium.

Manifest: `src-tauri/Cargo.lock`.

Affected path: Linux-only GTK/WebKit dependency chain pulled through Tauri:

```text
tauri -> gtk -> glib 0.18.x
```

Patched version: `glib 0.20.0`.

## Decision

AxiomPHP currently supports macOS and Windows desktop builds. The CI matrix enforces macOS and Windows Rust checks only, and no Linux desktop artifact is produced or distributed.

The vulnerable `glib` dependency is present in the lockfile because Cargo records target-specific dependencies for all supported Tauri targets. It is not part of the supported macOS or Windows runtime artifact.

Attempting to force `glib 0.20.0` is not safe because `gtk 0.18.x` requires `glib ^0.18`, and the current latest Tauri 2 release is still locked to that Linux GTK dependency chain.

## Required Follow-Up Before Linux Support

- Reopen this decision before enabling Linux builds.
- Upgrade Tauri/GTK once the upstream dependency chain supports `glib >= 0.20`.
- Add Linux CI only after the alert is resolved by upstream dependency upgrades.
- Do not ship Linux artifacts while this alert remains applicable to the Linux target.

## Operational Action

The GitHub Dependabot alert may be dismissed as `not_used` with a comment referencing this document while AxiomPHP remains a macOS/Windows-only desktop application.
