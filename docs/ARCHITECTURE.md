# Architecture

MSMS is a local-first desktop system with a Tauri shell, Rust orchestration layer, and React admin UI.

## High-Level Design

- The React frontend renders operational workflows and invokes typed desktop commands.
- The Rust backend owns persistence, provisioning, process control, backup orchestration, authentication, and updater integration.
- SQLite stores application state, while managed server files remain on disk in user-configured directories.
- Background responsibilities such as scheduled backups run inside the desktop app process.

## Core Domains

- `ManagedServer`: metadata and runtime state for a single provisioned Minecraft server.
- `JavaRuntime`: discovered or app-managed Java installations.
- `BackupJob`: schedule and retention policy for local snapshots.
- `AppSettings`: auth, diagnostics, updater, and filesystem defaults.

## Boundaries

- No external web API in v1.
- No remote host management in v1.
- No existing-server import in v1.
- No non-Vanilla server distributions in v1.

