# Codex Agent Guide

This repository is public. Keep public files free of deployment secrets, customer data, private host details, local filesystem state, generated installers, backups, and signing private keys.

Before changing code:

- Prefer existing React, TypeScript, Tauri, and Rust patterns.
- Preserve backward compatibility for persisted settings.
- Keep frontend TypeScript contracts in sync with Rust serde camelCase structs.
- Keep privileged operations typed and whitelisted; do not expose arbitrary command execution.
- Keep release builds console-free by preserving the Windows subsystem setting in `src-tauri/src/main.rs`.
- Use concise, utilitarian UI patterns; this is an operations tool.

Verification to run when practical:

```bash
npm run build
npm audit
cd src-tauri && cargo fmt --check && cargo test
cd .. && npm run tauri build -- --no-bundle
```

Private/local context for future agents may be kept under `.codex/`. That directory is ignored by Git and should not be published.
