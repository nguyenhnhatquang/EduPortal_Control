# EduPortal Control Manager

Desktop operations tool for managing an EduPortal deployment from a Tauri app.

## Stack

- React, TypeScript, and Vite
- Tauri v2 and Rust
- GitHub Actions for signed Windows releases

## Development

```bash
npm install
npm run build
npm audit
```

Rust checks:

```bash
cd src-tauri
cargo fmt --check
cargo test
```

Run the desktop app in development:

```bash
npm run tauri dev
```

## Telegram Bot

The optional Telegram admin bot is configured from the app Settings screen. Send `/start` to the bot to discover the numeric Telegram IDs, and keep bot tokens in local runtime settings only.

Build a local compile check without bundling an installer:

```bash
npm run tauri build -- --no-bundle
```

## Releases

Windows releases are built by GitHub Actions when a version tag is pushed.

| Version | Date | Release notes |
| --- | --- | --- |
| v0.2.8 | 2026-05-05 | Improves Telegram remote administration: the bot now refreshes allowlist settings without replaying a pending update, preventing the confusing `Unauthorized Telegram user.` plus status double-response after saving IDs. Adds `/menu`, `/help`, `/backup`, `/logs`, `/pm2`, and `/deploy` slash commands alongside `/start` and `/status`; `/deploy` still requires Confirm/Cancel and `/pm2` opens the restart target menu for safer operations. Verified with frontend build, npm audit, Rust format/test, and a Tauri release compile check without bundling. |

Before tagging, bump the app version in:

- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Then push a tag:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

The release workflow uses repository secrets for updater signing. Do not commit private keys, passwords, generated installers, or local deployment settings.

## Security Notes

- Runtime credentials must be configured on the deployed machine, not committed to this repository.
- The updater public key is intentionally public; the signing private key must remain in GitHub repository secrets only.
- Local deployment paths, database credentials, migration keys, generated backups, and installer artifacts should stay out of Git.

## Agent Notes

Public agent guidance lives in `AGENTS.md`. Private/local project context may exist under `.codex/` and is intentionally ignored by Git.
