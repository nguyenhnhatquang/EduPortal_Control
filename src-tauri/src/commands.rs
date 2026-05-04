use crate::{
    apply_caddy_config_blocking, apply_caddy_publish_test_config_blocking,
    configure_caddy_firewall_blocking, control_pm2_app_blocking, deploy_package_blocking,
    get_caddy_status_blocking, get_system_status_blocking, install_bundled_caddy_blocking,
    install_caddy_zip_blocking, install_software_package_blocking, list_database_backups_blocking,
    list_deployments_blocking, list_pm2_processes_blocking, list_software_packages_blocking,
    read_log_blocking, restore_database_backup_blocking, rollback_deployment_blocking,
    run_database_backup_blocking, run_migration_blocking, validate_package_blocking,
};
use crate::{
    domain::{
        CaddyCommandResult, CaddyFirewallResult, CaddyStatus, DatabaseBackupListing,
        DatabaseCommandResult, DeployResult, DeploymentState, LogReadResult, MigrationResult,
        PackageValidation, Pm2CommandResult, Pm2Process, RollbackResult, Settings,
        SoftwareInstallResult, SoftwarePackageStatus, SystemStatus,
    },
    runtime::run_blocking,
    storage,
};
use tauri::AppHandle;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    storage::load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    storage::save_settings(&app, settings)
}

#[tauri::command]
pub async fn get_system_status(app: AppHandle) -> Result<SystemStatus, String> {
    run_blocking("system status", move || get_system_status_blocking(app)).await
}

#[tauri::command]
pub async fn run_migration(app: AppHandle) -> Result<MigrationResult, String> {
    run_blocking("run migration", move || run_migration_blocking(app)).await
}

#[tauri::command]
pub async fn get_caddy_status(app: AppHandle) -> Result<CaddyStatus, String> {
    run_blocking("Caddy status", move || get_caddy_status_blocking(app)).await
}

#[tauri::command]
pub async fn configure_caddy_firewall() -> Result<CaddyFirewallResult, String> {
    run_blocking(
        "configure Caddy firewall",
        configure_caddy_firewall_blocking,
    )
    .await
}

#[tauri::command]
pub async fn install_caddy_zip(
    app: AppHandle,
    zip_path: String,
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    run_blocking("install Caddy", move || {
        install_caddy_zip_blocking(app, zip_path, settings)
    })
    .await
}

#[tauri::command]
pub async fn install_bundled_caddy(
    app: AppHandle,
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    run_blocking("install bundled Caddy", move || {
        install_bundled_caddy_blocking(app, settings)
    })
    .await
}

#[tauri::command]
pub async fn apply_caddy_config(
    app: AppHandle,
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    run_blocking("apply Caddy config", move || {
        apply_caddy_config_blocking(app, settings)
    })
    .await
}

#[tauri::command]
pub async fn apply_caddy_publish_test_config(
    settings: Settings,
) -> Result<CaddyCommandResult, String> {
    run_blocking("apply Caddy publish test config", move || {
        apply_caddy_publish_test_config_blocking(settings)
    })
    .await
}

#[tauri::command]
pub fn list_database_backups(app: AppHandle) -> Result<DatabaseBackupListing, String> {
    list_database_backups_blocking(app)
}

#[tauri::command]
pub async fn run_database_backup(
    app: AppHandle,
    settings: Settings,
) -> Result<DatabaseCommandResult, String> {
    run_blocking("database backup", move || {
        run_database_backup_blocking(app, settings)
    })
    .await
}

#[tauri::command]
pub async fn restore_database_backup(
    app: AppHandle,
    backup_path: String,
    settings: Settings,
) -> Result<DatabaseCommandResult, String> {
    run_blocking("database restore", move || {
        restore_database_backup_blocking(app, backup_path, settings)
    })
    .await
}

#[tauri::command]
pub async fn configure_database_backup_schedule(
    app: AppHandle,
    settings: Settings,
) -> Result<DatabaseCommandResult, String> {
    run_blocking("database backup schedule", move || {
        let settings = storage::save_settings(&app, settings)?;
        crate::configure_postgres_backup_schedule(&app, &settings)
    })
    .await
}

#[tauri::command]
pub async fn validate_package(zip_path: String) -> Result<PackageValidation, String> {
    run_blocking("validate package", move || {
        validate_package_blocking(zip_path)
    })
    .await
}

#[tauri::command]
pub async fn deploy_package(
    app: AppHandle,
    zip_path: String,
    settings: Settings,
) -> Result<DeployResult, String> {
    run_blocking("deploy package", move || {
        deploy_package_blocking(app, zip_path, settings)
    })
    .await
}

#[tauri::command]
pub fn list_deployments(app: AppHandle) -> Result<DeploymentState, String> {
    list_deployments_blocking(app)
}

#[tauri::command]
pub async fn rollback_deployment(
    app: AppHandle,
    deployment_id: String,
) -> Result<RollbackResult, String> {
    run_blocking("rollback deployment", move || {
        rollback_deployment_blocking(app, deployment_id)
    })
    .await
}

#[tauri::command]
pub fn read_log(app: AppHandle, app_name: String, lines: usize) -> Result<LogReadResult, String> {
    read_log_blocking(app, app_name, lines)
}

#[tauri::command]
pub async fn list_pm2_processes() -> Result<Vec<Pm2Process>, String> {
    run_blocking("list PM2 processes", list_pm2_processes_blocking).await
}

#[tauri::command]
pub async fn control_pm2_app(app_name: String, action: String) -> Result<Pm2CommandResult, String> {
    run_blocking("control PM2 app", move || {
        control_pm2_app_blocking(app_name, action)
    })
    .await
}

#[tauri::command]
pub async fn list_software_packages() -> Result<Vec<SoftwarePackageStatus>, String> {
    run_blocking("list software packages", list_software_packages_blocking).await
}

#[tauri::command]
pub async fn install_software_package(package_id: String) -> Result<SoftwareInstallResult, String> {
    run_blocking("install software package", move || {
        install_software_package_blocking(package_id)
    })
    .await
}
