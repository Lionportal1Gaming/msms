# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-11

### Added
- Desktop-first Minecraft server operations for locally provisioned Vanilla servers, including a Tauri shell, Rust orchestration layer, React admin UI, and SQLite-backed application state.
- Guided provisioning with the official Mojang release catalog, pinned Java runtime selection, installation-path planning, preflight validation, and pre-boot `server.properties` editing.
- Local server lifecycle controls with start, stop, restart, kill, live admin console command submission, persisted console history, and server configuration editing.
- Managed Java runtime discovery and installation workflows surfaced directly in the desktop app.
- Manual and scheduled backups with retention controls, restore support, catch-up execution, and pre-backup `save-all` handling for running servers.
- Local application authentication with first-run password setup plus opt-in diagnostics and operator settings management.
- In-app updater workspace with stable and beta channel selection, release notes, install readiness messaging, and a global update notice.
- Public GitHub release automation for `Lionportal1Gaming/msms`, including stable and beta feed support, GitHub Pages updater feeds, stable dry-run verification, and release-preflight tooling.
- Engineering documentation covering architecture, operator workflow, release process, MVP release readiness, and contribution standards.

### Changed
- The first public release narrative now reflects the complete MVP feature set rather than only the initial scaffold.
- The canonical distribution path is the public `Lionportal1Gaming/msms` repository with GitHub Releases for installers and GitHub Pages for updater metadata.

### Fixed
- Release validation now enforces org-owned GitHub repo targeting, channel-aware tag validation, required repo secrets and variables, and asset verification for stable releases.
- Release docs, operator docs, and readiness checklists now align with the live `0.1.0` release process and public updater feed hosting model.
