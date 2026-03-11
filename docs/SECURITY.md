# Security Notes

## Principles

- Default to least privilege for filesystem and process access.
- Protect the desktop app with a local password.
- Store secrets in the OS keychain when available.
- Keep telemetry disabled until explicitly enabled by the operator.

## Sensitive Data

- Password hashes are stored using Argon2.
- Plaintext secrets should never be persisted in SQLite.
- Backup locations and server paths may contain sensitive infrastructure details and must be treated as private.

## Incident Handling

- Security issues should be triaged privately.
- Releases that address a vulnerability must document operator remediation steps.

