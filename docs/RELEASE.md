# Release Process

## Prerequisites

- `gh` is installed locally and authenticated with `gh auth login -h github.com`.
- The canonical GitHub repository is the public org repo `Lionportal1Gaming/msms`.
- `origin` points to `github.com/Lionportal1Gaming/msms`.
- All tests pass locally and in CI.
- `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` share the same version.
- `CHANGELOG.md` contains the release heading and user-facing notes for the tagged version.
- `MSMS_UPDATER_PUBLIC_KEY`, `MSMS_UPDATER_STABLE_ENDPOINT`, and `MSMS_UPDATER_BETA_ENDPOINT` are configured for release automation.
- Tauri signing secrets are configured in GitHub Actions secrets.

## GitHub Pages Updater Feed

The MVP updater feed is published from GitHub Pages in the canonical org repo.

- Stable feed URL:
  `https://lionportal1gaming.github.io/msms/updates/stable/latest.json`
- Beta feed URL:
  `https://lionportal1gaming.github.io/msms/updates/beta/latest.json`

Configure the repo variables to those URLs unless you intentionally replace the feed host.

## First-Time GitHub Org Bootstrap

1. Re-authenticate GitHub CLI if needed:
   `gh auth login -h github.com`
2. Create or attach the canonical org repo:
   `npm run release:bootstrap`
3. Push the local `main` branch to `origin` if the GitHub repo is still empty so the default branch can be set correctly.
4. Verify the local checkout is targeting the org repo:
   `npm run release:repo-check`
5. Confirm the repository is public, the default branch is `main`, and GitHub Actions is enabled.
6. Configure the required repository variables and secrets:
   Repository variables:
   `MSMS_UPDATER_PUBLIC_KEY`
   `MSMS_UPDATER_STABLE_ENDPOINT=https://lionportal1gaming.github.io/msms/updates/stable/latest.json`
   `MSMS_UPDATER_BETA_ENDPOINT=https://lionportal1gaming.github.io/msms/updates/beta/latest.json`
   Repository secrets:
   `TAURI_SIGNING_PRIVATE_KEY`
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
7. Add branch protection on `main` with required status checks before release tags are pushed.
8. Enable GitHub Pages for the repository and serve from the `gh-pages` branch root.

## Stable Release Gates

- Stable tags must use `vX.Y.Z`.
- The GitHub repository must stay public under `Lionportal1Gaming/msms` for the GitHub Pages updater-feed path.
- `main` must be the default branch.
- GitHub Actions must be enabled.
- The required repository variables and secrets must be present before a stable tag is pushed.
- `main` must have branch protection with required status checks enabled.

## MVP Platform Bar

- macOS and Windows are release-blocking for MVP.
- Linux must continue to build in CI, but Linux smoke validation is not required before the MVP tag is considered ready.

## Local Preflight

1. Run `npm run check`.
2. Verify GitHub CLI, `origin`, repo settings, public Pages readiness, and org ownership with `npm run release:preflight`.
3. Confirm the updater environment values are present locally if you want to dry-run release validation:
   `MSMS_UPDATER_PUBLIC_KEY`, `MSMS_UPDATER_STABLE_ENDPOINT`, `MSMS_UPDATER_BETA_ENDPOINT`
4. For a tagged build candidate, run:
   `npm run release:preflight -- --channel stable --tag vX.Y.Z`
   Beta tags use `npm run release:preflight -- --channel beta --tag vX.Y.Z-beta.N`.

## First Stable Release Dry Run

Use this flow for the first live MVP rehearsal against the org repo after the stable tag has been pushed and the release workflow has finished:

1. Run:
   `npm run release:stable-dry-run -- vX.Y.Z`
2. Confirm the command resolves `gh` access against `Lionportal1Gaming/msms`, not a personal account.
3. Confirm the stable GitHub Release is published, not marked prerelease, and includes:
   macOS installer or updater bundle assets
   Windows installer assets
   Linux build assets
   `latest.json` updater metadata
4. Confirm the stable updater feed points at the published `latest.json`.
5. Complete the macOS and Windows smoke checks before calling the release candidate ready.

## Release Checklist

1. Merge only Conventional Commit history intended for the release.
2. Update `CHANGELOG.md` so the tagged version contains the full operator-facing release narrative and `Unreleased` is left clean.
3. Run the local preflight flow:
   `npm run check`
   `npm run release:repo-check`
   `npm run release:preflight`
4. Create and push the Git tag for the SemVer release.
   Stable tags use `vX.Y.Z`.
   Beta tags use `vX.Y.Z-beta.N`.
5. Verify GitHub Actions passed the release validation and prepared the Tauri updater config for the matching channel.
6. Verify GitHub Actions built signed installers for macOS and Windows, and confirm Linux still built successfully in CI.
7. Confirm the generated GitHub release body matches the intended operator-facing release notes.
8. Inspect the published GitHub release locally:
   `npm run release:verify -- vX.Y.Z`
   Beta tags use `npm run release:verify -- vX.Y.Z-beta.N`
   All release inspection must resolve to `Lionportal1Gaming/msms`, not a personal account.
   Stable verification must confirm macOS, Windows, Linux, and updater metadata assets are present on the GitHub Release.
9. Publish updater metadata to the matching channel feed:
   The release workflow publishes `latest.json` automatically to the GitHub Pages feed for the matching channel.
   Stable feed: `https://lionportal1gaming.github.io/msms/updates/stable/latest.json`
   Beta feed: `https://lionportal1gaming.github.io/msms/updates/beta/latest.json`
10. Validate updater metadata, installer signatures, and smoke-test installation.
11. Confirm the in-app updater sees the new version on the intended channel.

## MVP Smoke Checklist

Run these checks on macOS and Windows before calling the release candidate ready:

- First-run password setup and unlock
- Provision a Vanilla server and confirm the pinned Java runtime is used on start
- Start, stop, restart, and submit a console command
- Create a backup, run it, and restore the archive
- Check for app updates and verify the updater screen/restart-required messaging
- Install the generated package and verify the signed installer path succeeds

Linux expectation for MVP:

- Release workflow build passes
- No release-blocking smoke test is required

## Post-Release Verification

- Use `gh release view <tag> --repo Lionportal1Gaming/msms` or `npm run release:verify -- <tag>` to confirm the GitHub release is published and channel-correct.
- Confirm the GitHub Release contains installer artifacts and updater metadata assets.
- Confirm the stable or beta feed serves the expected `latest.json`.
- Confirm the in-app updater on a clean install sees the new version on the intended channel.

## Rollback

1. Revoke the affected stable or beta updater feed if the issue is severe.
2. Publish a superseding patch release rather than rewriting version history.
3. Record the incident and remediation steps in engineering notes.
