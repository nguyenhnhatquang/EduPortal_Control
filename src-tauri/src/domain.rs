use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

pub(crate) const PORTAL_INDEX: &str = "Portal/build/index.js";
pub(crate) const WEBAPI_EXE: &str = "WebApi/WebApi.exe";
pub(crate) const DATABASE_BACKUP_TASK_NAME: &str = "EduPortal_Control PostgreSQL Backup";
pub(crate) const CADDY_APP_NAME: &str = "Caddy";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub deploy_root: String,
    pub retention: usize,
    pub portal_env: BTreeMap<String, String>,
    pub web_api_env: BTreeMap<String, String>,
    #[serde(default = "default_true")]
    pub portal_install_dependencies: bool,
    #[serde(default = "default_portal_asset_copy")]
    pub portal_asset_copy: PortalAssetCopy,
    #[serde(default = "default_migration_url")]
    pub migration_url: String,
    #[serde(default = "default_migration_key")]
    pub migration_key: String,
    #[serde(default = "default_migration_timeout_secs")]
    pub migration_timeout_secs: u64,
    #[serde(default = "default_database_settings")]
    pub database: DatabaseSettings,
    #[serde(default = "default_caddy_settings")]
    pub caddy: CaddySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalAssetCopy {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_portal_asset_source")]
    pub source: String,
    #[serde(default = "default_portal_asset_destination")]
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSettings {
    #[serde(default = "default_database_host")]
    pub host: String,
    #[serde(default = "default_database_port")]
    pub port: u16,
    #[serde(default = "default_database_name")]
    pub database: String,
    #[serde(default = "default_database_username")]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub bin_dir: String,
    #[serde(default = "default_database_backup_dir")]
    pub backup_dir: String,
    #[serde(default = "default_database_backup_retention")]
    pub backup_retention: usize,
    #[serde(default = "default_database_backup_schedule")]
    pub backup_schedule: DatabaseBackupSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupSchedule {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_database_backup_schedule_frequency")]
    pub frequency: String,
    #[serde(default = "default_database_backup_schedule_time")]
    pub time: String,
    #[serde(default = "default_database_backup_schedule_day")]
    pub day_of_week: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaddySettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_caddy_install_dir")]
    pub install_dir: String,
    #[serde(default = "default_caddy_config_path")]
    pub config_path: String,
    #[serde(default = "default_caddy_config")]
    pub config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRecord {
    pub id: String,
    pub created_at: String,
    pub source_zip: String,
    pub deployment_path: String,
    pub config_path: String,
    pub portal_env: BTreeMap<String, String>,
    pub web_api_env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentState {
    pub active_deployment_id: Option<String>,
    pub deployments: Vec<DeploymentRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub app_version: String,
    pub os: String,
    pub is_windows: bool,
    pub deploy_root: String,
    pub deploy_root_exists: bool,
    pub deploy_root_readonly: Option<bool>,
    pub pm2_execution_enabled: bool,
    pub node_version: Option<String>,
    pub node_error: Option<String>,
    pub pm2_version: Option<String>,
    pub pm2_error: Option<String>,
    pub active_deployment: Option<DeploymentRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePackageStatus {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub error: Option<String>,
    pub executable: String,
    pub path_entries: Vec<String>,
    pub missing_path_entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareInstallResult {
    pub package_id: String,
    pub attempted: bool,
    pub skipped: bool,
    pub success: bool,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
    pub path_entries_added: Vec<String>,
    pub status: SoftwarePackageStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationResult {
    pub success: bool,
    pub url: String,
    pub status_code: Option<u16>,
    pub message: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseCommandResult {
    pub attempted: bool,
    pub skipped: bool,
    pub success: bool,
    pub command: String,
    pub path: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupFile {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupListing {
    pub backup_dir: String,
    pub files: Vec<DatabaseBackupFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaddyStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub error: Option<String>,
    pub executable_path: String,
    pub install_dir: String,
    pub config_path: String,
    pub config_exists: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaddyCommandResult {
    pub attempted: bool,
    pub skipped: bool,
    pub success: bool,
    pub command: String,
    pub path: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
    pub status: CaddyStatus,
    pub pm2: Option<Pm2CommandResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageValidation {
    pub valid: bool,
    pub zip_path: String,
    pub entries_checked: usize,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pm2CommandResult {
    pub attempted: bool,
    pub skipped: bool,
    pub success: bool,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pm2Process {
    pub name: String,
    pub pm_id: Option<i64>,
    pub status: String,
    pub pid: Option<i64>,
    pub restart_time: Option<i64>,
    pub unstable_restarts: Option<i64>,
    pub cpu: Option<f64>,
    pub memory: Option<u64>,
    pub uptime: Option<i64>,
    pub script_path: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostDeployStepResult {
    pub name: String,
    pub success: bool,
    pub skipped: bool,
    pub message: String,
    pub command: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployProgressEvent {
    pub step_id: String,
    pub label: String,
    pub state: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployResult {
    pub deployment: DeploymentRecord,
    pub post_deploy: Vec<PostDeployStepResult>,
    pub pm2: Pm2CommandResult,
    pub removed_deployments: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackResult {
    pub deployment: DeploymentRecord,
    pub pm2: Pm2CommandResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogReadResult {
    pub app_name: String,
    pub path: String,
    pub lines: Vec<String>,
}

pub(crate) fn default_settings() -> Settings {
    Settings {
        deploy_root: default_deploy_root(),
        retention: 5,
        portal_env: default_portal_env(),
        web_api_env: default_web_api_env(),
        portal_install_dependencies: true,
        portal_asset_copy: default_portal_asset_copy(),
        migration_url: default_migration_url(),
        migration_key: default_migration_key(),
        migration_timeout_secs: default_migration_timeout_secs(),
        database: default_database_settings(),
        caddy: default_caddy_settings(),
    }
}

fn default_true() -> bool {
    true
}

fn default_portal_asset_source() -> String {
    "kanji".to_string()
}

fn default_portal_asset_destination() -> String {
    "build/client/kanji".to_string()
}

fn default_portal_asset_copy() -> PortalAssetCopy {
    PortalAssetCopy {
        enabled: true,
        source: default_portal_asset_source(),
        destination: default_portal_asset_destination(),
    }
}

fn default_migration_url() -> String {
    String::new()
}

fn default_migration_key() -> String {
    String::new()
}

fn default_migration_timeout_secs() -> u64 {
    120
}

fn default_database_host() -> String {
    "localhost".to_string()
}

fn default_database_port() -> u16 {
    5432
}

fn default_database_name() -> String {
    "eduportal_control".to_string()
}

fn default_database_username() -> String {
    "postgres".to_string()
}

fn default_database_backup_dir() -> String {
    "backups/postgresql".to_string()
}

fn default_database_backup_retention() -> usize {
    14
}

fn default_database_backup_schedule_frequency() -> String {
    "daily".to_string()
}

fn default_database_backup_schedule_time() -> String {
    "02:00".to_string()
}

fn default_database_backup_schedule_day() -> String {
    "Monday".to_string()
}

fn default_database_backup_schedule() -> DatabaseBackupSchedule {
    DatabaseBackupSchedule {
        enabled: false,
        frequency: default_database_backup_schedule_frequency(),
        time: default_database_backup_schedule_time(),
        day_of_week: default_database_backup_schedule_day(),
    }
}

fn default_database_settings() -> DatabaseSettings {
    DatabaseSettings {
        host: default_database_host(),
        port: default_database_port(),
        database: default_database_name(),
        username: default_database_username(),
        password: String::new(),
        bin_dir: String::new(),
        backup_dir: default_database_backup_dir(),
        backup_retention: default_database_backup_retention(),
        backup_schedule: default_database_backup_schedule(),
    }
}

fn default_caddy_install_dir() -> String {
    "caddy".to_string()
}

fn default_caddy_config_path() -> String {
    "caddy/Caddyfile".to_string()
}

fn default_caddy_config() -> String {
    r#":80 {
    encode gzip

    handle /api/* {
        reverse_proxy localhost:7000
    }

    reverse_proxy localhost:8080
}
"#
    .to_string()
}

fn default_caddy_settings() -> CaddySettings {
    CaddySettings {
        enabled: true,
        install_dir: default_caddy_install_dir(),
        config_path: default_caddy_config_path(),
        config: default_caddy_config(),
    }
}

fn default_deploy_root() -> String {
    if cfg!(windows) {
        "C:\\deploy".to_string()
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join("EduPortal_ControlDeploy")
            .to_string_lossy()
            .to_string()
    }
}

fn default_portal_env() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("NODE_ENV".to_string(), "production".to_string()),
        ("PORT".to_string(), "8080".to_string()),
        ("BODY_SIZE_LIMIT".to_string(), "10M".to_string()),
    ])
}

fn default_web_api_env() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("DOTNET_ENVIRONMENT".to_string(), "Production".to_string()),
        (
            "ASPNETCORE_URLS".to_string(),
            "http://localhost:7000".to_string(),
        ),
    ])
}

pub(crate) fn sanitize_settings(mut settings: Settings) -> Settings {
    if settings.deploy_root.trim().is_empty() {
        settings.deploy_root = default_deploy_root();
    }
    settings.retention = settings.retention.clamp(1, 50);

    let defaults = default_settings();
    for (key, value) in defaults.portal_env {
        settings.portal_env.entry(key).or_insert(value);
    }
    for (key, value) in defaults.web_api_env {
        settings.web_api_env.entry(key).or_insert(value);
    }

    settings.portal_env = sanitize_env(settings.portal_env);
    settings.web_api_env = sanitize_env(settings.web_api_env);
    settings.portal_asset_copy.source = sanitize_asset_source_path(
        &settings.portal_asset_copy.source,
        &default_portal_asset_source(),
    );
    settings.portal_asset_copy.destination = sanitize_relative_path(
        &settings.portal_asset_copy.destination,
        &default_portal_asset_destination(),
    );
    settings.migration_url = settings.migration_url.trim().to_string();
    settings.migration_key = settings.migration_key.trim().to_string();
    settings.migration_timeout_secs = settings.migration_timeout_secs.clamp(5, 600);
    settings.database = sanitize_database_settings(settings.database);
    settings.caddy = sanitize_caddy_settings(settings.caddy);
    settings
}

fn sanitize_caddy_settings(mut caddy: CaddySettings) -> CaddySettings {
    caddy.install_dir =
        sanitize_asset_source_path(&caddy.install_dir, &default_caddy_install_dir());
    caddy.config_path =
        sanitize_asset_source_path(&caddy.config_path, &default_caddy_config_path());
    if caddy.config.trim().is_empty() {
        caddy.config = default_caddy_config();
    }
    caddy
}

fn sanitize_database_settings(mut database: DatabaseSettings) -> DatabaseSettings {
    if database.host.trim().is_empty() {
        database.host = default_database_host();
    } else {
        database.host = database.host.trim().to_string();
    }

    if database.port == 0 {
        database.port = default_database_port();
    }

    if database.database.trim().is_empty() {
        database.database = default_database_name();
    } else {
        database.database = database.database.trim().to_string();
    }

    if database.username.trim().is_empty() {
        database.username = default_database_username();
    } else {
        database.username = database.username.trim().to_string();
    }

    database.bin_dir = sanitize_optional_absolute_path(&database.bin_dir);
    database.backup_dir =
        sanitize_asset_source_path(&database.backup_dir, &default_database_backup_dir());
    database.backup_retention = database.backup_retention.clamp(1, 365);
    database.backup_schedule.frequency =
        sanitize_database_schedule_frequency(&database.backup_schedule.frequency);
    database.backup_schedule.time = sanitize_database_schedule_time(&database.backup_schedule.time);
    database.backup_schedule.day_of_week =
        sanitize_database_schedule_day(&database.backup_schedule.day_of_week);

    database
}

fn sanitize_optional_absolute_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if looks_windows_path(trimmed) || Path::new(trimmed).is_absolute() {
        trimmed.to_string()
    } else {
        String::new()
    }
}

fn sanitize_database_schedule_frequency(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "weekly" => "weekly".to_string(),
        _ => default_database_backup_schedule_frequency(),
    }
}

fn sanitize_database_schedule_day(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "tuesday" => "Tuesday".to_string(),
        "wednesday" => "Wednesday".to_string(),
        "thursday" => "Thursday".to_string(),
        "friday" => "Friday".to_string(),
        "saturday" => "Saturday".to_string(),
        "sunday" => "Sunday".to_string(),
        _ => default_database_backup_schedule_day(),
    }
}

fn sanitize_database_schedule_time(value: &str) -> String {
    let trimmed = value.trim();
    let Some((hour, minute)) = trimmed.split_once(':') else {
        return default_database_backup_schedule_time();
    };
    let Ok(hour) = hour.parse::<u8>() else {
        return default_database_backup_schedule_time();
    };
    let Ok(minute) = minute.parse::<u8>() else {
        return default_database_backup_schedule_time();
    };
    if hour > 23 || minute > 59 {
        return default_database_backup_schedule_time();
    }
    format!("{hour:02}:{minute:02}")
}

fn sanitize_env(env: BTreeMap<String, String>) -> BTreeMap<String, String> {
    env.into_iter()
        .filter_map(|(key, value)| {
            let key = key.trim().to_string();
            if key.is_empty() {
                None
            } else {
                Some((key, value))
            }
        })
        .collect()
}

fn sanitize_relative_path(value: &str, fallback: &str) -> String {
    let normalized = value.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.contains(':') {
        return fallback.to_string();
    }

    let path = Path::new(&normalized);
    let valid = path
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));

    if valid {
        normalized.trim_start_matches("./").to_string()
    } else {
        fallback.to_string()
    }
}

fn sanitize_asset_source_path(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fallback.to_string();
    }

    if looks_windows_path(trimmed) || Path::new(trimmed).is_absolute() {
        return trimmed.to_string();
    }

    sanitize_relative_path(trimmed, fallback)
}

fn looks_windows_path(path: &str) -> bool {
    path.contains('\\') || path.as_bytes().get(1) == Some(&b':')
}
