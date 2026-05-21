# AxiomPHP

AxiomPHP is a production-oriented desktop application foundation for a modern local PHP development control center. The long-term goal is to provide a safer, cleaner replacement for XAMPP-style workflows while keeping services, projects, credentials, and operating-system actions behind explicit boundaries.

Current scope: production foundation plus safe configuration boundaries. Project PHP binary selection, project process controls, managed database provisioning, security controls, backup/restore orchestration, and project Docker orchestration are available behind Rust application use cases and infrastructure ports.

## Problem Statement

Traditional XAMPP-style tooling is convenient, but it couples Apache, PHP, and databases tightly, often relies on weak default security assumptions, and makes per-project isolation or reproducibility difficult. AxiomPHP is structured to evolve into a desktop utility that manages local PHP environments with cleaner boundaries, safer defaults, and project-based configuration.

## Tech Stack

- Rust and Tauri for the desktop backend and OS boundary
- TypeScript, React, and Vite for the frontend
- Tailwind CSS and MUI for UI primitives and theming
- Framer Motion prepared for future interactive animation
- Bun as the frontend package manager

## Architecture Overview

The project follows Clean Architecture on the Rust side and feature-based architecture on the React side.

- `src/app` bootstraps the React app, providers, routes, and global styles.
- `src/core` contains global API clients, configuration, design tokens, theme setup, frontend validation, and accessibility helpers.
- `src/shared` contains reusable presentation components, hooks, utility functions, and common types.
- `src/features` contains isolated feature modules for dashboard, projects, services, runtimes, databases, logs, and settings.
- `src-tauri/src/domain` contains pure domain models.
- `src-tauri/src/application` contains use-case boundaries.
- `src-tauri/src/ports` contains traits for external systems.
- `src-tauri/src/infrastructure` contains adapters for local persistence, safe passive service probes, and future external systems.
- `src-tauri/src/platform` contains macOS and Windows adapter placeholders.
- `src-tauri/src/commands` is reserved for thin Tauri command handlers that call application use cases.
- `src-tauri/src/shared` contains error, result, validation, and serialization foundations.

## Security Design Notes

Future implementation must keep Rust as the security boundary between UI intent and OS-level actions.

- Validate all user-provided paths before filesystem access.
- Validate ports, service names, project names, and environment variable keys.
- Avoid unsafe shell execution and shell string concatenation.
- Route all process execution through the command runner abstraction.
- Run package-manager installation only after explicit frontend confirmation.
- Keep package-manager command arguments backend-owned and version-catalog based.
- Keep backup encryption and signing keys in secure storage or explicit external key environment variables.
- Require digest-pinned Docker image references and registry metadata verification before starting project containers.
- Keep Docker CLI execution behind backend allowlists and sanitized log readers.
- Verify signed backup artifacts before managed restore when a signature sidecar exists.
- Run OS-level background backup scheduling through app-owned launch/task adapters only.
- Never expose secrets in frontend logs or serialized command errors.
- Never store passwords or tokens in plain text.
- Use platform-specific secure storage such as Keychain on macOS and Credential Manager on Windows.
- Keep Tauri command handlers thin and free of business logic.
- Use least-privilege Tauri capabilities and avoid enabling broad filesystem, shell, or process permissions.
- Prepare audit logging and permission checks before privileged operations are added.

## Cross-Platform Notes

The backend is structured for macOS and Windows first, with `platform/common` keeping shared abstractions separate from OS-specific adapters. Future Linux support can be added without changing the domain or application layers.

## Future Roadmap

- Project-based PHP environment configuration
- Project process switching and runtime supervision
- Optional cosign signature verification for Docker images
- Port conflict detection
- Environment profile management
- Logs viewer and service health status
- Permission and audit log workflows
- Signed release promotion dashboards and installer distribution policy

## Development Commands

Install dependencies:

```bash
bun install
```

Run the frontend only:

```bash
bun dev
```

Run the Tauri desktop app:

```bash
bun tauri dev
```

Build the frontend:

```bash
bun run build
```

Build the desktop app:

```bash
bun tauri build
```

Quality checks:

```bash
bun lint
bun typecheck
bun format
bun run test:e2e:backup
bun run test:e2e:docker
bun run release:verify-env
bun run release:config
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```

## Testing And Release

The repository includes macOS and Windows CI matrices for frontend, Rust, and
release-config validation. Capability E2E suites cover managed backup/restore,
cloud backup provider CLI boundaries, KMS envelope restore, cross-machine
backup trust enrollment, Docker Compose generation, and Docker orchestration
through allowlisted fake CLIs. Real Docker runtime execution remains opt-in
because it starts containers and requires digest-pinned images.

Release packaging is handled by `.github/workflows/release.yml`. Signed tag
releases fail closed when required macOS signing/notarization or Windows signing
inputs are missing. Dry-run manual releases can build unsigned packages while
still generating release manifests with SHA-256 hashes.

## Current Hardening Scope

The backup/restore layer supports managed artifacts, file picker restore, scheduled policies, OS scheduler installation, mounted destinations, native S3/R2/GCS uploads with CLI fallback, SFTP CLI destinations, integrity receipts, point-in-time snapshot restore, replay restore with SQL, MySQL binlog, PostgreSQL WAL-derived SQL, PostgreSQL physical WAL restore manifests, conservative rollback generation, explicit rollback annotations for destructive SQL, trust bundle and artifact-hash enrollment, passphrase-protected cross-machine recovery key export/import, retention, compression, encryption, HMAC signing, and AWS/GCP KMS-wrapped data-key envelopes.

The Docker layer supports per-project Compose plans, PHP/MySQL/PostgreSQL/Redis/Mailpit/reverse proxy/queue/search/object-storage/worker profiles, project volume lifecycle, digest-pinned image trust gates, registry metadata inspection, optional cosign verification, private registry auth through validated `DOCKER_AUTH_CONFIG`, user-facing image digest resolution, per-container resource limits, Docker Desktop diagnostics, and sanitized Docker log reads.

Still intentionally conservative: destructive migration rollback is not guessed without explicit annotations, real Docker runtime tests remain opt-in, and external KMS/provider credentials are supplied by environment or platform secure storage rather than frontend state.
