# Service Control Contribution Scope

## Goal

Prepare a production-safe contribution track for PHP runtime, database, reverse proxy, and supporting local service lifecycle controls.

## Boundaries

- Tauri commands remain thin and call application use cases only.
- Service lifecycle orchestration must use the `ServiceManager` port.
- Process execution must go through the command runner abstraction.
- Platform-specific service details stay under `src-tauri/src/platform`.
- Frontend controls must not imply active service management until backend use cases are implemented.

## Safety Gates

- Validate service names before dispatching any backend action.
- Validate ports before any service start request.
- Never build command strings from unchecked user input.
- Never expose secrets or raw command output to frontend logs.
- Add tests before enabling start, stop, or restart commands.

## Current Implementation

- Service inventory is served by Rust through thin Tauri commands.
- Service IDs are validated before use cases call the service manager port.
- Lifecycle requests return blocked outcomes while runtime drivers are not configured.
- No OS-level process, service, Docker, PHP, MySQL, PostgreSQL, hosts file, or certificate action is executed.
- The Services screen calls the backend for inventory and status checks instead of rendering static placeholders.

## Out Of Scope

- Real start, stop, or restart logic.
- Direct PHP, MySQL, PostgreSQL, Docker, or reverse proxy execution.
- Host file changes.
- Certificate generation.
