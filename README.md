# AxiomPHP

AxiomPHP is a production-oriented desktop application foundation for a modern local PHP development control center. The long-term goal is to provide a safer, cleaner replacement for XAMPP-style workflows while keeping services, projects, credentials, and operating-system actions behind explicit boundaries.

Current scope: architecture scaffold only. Service management, Docker orchestration, PHP runtime control, MySQL, PostgreSQL, host file modification, certificate generation, and process execution are intentionally not implemented.

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
- `src-tauri/src/infrastructure` contains adapter placeholders.
- `src-tauri/src/platform` contains macOS and Windows adapter placeholders.
- `src-tauri/src/commands` is reserved for thin Tauri command handlers that call application use cases.
- `src-tauri/src/shared` contains error, result, validation, and serialization foundations.

## Security Design Notes

Future implementation must keep Rust as the security boundary between UI intent and OS-level actions.

- Validate all user-provided paths before filesystem access.
- Validate ports, service names, project names, and environment variable keys.
- Avoid unsafe shell execution and shell string concatenation.
- Route all future process execution through a command runner abstraction.
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
- PHP runtime discovery and validation
- MySQL and PostgreSQL service adapters
- Docker-based service orchestration
- Reverse proxy and local domain management
- Local HTTPS certificate workflow
- Port conflict detection
- Environment profile management
- Logs viewer and service health status
- Permission and audit log workflows

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
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```

## Not Implemented Yet

This scaffold does not start, stop, restart, detect, or manage any service. It does not execute OS commands, control Docker, connect to databases, write host files, generate SSL certificates, or manage PHP runtimes. Those capabilities should be implemented later through the existing ports, application use cases, infrastructure adapters, and platform-specific modules.
