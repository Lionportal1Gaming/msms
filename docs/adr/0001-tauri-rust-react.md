# ADR 0001: Tauri + Rust + React

## Status

Accepted

## Context

MSMS needs a professional desktop runtime with strong local process control, native packaging, and efficient resource use across Windows, Linux, and macOS.

## Decision

Use Tauri for desktop packaging and command transport, Rust for operational services and persistence, and React + TypeScript for the operator UI.

## Consequences

- We gain strong native integration and low runtime overhead.
- We centralize sensitive operations in Rust rather than browser-managed code.
- We accept a higher bar for local toolchain setup than a pure web stack.

