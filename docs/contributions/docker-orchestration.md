# Docker Orchestration Contribution Scope

## Goal

Provide project-based Docker orchestration without weakening the desktop app security boundary.
Docker actions must remain backend-owned, allowlisted, and project-scoped.

## Current Implementation

- The Rust backend exposes Docker diagnostics, per-project Compose generation, start, stop, restart, volume lifecycle, status, and sanitized log read commands.
- Tauri command handlers are thin and call `application/docker` use cases.
- Use cases resolve registered projects through `ProjectRepository` before invoking the Docker orchestration port.
- `ProjectDockerOrchestrator` writes app-owned Compose files under the AxiomPHP data directory.
- Compose generation supports PHP, MySQL, PostgreSQL, Redis, Mailpit, and reverse proxy profiles.
- MySQL, PostgreSQL, and Redis use project-specific Docker named volumes with `dev.axiomphp.project-id` labels.
- Reverse proxy config is generated per project and proxies to the project PHP service.
- Docker CLI execution goes through `CommandRunner` with an absolute Docker binary allowlist.
- Docker logs are read through the Rust backend and sanitized before crossing into the frontend.
- Docker diagnostics report CLI, engine, Compose, and selected context readiness after Docker Desktop reset.
- Optional `AXIOM_DOCKER_CONTEXT` routes diagnostics and runtime commands through a validated Docker context name.
- Per-project resource limits are accepted for generated services as Compose `cpus` and `mem_limit` settings.

## Image Trust Policy

Project container start is blocked unless every selected image is configured with an immutable
`@sha256:` digest and registry metadata can be inspected through the backend Docker boundary.

Configure trusted image references with:

```bash
AXIOM_DOCKER_ALLOWED_REGISTRIES=docker.io,registry-1.docker.io
AXIOM_DOCKER_MAILPIT_IMAGE=axllent/mailpit:v1.22@sha256:<digest>
AXIOM_DOCKER_PHP_IMAGE=php:8.4-cli@sha256:<digest>
AXIOM_DOCKER_MYSQL_IMAGE=mysql:8.4@sha256:<digest>
AXIOM_DOCKER_POSTGRES_IMAGE=postgres:17@sha256:<digest>
AXIOM_DOCKER_REDIS_IMAGE=redis:7-alpine@sha256:<digest>
AXIOM_DOCKER_REVERSE_PROXY_IMAGE=nginx:1.27-alpine@sha256:<digest>
```

Tagged image references are shown in the UI for planning and can be resolved to digest-pinned
references with the image pinning workflow. Runtime start remains blocked until digest pinning,
allowed registry checks, and metadata verification all pass.

This branch verifies registry metadata with `docker buildx imagetools inspect`. It does not perform
cosign, Notary, or external transparency-log verification yet.

## Safety Gates

- Do not call Docker from frontend code.
- Do not execute through a shell.
- Do not pass arbitrary user-provided Docker arguments.
- Keep Docker project names derived from validated project IDs.
- Keep database container secrets in secure storage and app-owned env files with restricted file permissions.
- Confirm before removing project Docker volumes from the UI.
- Never expose env file contents or raw sensitive Docker output in frontend logs.
- Use digest-pinned image overrides from the frontend command payload or environment variables only.
- Validate CPU and memory limits before writing Compose files.

## Integration Test

The Docker integration test is opt-in because it starts a real container. It uses an isolated
temporary project directory and can target a named Docker context.

```bash
AXIOM_RUN_DOCKER_INTEGRATION_TEST=1 \
AXIOM_DOCKER_INTEGRATION_PHP_IMAGE=php:8.4-cli@sha256:<digest> \
AXIOM_DOCKER_CONTEXT=desktop-linux \
cargo test --manifest-path src-tauri/Cargo.toml --test docker_orchestration_integration
```

## Remaining Hardening

- Registry signature verification with cosign, Notary, or organization trust metadata.
- Registry authentication UX for private image metadata resolution.
- Broader template coverage for queues, search, object storage, and project-specific workers.
