# Security Architecture

AxiomPHP is designed as a secure desktop utility for local PHP development workflows. The repository now includes explicit PHP package-manager installation, project PHP process supervision, service lifecycle controls, project-scoped Docker Compose orchestration, managed database provisioning, security controls, and backup/restore orchestration behind Clean Architecture boundaries.

## Trust Boundaries

- The React frontend is presentation and user intent only.
- The Tauri command layer is a narrow serialization boundary.
- Application use cases own orchestration.
- Domain modules stay pure and platform independent.
- Infrastructure adapters own external systems.
- Platform adapters isolate macOS and Windows behavior.

## Command Execution Policy

OS-level execution must use structured command arguments through `CommandRunner`. Shell string concatenation, unsanitized arguments, and direct process execution from Tauri commands are not allowed.

Package-manager installation is constrained by these rules:

- The frontend can request a PHP catalog version only.
- Backend use cases validate the project ID and PHP version before execution.
- Infrastructure adapters choose package names and arguments.
- `CommandRunner` allows only the resolved Homebrew or Scoop executable path.
- Commands have fixed arguments, timeouts, and output limits.
- Failed installs do not update the project PHP selection.

Docker orchestration is constrained by these rules:

- The frontend sends only a persisted project ID and user intent.
- Backend use cases load the project and selected PHP version from repositories.
- Compose files are generated into an app-owned runtime directory.
- The project document root is validated as an existing absolute directory before being bound into Compose.
- Docker commands use a resolved Docker executable path and fixed backend-owned arguments.
- Each project uses a deterministic Compose project name, so start/stop/restart does not target unrelated projects.

## Validation Policy

Future implementation must validate:

- project names
- filesystem paths
- runtime paths
- service names
- port numbers
- local domains
- environment variable keys
- certificate paths

## Secret Handling Policy

Secrets must not cross into frontend logs or command error payloads. Credentials must use platform storage adapters such as macOS Keychain or Windows Credential Manager.

## Auditability

Privileged operations should produce audit entries with safe metadata only. Audit logs must not contain tokens, passwords, private keys, or full environment dumps.
