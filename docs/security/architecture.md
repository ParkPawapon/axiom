# Security Architecture

AxiomPHP is designed as a secure desktop utility for local PHP development workflows. The repository now includes project CRUD, PHP runtime selection, project PHP process supervision, service lifecycle adapters, managed database provisioning, backup/restore orchestration, security controls, and project-scoped Docker orchestration behind Rust application use cases and infrastructure ports.

## Trust Boundaries

- The React frontend is presentation and user intent only.
- The Tauri command layer is a narrow serialization boundary.
- Application use cases own orchestration.
- Domain modules stay pure and platform independent.
- Infrastructure adapters own external systems.
- Platform adapters isolate macOS, Windows, and optional Linux scheduler behavior.

## Command Execution Policy

OS-level execution must use structured command arguments through `CommandRunner`. Shell string concatenation, unsanitized arguments, and direct process execution from Tauri commands are not allowed.

Package-manager installation is constrained by these rules:

- The frontend can request a PHP catalog version only.
- Backend use cases validate the project ID and PHP version before execution.
- Infrastructure adapters choose package names and arguments.
- `CommandRunner` allows only the resolved Homebrew or Scoop executable path.
- Commands have fixed arguments, timeouts, and output limits.
- Failed installs do not update the project PHP selection.

Docker, database, certificate, service, scheduler, and backup operations follow the same rule: Tauri commands call use cases only, use cases call ports, and infrastructure adapters perform allowlisted external execution with bounded arguments, timeouts, output limits, and sanitized diagnostics.

## Validation Policy

Implementation must validate:

- project names
- filesystem paths
- runtime paths
- service names
- port numbers
- local domains
- environment variable keys
- certificate paths
- Docker image references
- backup destinations and replay files
- rollback annotations

## Secret Handling Policy

Secrets must not cross into frontend logs or command error payloads. Credentials must use platform storage adapters such as macOS Keychain or Windows Credential Manager.

## Auditability

Privileged operations should produce audit entries with safe metadata only. Audit logs must not contain tokens, passwords, private keys, or full environment dumps.
