# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub org bootstrap tooling for the canonical `Lionportal1Gaming/msms` repository, including org-ownership checks and a private-repo bootstrap script.
- Release preflight now validates `origin`, GitHub org ownership, and org-scoped GitHub Release verification instead of relying on the active personal account context.
- Release docs now define `Lionportal1Gaming/msms` as the official MVP push/release target and document required org repo settings.
- MVP release-hardening flow with `gh`-based local preflight, GitHub release verification, and explicit macOS/Windows release gates.
- Release validation now checks Tauri config version alignment, updater config slots, and tag/channel consistency for stable and beta releases.
- Release docs now include platform smoke-test expectations and an MVP readiness checklist.
- Catalog-backed Vanilla provisioning with Mojang release selection, preflight validation, and pinned per-server Java runtime selection.
- Multi-step provisioning workflow with full `server.properties` editing before first boot and review-stage validation feedback.
- Provisioning safety checks for managed-server port conflicts, conflicting install directories, and out-of-policy memory requests.
- In-app updater UX with stable and beta channel selection, release notes, and install controls.
- Global update surfacing outside Settings plus restart-required messaging after install.
- Separate stable and beta updater feed configuration guidance for release publishing.

## [0.1.0] - 2026-03-10

### Added
- Initial MSMS foundation scaffold with a Tauri desktop architecture.
- Rust service modules for provisioning, runtime management, backups, authentication, and lifecycle control.
- React admin dashboard covering server overview, provisioning, backups, console, and settings.
- Release engineering baseline including CI, release workflow, documentation handbook, and changelog validation.
