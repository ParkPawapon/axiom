# Service Control Contribution Scope

## Goal

Prepare a production-safe contribution track for PHP runtime, database, reverse proxy, and supporting local service lifecycle controls.

## Boundaries

- Tauri commands remain thin and call application use cases only.
- Service lifecycle orchestration must use the `ServiceManager` port.
- Process execution must go through the command runner abstraction.
- Platform-specific service details stay under `src-tauri/src/platform`.
- Frontend controls must reflect backend capability flags from real service probes.

## Safety Gates

- Validate service names before dispatching any backend action.
- Validate ports before any service start request.
- Never build command strings from unchecked user input.
- Never expose secrets or raw command output to frontend logs.
- Add tests before expanding any service allowlist or adding privileged commands.

## Current Implementation

- Service inventory is served by Rust through thin Tauri commands.
- Service IDs are validated before use cases call the service manager port.
- Service-specific status adapters perform capability probes for PHP, MySQL, PostgreSQL, Docker, and reverse proxy candidates.
- Each probe resolves supported executables from `PATH` or fixed platform paths, builds an absolute-path `CommandPolicy` allowlist for that single binary, and runs only adapter-owned commands through `CommandRunner`.
- MySQL lifecycle uses allowlisted Homebrew launchd labels on macOS and known Windows service names on Windows.
- PostgreSQL lifecycle uses allowlisted Homebrew launchd labels on macOS and known Windows service names on Windows.
- Reverse proxy lifecycle uses allowlisted Caddy or Nginx launchd labels on macOS and known Windows service names on Windows.
- Docker lifecycle uses Docker CLI diagnostics, Docker Compose project visibility, optional `AXIOM_DOCKER_COMPOSE_FILE` up/down orchestration, Docker Desktop start/shutdown helpers on macOS, and Windows service control on Windows.
- Project Docker orchestration now lives behind a separate application use case and infrastructure adapter for per-project Compose generation, profiles, volumes, diagnostics, and sanitized logs.
- Lifecycle requests return blocked outcomes when a supported service label, plist, Windows service, Docker Desktop helper, or Docker CLI boundary is not configured.
- The Services screen calls the backend for inventory and status checks instead of rendering static placeholders.

## Out Of Scope

- Global PHP runtime lifecycle. PHP remains project-process scoped.
- Database credential creation and rotation.
- Database data directory provisioning.
- Registry trust metadata beyond required image digest pinning.
- Host file changes.
- Certificate generation.
