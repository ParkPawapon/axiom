# Process Boundary Contribution Scope

## Goal

Provide the secure process execution foundation required before real OS-level service management is enabled.

## Current Implementation

- `CommandRunner` executes real OS processes without shell execution.
- `CommandPolicy` blocks all programs by default and requires explicit allowlists.
- Shell programs are blocked even if accidentally allowlisted.
- Process arguments, environment keys, working directories, and timeouts are validated before spawn.
- Child processes use null stdin, captured stdout/stderr, capped output buffers, timeout handling, and safe process kill on timeout.
- Process output is redacted before it can cross the process boundary.
- Tests cover allowlist execution, denied commands, output truncation, redaction, and policy validation.

## Safety Rules

- Do not expose a generic frontend command runner.
- Do not run shell strings.
- Do not log raw arguments, environment values, stdout, or stderr.
- Use explicit runtime/service adapters to decide which programs are allowed.
- Add service-specific tests before connecting this runner to service start or stop use cases.

## Still Out Of Scope

- Starting PHP, MySQL, PostgreSQL, Docker, or reverse proxy services.
- Editing hosts files.
- Generating or installing certificates.
- Long-running daemon supervision.
- Privilege elevation.
