# Release Process

## Prerequisites

- `gh` is installed locally and authenticated with `gh auth login -h github.com`.
- The canonical GitHub repository is the private org repo `Lionportal1Gaming/msms`.
- `origin` points to `github.com/Lionportal1Gaming/msms`.
- All tests pass locally and in CI.
- `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` share the same version.
- `CHANGELOG.md` contains the release heading and user-facing notes for the tagged version.
- `MSMS_UPDATER_PUBLIC_KEY`, `MSMS_UPDATER_STABLE_ENDPOINT`, and `MSMS_UPDATER_BETA_ENDPOINT` are configured for release automation.
- Tauri signing secrets are configured in GitHub Actions secrets.

## First-Time GitHub Org Bootstrap

1. Re-authenticate GitHub CLI if needed:
   `gh auth login -h github.com`
2. Create or attach the canonical org repo:
   `npm run release:bootstrap`
3. Verify the local checkout is targeting the org repo:
   `npm run release:repo-check`
4. Confirm the repository is private, the default branch is `main`, and GitHub Actions is enabled.
5. Configure the required repository variables and secrets:
   Repository variables:
   `MSMS_UPDATER_PUBLIC_KEY`
   `MSMS_UPDATER_STABLE_ENDPOINT`
   `MSMS_UPDATER_BETA_ENDPOINT`
   Repository secrets:
   `TAURI_SIGNING_PRIVATE_KEY`
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
6. Add branch protection on `main` with CI required before release tags are pushed.

## MVP Platform Bar

- macOS and Windows are release-blocking for MVP.
- Linux must continue to build in CI, but Linux smoke validation is not required before the MVP tag is considered ready.

## Local Preflight

1. Run `npm run check`.
2. Verify GitHub CLI, `origin`, and org ownership with `npm run release:preflight`.
3. Confirm the updater environment values are present locally if you want to dry-run release validation:
   `MSMS_UPDATER_PUBLIC_KEY`, `MSMS_UPDATER_STABLE_ENDPOINT`, `MSMS_UPDATER_BETA_ENDPOINT`
4. For a tagged build candidate, run:
   `node scripts/validate-release.mjs --channel stable --tag vX.Y.Z --require-gh --require-remote`
   Beta tags use `--channel beta --tag vX.Y.Z-beta.N --require-gh --require-remote`.

## Release Checklist

1. Merge only Conventional Commit history intended for the release.
2. Update version numbers and `CHANGELOG.md`.
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
   The verification step must confirm both installer assets and updater metadata assets are present on the GitHub Release.
9. Publish updater metadata to the matching channel feed:
   Stable feed: `MSMS_UPDATER_STABLE_ENDPOINT`
   Beta feed: `MSMS_UPDATER_BETA_ENDPOINT`
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
