# Security Policy

## Supported Versions

The project is in architecture scaffold stage. Security fixes should target the default branch and the active production foundation branch.

## Security Principles

- Rust is the security boundary between frontend intent and operating-system actions.
- Tauri commands must stay thin and delegate to application use cases.
- OS commands must go through a structured command runner abstraction.
- Secrets must never be sent to frontend logs or stored in plain text.
- Platform credentials must use secure storage adapters.
- User input must be validated before filesystem, process, networking, certificate, or service operations.
- GitHub branch rules must require review and status checks before merge.

## Reporting

Report private security concerns directly to the repository owner.

Do not open public issues for secrets, credential leaks, command injection concerns, or bypass vulnerabilities.
