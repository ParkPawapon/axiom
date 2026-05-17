# Project Docker Orchestration Contribution Scope

## Goal

Provide project-based Docker Compose generation and runtime orchestration without letting the frontend build shell commands or operate on unrelated Docker projects.

## Current Implementation

- Project Docker commands are exposed through thin Tauri handlers under the Projects command boundary.
- Application use cases load persisted project metadata before any Docker action.
- The Docker infrastructure adapter generates Compose files under the app-owned data directory at `docker/projects/<project-id>/compose.yaml`.
- Compose uses the persisted document root as a bind mount after backend path validation.
- Compose uses a deterministic project name derived from the AxiomPHP project ID.
- Runtime actions call `docker compose up`, `down`, `ps`, and `port` through the allowlisted Docker CLI path.
- The frontend Projects screen shows status, Compose path, container ID, and local URL when Docker reports a published port.

## Testing Notes

Docker Desktop must be installed and running before runtime actions can succeed. If Docker Desktop data was removed, launch Docker Desktop first and let it recreate its engine state before testing AxiomPHP Docker actions.

Useful verification commands:

```bash
docker info
docker compose ls
```

## Boundaries

- Do not execute Docker through shell strings.
- Do not accept arbitrary Compose paths from the frontend for project runtime actions.
- Do not target Docker projects that do not use the AxiomPHP project-scoped name.
- Do not store generated Compose files inside user project directories.
- Do not expose Docker command output directly into frontend logs.

## Future Work

- Add project-specific database and reverse proxy services to generated Compose profiles.
- Add configurable Docker image trust policies and digest pinning.
- Add Docker volume lifecycle controls per project.
- Add Docker log streaming through a sanitized backend log reader.
- Add richer Docker diagnostics for missing Docker Desktop setup after engine reset.
