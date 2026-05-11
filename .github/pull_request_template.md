## Summary

- 

## Security Boundary Check

- [ ] No unsafe shell execution was added.
- [ ] No secrets are exposed to the frontend or logs.
- [ ] User-provided paths, ports, names, and environment keys are validated before use.
- [ ] Tauri commands remain thin and call application use cases only.
- [ ] Least-privilege Tauri permissions are preserved.

## Validation

- [ ] `bun typecheck`
- [ ] `bun lint`
- [ ] `bun run build`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`
