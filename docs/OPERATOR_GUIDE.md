# Operator Guide

## First Run

1. Launch MSMS.
2. Create the local application password.
3. Choose the update channel plus default runtime, server, and backup directories.
4. Decide whether to opt in to diagnostics.

## Provision A Server

1. Open the provisioning workflow.
2. Select an official Mojang release from the catalog and choose the Java runtime that will be pinned to that server.
3. Review the computed installation path, memory allocation, and network port.
4. Resolve any validation warnings before continuing.
5. Edit `server.properties` as needed, review the final configuration, accept the EULA, and provision the server.
6. Start the server and monitor the console output.

## Backups

- Use manual backups before risky changes.
- Configure retention limits for scheduled backups.
- Test restore procedures before production use.

## Updates

- Check for updates from Settings or the global update notice.
- Stable and beta channels are tracked independently.
- Installing an app update will gracefully stop running managed servers before the installer runs.
- Review the published release notes in the updater panel before installing.
- Stable updater metadata is published to `https://lionportal1gaming.github.io/msms/updates/stable/latest.json`.
- Beta updater metadata is published to `https://lionportal1gaming.github.io/msms/updates/beta/latest.json`.

## MVP Release Support

- macOS and Windows are the MVP release-blocking desktop platforms.
- Linux builds are still produced in CI, but Linux smoke validation is not required before the MVP release is approved.
- Official release artifacts and updater metadata are published from the `Lionportal1Gaming/msms` GitHub repository.
- The first public release line is `0.1.0`.
