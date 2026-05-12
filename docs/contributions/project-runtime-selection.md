# Project Runtime Selection Contribution Scope

## Goal

Persist a per-project PHP version preference and install/select trusted local PHP binaries without enabling project process supervision or service lifecycle execution.

## Current Implementation

- Project PHP version selection is exposed through thin Tauri commands.
- Application use cases validate project IDs and PHP version values before repository writes.
- The PHP branch catalog includes PHP 5.6, 7.0-7.4, and 8.0-8.5. PHP 6 is intentionally omitted because it was never a supported production PHP branch.
- Runtime availability is detected from real local PHP binaries using fixed `php --version` probes through `CommandRunner`.
- Project switching is allowed only when a matching PHP binary is detected on the machine.
- PHP 5.x, 7.x, 8.0, and 8.1 are marked end-of-life. PHP 8.2 and 8.3 are marked security support. PHP 8.4 and 8.5 are marked active support based on the official PHP supported versions schedule: https://www.php.net/supported-versions.php.
- Runtime selections and install requests are stored in the app config directory through a file-backed repository.
- The frontend Projects page can read, install, and switch PHP version preferences for the current project profile.
- Automatic installation is routed through the Rust backend only. macOS uses Homebrew; Windows uses Scoop. Unsupported platforms return a safe configuration error.
- Homebrew installs PHP 8.2-8.5 from core formulae where available, and installs PHP 5.6/7.x/8.0/8.1 from the trusted `shivammathur/php` tap after explicit confirmation.
- Scoop installs versioned PHP branches from the `versions` bucket after explicit confirmation.

## Safety Rules

- Do not infer runtime availability from a selected version.
- Do not link, start, or execute PHP project processes from the selection command.
- Do not run package-manager installs without explicit frontend confirmation.
- Do not allow the frontend to provide command arguments or package names.
- Do not accept arbitrary version strings outside the supported catalog.
- Require explicit user confirmation before installing any PHP branch, with stronger lifecycle warnings for PHP 8.1 and older branches.
- Route every package-manager call through `CommandRunner` with a resolved executable path allowlist, fixed arguments, timeout, and output limits.
- Keep future runtime execution behind runtime adapters and `CommandRunner`.
- Treat the selected PHP binary path as a project runtime preference until a validated runtime driver exists.

## Still Out Of Scope

- Project creation and document root management.
- Per-project process supervision.
- Web server configuration generation.
- Database provisioning.
