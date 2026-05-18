# Docker Orchestration Contribution Scope

## Goal

Provide project-based Docker orchestration without weakening the desktop app security boundary.
Docker actions must remain backend-owned, allowlisted, and project-scoped.

## Current Implementation

- The Rust backend exposes Docker diagnostics, per-project Compose generation, start, stop, restart, volume lifecycle, status, and sanitized log read commands.
- Tauri command handlers are thin and call `application/docker` use cases.
- Use cases resolve registered projects through `ProjectRepository` before invoking the Docker orchestration port.
- `ProjectDockerOrchestrator` writes app-owned Compose files under the AxiomPHP data directory.
- Compose generation supports PHP, MySQL, PostgreSQL, and reverse proxy profiles.
- MySQL and PostgreSQL use project-specific Docker named volumes with `dev.axiomphp.project-id` labels.
- Reverse proxy config is generated per project and proxies to the project PHP service.
- Docker CLI execution goes through `CommandRunner` with an absolute Docker binary allowlist.
- Docker logs are read through the Rust backend and sanitized before crossing into the frontend.
- Docker diagnostics report CLI, engine, Compose, and selected context readiness after Docker Desktop reset.

## Image Trust Policy

Project container start is blocked unless every selected image is configured with an immutable
`@sha256:` digest.

Configure trusted image references with:

```bash
AXIOM_DOCKER_PHP_IMAGE=php:8.4-cli@sha256:<digest>
AXIOM_DOCKER_MYSQL_IMAGE=mysql:8.4@sha256:<digest>
AXIOM_DOCKER_POSTGRES_IMAGE=postgres:17@sha256:<digest>
AXIOM_DOCKER_REVERSE_PROXY_IMAGE=nginx:1.27-alpine@sha256:<digest>
```

Tagged image references are shown in the UI for planning only. Runtime start remains blocked until
digest pinning is present.

## Safety Gates

- Do not call Docker from frontend code.
- Do not execute through a shell.
- Do not pass arbitrary user-provided Docker arguments.
- Keep Docker project names derived from validated project IDs.
- Keep database container secrets in secure storage and app-owned env files with restricted file permissions.
- Confirm before removing project Docker volumes from the UI.
- Never expose env file contents or raw sensitive Docker output in frontend logs.

## Remaining Hardening

- Native registry trust metadata verification beyond digest pinning.
- User-facing image pin management and digest resolution workflow.
- Compose profile templates for additional services.
- Per-project container resource limits.
- Integration tests against an isolated Docker context.
