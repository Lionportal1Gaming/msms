# MSMS

MSMS is a cross-platform Minecraft Server Management System for self-hosting administrators. It provides a desktop control plane for provisioning new local Vanilla servers, managing Java runtimes, controlling the server lifecycle, scheduling backups, and distributing signed app updates.

Provisioning is catalog-backed against Mojang release metadata, validates target directories and managed-server port conflicts before writing files, and pins the selected Java runtime per server for later start and restart operations.

## Stack

- Tauri desktop shell
- Rust application services with SQLite persistence
- React + TypeScript desktop UI
- GitHub Actions for CI and release automation

## Current Scope

- Local-machine management only
- Newly provisioned Vanilla servers only
- Manual and scheduled local backups
- Local app password with secure storage abstraction
- Opt-in diagnostics
- Public release artifacts with private source

## Development

1. Install Node.js 24+ and Rust stable.
2. Install GitHub CLI (`gh`) and authenticate it before running the documented release flow.
3. Install system prerequisites for Tauri for your platform.
4. Run `npm install`.
5. Run `npm run tauri dev`.

## MVP Release Bar

- macOS and Windows are release-blocking for the MVP release path.
- Linux remains in CI and must continue building, but Linux smoke validation is not required before shipping MVP.
- Official pushes, tags, and GitHub Releases target `Lionportal1Gaming/msms`, not a personal account.
- Use `npm run release:preflight` before tagging and `npm run release:stable-dry-run -- vX.Y.Z` for the first stable release rehearsal after the GitHub Release is published.

## Release Standards

- Conventional Commits for commit history
- Semantic Versioning for releases
- `CHANGELOG.md` updated for user-facing changes
- Signed installers and updater metadata for production releases

Additional guidance lives in [docs/HANDBOOK.md](/Users/alextaylor/Documents/Coding/codex-workspace/project02/docs/HANDBOOK.md).
