# Security Architecture

AxiomPHP is designed as a secure desktop utility for local PHP development workflows. The current repository is a scaffold only, but the boundaries are intentionally prepared before privileged features are added.

## Trust Boundaries

- The React frontend is presentation and user intent only.
- The Tauri command layer is a narrow serialization boundary.
- Application use cases own orchestration.
- Domain modules stay pure and platform independent.
- Infrastructure adapters own external systems.
- Platform adapters isolate macOS and Windows behavior.

## Command Execution Policy

Future OS-level execution must use structured command arguments through `CommandRunner`. Shell string concatenation, unsanitized arguments, and direct process execution from Tauri commands are not allowed.

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
