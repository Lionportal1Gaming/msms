use msms::models::{
    BackupScheduleConfig, BackupSchedulePreset, CreateBackupJobRequest, RestoreBackupRequest,
};
use msms::services::BackupService;

#[test]
fn backup_restore_request_type_is_constructible() {
    let create = CreateBackupJobRequest {
        server_id: "srv-test".into(),
        schedule_preset: BackupSchedulePreset::Daily,
        schedule_config: BackupScheduleConfig {
            interval_hours: None,
            weekday: None,
            hour: Some(2),
            minute: Some(0),
        },
        retention_count: 7,
        destination_path: "/tmp/backups".into(),
    };

    let restore = RestoreBackupRequest {
        archive_path: "/tmp/backups/test.zip".into(),
        target_directory: "/tmp/server".into(),
    };

    assert_eq!(create.retention_count, 7);
    assert_eq!(create.schedule_preset, BackupSchedulePreset::Daily);
    assert!(restore.archive_path.ends_with(".zip"));
    let _ = BackupService::restore;
}
