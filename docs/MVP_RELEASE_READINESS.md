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
- `origin` points to `github.com/Lionportal1Gaming/msms`.
- `npm run release:repo-check` passes.
- `npm run check` passes.
- `npm run release:preflight` passes.
- `npm run release:stable-dry-run -- vX.Y.Z` passes for the first stable tag candidate after the GitHub Release is published.
- Release validation confirms version alignment, changelog presence, updater config readiness, and tag/channel consistency.

## GitHub Org Readiness

- The canonical private repository exists under `Lionportal1Gaming`.
- Official pushes and release tags target the org repo, not a personal account.
- GitHub Releases for MVP installers and updater metadata are published from `Lionportal1Gaming/msms`.
- Repository variables and signing secrets are configured before the first release tag.
- GitHub Actions is enabled for the private repo.
- `main` is the default branch and has branch protection with required status checks.
- GitHub Release verification confirms stable release assets for macOS, Windows, Linux, and updater metadata are present.

## Platform Sign-Off

- macOS installer smoke test passes.
- macOS updater smoke test passes.
- Windows installer smoke test passes.
- Windows updater smoke test passes.
- Linux build passes in CI.

Linux is not release-blocking for MVP, but a failed Linux build still blocks the release workflow.
