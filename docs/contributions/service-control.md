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

## Out Of Scope

- Real start, stop, or restart logic.
- Direct PHP, MySQL, PostgreSQL, Docker, or reverse proxy execution.
- Host file changes.
- Certificate generation.
