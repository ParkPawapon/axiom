# Project Runtime Selection Contribution Scope

## Goal

Persist a per-project PHP version preference without enabling runtime installation, runtime switching, or service lifecycle execution.

## Current Implementation

- Project PHP version selection is exposed through thin Tauri commands.
- Application use cases validate project IDs and PHP version values before repository writes.
- The supported PHP branch catalog currently includes PHP 8.5, 8.4, 8.3, and 8.2 based on the official PHP supported versions schedule: https://www.php.net/supported-versions.php.
- Runtime preferences are stored in the app config directory through a file-backed repository.
- The frontend Projects page can read and save the PHP version preference for the current project profile.

## Safety Rules

- Do not infer runtime availability from a selected version.
- Do not install, link, start, or switch PHP binaries from the selection command.
- Do not accept arbitrary version strings outside the supported catalog.
- Keep future runtime execution behind runtime adapters and `CommandRunner`.
- Treat the selected PHP version as configuration until a validated runtime driver exists.

## Still Out Of Scope

- Project creation and document root management.
- PHP binary installation or discovery-driven switching.
- Per-project process supervision.
- Web server configuration generation.
- Database provisioning.
