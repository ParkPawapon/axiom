# Testing And Release Automation

## Scope

This document defines the automated testing and release boundary for AxiomPHP.
The goal is to verify security-sensitive desktop capabilities before packaging
and to keep release signing, notarization, and artifact verification explicit.

## Automated E2E Coverage

The Rust integration suites cover the capabilities that previously had only
manual verification:

- `backup_restore_e2e` verifies managed backup artifact restore, cross-machine
  artifact trust enrollment, remote cloud destination copy receipts, and AWS KMS
  envelope restore through an allowlisted fake provider CLI.
- `docker_compose_generation_e2e` verifies all project Docker service profiles,
  volumes, reverse proxy generation, digest-pinned images, and resource limits.
- `docker_orchestration_e2e` verifies diagnostics, image digest resolution,
  Compose generation, volume lifecycle, start/restart/stop orchestration, and
  sanitized Docker logs through an allowlisted fake Docker CLI.
- `docker_orchestration_integration` remains opt-in for real Docker runtime
  execution because it starts containers and requires a digest-pinned image.

## CI Matrix

`.github/workflows/ci.yml` runs frontend, Rust, lint, and test gates on:

- `macos-14`
- `windows-latest`

`.github/workflows/e2e.yml` runs capability-oriented E2E suites across macOS
and Windows. Linux is used only for the optional real Docker smoke test because
GitHub-hosted macOS and Windows runners do not provide a production-equivalent
Docker Desktop runtime by default.

## Release Packaging

`.github/workflows/release.yml` packages macOS and Windows builds from tags or
manual dispatch. Tag builds require signed release mode.

Release scripts:

- `scripts/release/verify-signing-env.mjs` validates that signing and
  notarization inputs exist without printing secret values.
- `scripts/release/create-tauri-release-config.mjs` generates a temporary Tauri
  config with hardened macOS runtime settings and Windows signing metadata.
- `scripts/release/verify-release-artifacts.mjs` writes a SHA-256 manifest for
  generated bundles and runs platform signature checks when possible.

## Required Release Secrets

macOS signed releases require:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- App Store Connect notarization: `APPLE_API_ISSUER`, `APPLE_API_KEY`, and
  `APPLE_API_KEY_BASE64` or `APPLE_API_KEY_PATH`
- Or Apple ID notarization: `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID`

Windows signed releases require one of:

- PFX signing: `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`, and
  `WINDOWS_CERTIFICATE_THUMBPRINT`
- Azure Trusted Signing: `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`,
  `AZURE_TENANT_ID`, `AZURE_TRUSTED_SIGNING_ACCOUNT`,
  `AZURE_TRUSTED_SIGNING_CERTIFICATE_PROFILE`, and
  `AZURE_TRUSTED_SIGNING_ENDPOINT`
- A controlled `WINDOWS_SIGN_COMMAND`

## Release Safety Rules

- CI never stores signing credentials in repository files.
- Tag releases fail closed when required signing or notarization secrets are
  missing.
- Manual dry-run releases build unsigned packages with the same artifact
  verification path.
- Release manifests include SHA-256 hashes for generated artifacts.
- Real Docker runtime tests stay opt-in and require digest-pinned images.
