# Control Center UI Contribution Scope

## Goal

Make AxiomPHP usable as a clean desktop control center while keeping React as a presentation layer and Rust as the security boundary.

## Added Surfaces

- `Control Center` is the first route and default daily-use screen.
- `Setup` provides guided readiness diagnostics without triggering unsafe installation or privileged commands from the frontend.
- `Logs` provides unified sanitized log access for supported backend readers and clear unavailable states for sources that are not connected yet.

## Backend Boundaries

- Control Center data is aggregated by `application/control_center`.
- Tauri commands in `commands/control_center_commands.rs` remain thin and call use cases only.
- UI DTOs live under `domain/control_center` and are serialized explicitly.
- Quick actions are enabled or disabled from backend readiness state, not from frontend guesses.
- Log reads continue through backend log-reader ports and Docker orchestration boundaries.

## Frontend Boundaries

- `features/control-center` owns dashboard composition, status vocabulary mapping, diagnostics, quick actions, and summaries.
- `features/setup-wizard` owns guided setup presentation.
- `features/unified-logs` owns source filters, search, severity filtering, tail count, and sanitized line display.
- Components render state and send user intent only. They do not decide process safety, Docker trust, database readiness, or permission policy.

## Safety Notes

- Disabled actions explain missing prerequisites.
- Browser and folder opening use the Tauri opener plugin instead of shell commands.
- Advanced Docker digest, registry trust, KMS, scheduler, and permission details stay out of the primary dashboard unless diagnostics require attention.
- No fake logs or fake service states are rendered.
- Secrets must never be displayed, copied, logged, or serialized to frontend errors.
