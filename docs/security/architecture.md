# Security Architecture

AxiomPHP is designed as a secure desktop utility for local PHP development workflows. The repository now includes safe passive probes and explicit PHP package-manager installation, while project process supervision and service lifecycle control remain behind future boundaries.

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
