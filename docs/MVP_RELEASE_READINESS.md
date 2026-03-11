# MVP Release Readiness

Use this checklist to decide whether a build is safe to tag for the MVP release path.

## Ship Blockers

- Local password setup and unlock work on a clean install.
- Vanilla provisioning completes with a pinned Java runtime.
- Server lifecycle controls work: start, stop, restart, kill, and console command submission.
- Backup execution and restore complete successfully.
- Update checks work and the updater install flow shows correct readiness/restart messaging.

## Release Tooling

- `gh` is installed and authenticated locally.
- `npm run check` passes.
- `npm run release:preflight` passes.
- Release validation confirms version alignment, changelog presence, updater config readiness, and tag/channel consistency.

## Platform Sign-Off

- macOS installer smoke test passes.
- macOS updater smoke test passes.
- Windows installer smoke test passes.
- Windows updater smoke test passes.
- Linux build passes in CI.

Linux is not release-blocking for MVP, but a failed Linux build still blocks the release workflow.
