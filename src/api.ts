import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import type {
  CaddyCommandResult,
  CaddyStatus,
  DatabaseBackupListing,
  DatabaseCommandResult,
  DeploymentState,
  DeployResult,
  LogReadResult,
  ManagerUpdateInfo,
  ManagerUpdateProgress,
  ManagedAppName,
  MigrationResult,
  PackageValidation,
  Pm2Action,
  Pm2CommandResult,
  Pm2Process,
  RollbackResult,
  Settings,
  SoftwareInstallResult,
  SoftwarePackageId,
  SoftwarePackageStatus,
  SystemStatus,
} from "./types";

const mockSettings: Settings = {
  deployRoot: navigator.userAgent.includes("Windows") ? "C:\\deploy" : "~/EduPortal_ControlDeploy",
  retention: 5,
  portalEnv: {
    NODE_ENV: "production",
    PORT: "8080",
    BODY_SIZE_LIMIT: "10M",
  },
  webApiEnv: {
    DOTNET_ENVIRONMENT: "Production",
    ASPNETCORE_URLS: "http://localhost:7000",
  },
  portalInstallDependencies: true,
  portalAssetCopy: {
    enabled: true,
    source: "kanji",
    destination: "build/client/kanji",
  },
  migrationUrl: "",
  migrationKey: "",
  migrationTimeoutSecs: 120,
  database: {
    host: "localhost",
    port: 5432,
    database: "eduportal_control",
    username: "postgres",
    password: "",
    binDir: "",
    backupDir: "backups/postgresql",
    backupRetention: 14,
    backupSchedule: {
      enabled: false,
      frequency: "daily",
      time: "02:00",
      dayOfWeek: "Monday",
    },
  },
  caddy: {
    enabled: true,
    installDir: "caddy",
    configPath: "caddy/Caddyfile",
    config: `:80 {
    encode gzip

    handle /api/* {
        reverse_proxy localhost:7000
    }

    reverse_proxy localhost:8080
}
`,
  },
};

const mockDeploymentState: DeploymentState = {
  activeDeploymentId: null,
  deployments: [],
};

const mockPm2Processes: Pm2Process[] = [
  {
    name: "Portal",
    pmId: null,
    status: "preview",
    pid: null,
    restartTime: null,
    unstableRestarts: null,
    cpu: null,
    memory: null,
    uptime: null,
    scriptPath: null,
    cwd: null,
  },
  {
    name: "WebApi",
    pmId: null,
    status: "preview",
    pid: null,
    restartTime: null,
    unstableRestarts: null,
    cpu: null,
    memory: null,
    uptime: null,
    scriptPath: null,
    cwd: null,
  },
  {
    name: "Caddy",
    pmId: null,
    status: "preview",
    pid: null,
    restartTime: null,
    unstableRestarts: null,
    cpu: null,
    memory: null,
    uptime: null,
    scriptPath: null,
    cwd: null,
  },
];

let pendingManagerUpdate: Update | null = null;

const mockSoftwarePackages: SoftwarePackageStatus[] = [
  {
    id: "nodejs",
    name: "Node.js",
    installed: false,
    version: null,
    error: "Tauri runtime is not active",
    executable: "node.exe",
    pathEntries: ["C:\\Program Files\\nodejs"],
    missingPathEntries: ["C:\\Program Files\\nodejs"],
  },
  {
    id: "pm2",
    name: "PM2",
    installed: false,
    version: null,
    error: "Tauri runtime is not active",
    executable: "pm2.cmd",
    pathEntries: [],
    missingPathEntries: [],
  },
  {
    id: "postgresql",
    name: "PostgreSQL",
    installed: false,
    version: null,
    error: "Tauri runtime is not active",
    executable: "psql.exe",
    pathEntries: ["C:\\Program Files\\PostgreSQL\\18\\bin"],
    missingPathEntries: ["C:\\Program Files\\PostgreSQL\\18\\bin"],
  },
];

export function isTauriRuntime() {
  return Boolean(window.__TAURI_INTERNALS__);
}

export async function pickZipPackage() {
  if (!isTauriRuntime()) {
    return window.prompt("Zip package path")?.trim() || null;
  }

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Deployment package", extensions: ["zip"] }],
  });

  return typeof selected === "string" ? selected : null;
}

export async function pickCaddyZipPackage() {
  if (!isTauriRuntime()) {
    return window.prompt("Caddy zip path")?.trim() || null;
  }

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Caddy zip", extensions: ["zip"] }],
  });

  return typeof selected === "string" ? selected : null;
}

export async function pickDatabaseBackupFile() {
  if (!isTauriRuntime()) {
    return window.prompt("PostgreSQL backup file path")?.trim() || null;
  }

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "PostgreSQL backup", extensions: ["dump", "backup", "sql"] }],
  });

  return typeof selected === "string" ? selected : null;
}

export async function pickDatabaseBackupDirectory() {
  if (!isTauriRuntime()) {
    return window.prompt("PostgreSQL backup directory")?.trim() || null;
  }

  const selected = await open({
    multiple: false,
    directory: true,
  });

  return typeof selected === "string" ? selected : null;
}

export async function getSettings() {
  if (!isTauriRuntime()) return mockSettings;
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings) {
  if (!isTauriRuntime()) return settings;
  return invoke<Settings>("save_settings", { settings });
}

export async function getSystemStatus() {
  if (!isTauriRuntime()) {
    return {
      appVersion: "0.1.0",
      os: "browser-preview",
      isWindows: false,
      deployRoot: mockSettings.deployRoot,
      deployRootExists: false,
      deployRootReadonly: null,
      pm2ExecutionEnabled: false,
      nodeVersion: null,
      nodeError: "Tauri runtime is not active",
      pm2Version: null,
      pm2Error: "Tauri runtime is not active",
      activeDeployment: null,
    } satisfies SystemStatus;
  }
  return invoke<SystemStatus>("get_system_status");
}

function toManagerUpdateInfo(update: Update): ManagerUpdateInfo {
  return {
    currentVersion: update.currentVersion,
    version: update.version,
    date: update.date ?? null,
    body: update.body ?? null,
  };
}

export async function checkManagerUpdate() {
  if (!isTauriRuntime()) return null;

  if (pendingManagerUpdate) {
    void pendingManagerUpdate.close().catch(() => undefined);
    pendingManagerUpdate = null;
  }

  const update = await check({ timeout: 30_000 });
  pendingManagerUpdate = update;
  return update ? toManagerUpdateInfo(update) : null;
}

export async function installManagerUpdate(onProgress?: (progress: ManagerUpdateProgress) => void) {
  if (!isTauriRuntime()) return;

  if (!pendingManagerUpdate) {
    pendingManagerUpdate = await check({ timeout: 30_000 });
  }

  const update = pendingManagerUpdate;
  if (!update) {
    throw new Error("No manager update is available.");
  }

  let downloadedBytes = 0;
  let contentLength: number | null = null;

  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === "Started") {
      downloadedBytes = 0;
      contentLength = event.data.contentLength ?? null;
    }
    if (event.event === "Progress") {
      downloadedBytes += event.data.chunkLength;
    }
    if (event.event === "Finished") {
      downloadedBytes = contentLength ?? downloadedBytes;
    }
    onProgress?.({ downloadedBytes, contentLength });
  });

  pendingManagerUpdate = null;
  await relaunch().catch(() => undefined);
}

export async function getCaddyStatus() {
  if (!isTauriRuntime()) {
    return {
      installed: false,
      version: null,
      error: "Tauri runtime is not active",
      executablePath: `${mockSettings.deployRoot}\\caddy\\caddy.exe`,
      installDir: `${mockSettings.deployRoot}\\caddy`,
      configPath: `${mockSettings.deployRoot}\\caddy\\Caddyfile`,
      configExists: false,
    } satisfies CaddyStatus;
  }
  return invoke<CaddyStatus>("get_caddy_status");
}

export async function installCaddyZip(zipPath: string, settings: Settings) {
  if (!isTauriRuntime()) {
    const status = {
      installed: zipPath.toLowerCase().endsWith(".zip"),
      version: zipPath.toLowerCase().endsWith(".zip") ? "preview" : null,
      error: zipPath.toLowerCase().endsWith(".zip") ? null : "Zip path is required",
      executablePath: `${settings.deployRoot}\\${settings.caddy.installDir}\\caddy.exe`,
      installDir: `${settings.deployRoot}\\${settings.caddy.installDir}`,
      configPath: `${settings.deployRoot}\\${settings.caddy.configPath}`,
      configExists: true,
    } satisfies CaddyStatus;
    return {
      attempted: true,
      skipped: false,
      success: status.installed,
      command: "extract preview",
      path: status.executablePath,
      stdout: "",
      stderr: "",
      message: status.installed ? "Caddy install preview completed." : "Caddy zip is missing.",
      status,
      pm2: null,
    } satisfies CaddyCommandResult;
  }
  return invoke<CaddyCommandResult>("install_caddy_zip", { zipPath, settings });
}

export async function installBundledCaddy(settings: Settings) {
  if (!isTauriRuntime()) {
    const status = {
      installed: true,
      version: "v2.11.2 preview",
      error: null,
      executablePath: `${settings.deployRoot}\\${settings.caddy.installDir}\\caddy.exe`,
      installDir: `${settings.deployRoot}\\${settings.caddy.installDir}`,
      configPath: `${settings.deployRoot}\\${settings.caddy.configPath}`,
      configExists: true,
    } satisfies CaddyStatus;
    return {
      attempted: true,
      skipped: false,
      success: true,
      command: "extract bundled Caddy preview",
      path: status.executablePath,
      stdout: "",
      stderr: "",
      message: "Bundled Caddy install preview completed.",
      status,
      pm2: null,
    } satisfies CaddyCommandResult;
  }
  return invoke<CaddyCommandResult>("install_bundled_caddy", { settings });
}

export async function applyCaddyConfig(settings: Settings) {
  if (!isTauriRuntime()) {
    const status = {
      installed: true,
      version: "preview",
      error: null,
      executablePath: `${settings.deployRoot}\\${settings.caddy.installDir}\\caddy.exe`,
      installDir: `${settings.deployRoot}\\${settings.caddy.installDir}`,
      configPath: `${settings.deployRoot}\\${settings.caddy.configPath}`,
      configExists: true,
    } satisfies CaddyStatus;
    return {
      attempted: true,
      skipped: !settings.caddy.enabled,
      success: true,
      command: "caddy validate preview",
      path: status.configPath,
      stdout: "",
      stderr: "",
      message: settings.caddy.enabled
        ? "Caddyfile preview validated and PM2 reload preview completed."
        : "Caddyfile preview validated. PM2 Caddy is disabled.",
      status,
      pm2: null,
    } satisfies CaddyCommandResult;
  }
  return invoke<CaddyCommandResult>("apply_caddy_config", { settings });
}

export async function validatePackage(zipPath: string) {
  if (!isTauriRuntime()) {
    return {
      valid: zipPath.toLowerCase().endsWith(".zip"),
      zipPath,
      entriesChecked: 0,
      missing: zipPath.toLowerCase().endsWith(".zip")
        ? []
        : ["Portal/build/index.js", "WebApi/WebApi.exe"],
    } satisfies PackageValidation;
  }
  return invoke<PackageValidation>("validate_package", { zipPath });
}

export async function deployPackage(zipPath: string, settings: Settings) {
  return invoke<DeployResult>("deploy_package", { zipPath, settings });
}

export async function listDeployments() {
  if (!isTauriRuntime()) return mockDeploymentState;
  return invoke<DeploymentState>("list_deployments");
}

export async function rollbackDeployment(deploymentId: string) {
  return invoke<RollbackResult>("rollback_deployment", { deploymentId });
}

export async function readLog(appName: ManagedAppName, lines: number) {
  if (!isTauriRuntime()) {
    return {
      appName,
      path: `${mockSettings.deployRoot}/pm2/logs/${appName}.log`,
      lines: ["Tauri runtime is not active. Run the desktop app to read VPS logs."],
    } satisfies LogReadResult;
  }
  return invoke<LogReadResult>("read_log", { appName, lines });
}

export async function listPm2Processes() {
  if (!isTauriRuntime()) return mockPm2Processes;
  return invoke<Pm2Process[]>("list_pm2_processes");
}

export async function controlPm2App(appName: ManagedAppName, action: Pm2Action) {
  if (!isTauriRuntime()) {
    return {
      attempted: false,
      skipped: true,
      success: true,
      command: `pm2 ${action} ${appName}`,
      stdout: "",
      stderr: "",
      message: "Tauri runtime is not active. Run the desktop app on the VPS to control PM2.",
    } satisfies Pm2CommandResult;
  }
  return invoke<Pm2CommandResult>("control_pm2_app", { appName, action });
}

export async function listSoftwarePackages() {
  if (!isTauriRuntime()) return mockSoftwarePackages;
  return invoke<SoftwarePackageStatus[]>("list_software_packages");
}

export async function installSoftwarePackage(packageId: SoftwarePackageId) {
  if (!isTauriRuntime()) {
    const status = mockSoftwarePackages.find((item) => item.id === packageId) ?? mockSoftwarePackages[0];
    return {
      packageId,
      attempted: false,
      skipped: true,
      success: true,
      command: "preview",
      stdout: "",
      stderr: "",
      message: "Tauri runtime is not active. Run the desktop app on the VPS to install software.",
      pathEntriesAdded: [],
      status,
    } satisfies SoftwareInstallResult;
  }
  return invoke<SoftwareInstallResult>("install_software_package", { packageId });
}

export async function runMigration() {
  if (!isTauriRuntime()) {
    return {
      success: true,
      url: mockSettings.migrationUrl,
      statusCode: 200,
      message: "Migration preview completed.",
      body: "done",
    } satisfies MigrationResult;
  }
  return invoke<MigrationResult>("run_migration");
}

export async function listDatabaseBackups() {
  if (!isTauriRuntime()) {
    return {
      backupDir: mockSettings.database.backupDir,
      files: [],
    } satisfies DatabaseBackupListing;
  }
  return invoke<DatabaseBackupListing>("list_database_backups");
}

export async function runDatabaseBackup(settings: Settings) {
  if (!isTauriRuntime()) {
    const path = `${settings.database.backupDir}/${settings.database.database}_preview.dump`;
    return {
      attempted: false,
      skipped: true,
      success: true,
      command: "pg_dump preview",
      path,
      stdout: "",
      stderr: "",
      message: `Backup preview created at ${path}.`,
    } satisfies DatabaseCommandResult;
  }
  return invoke<DatabaseCommandResult>("run_database_backup", { settings });
}

export async function restoreDatabaseBackup(backupPath: string, settings: Settings) {
  if (!isTauriRuntime()) {
    return {
      attempted: false,
      skipped: true,
      success: true,
      command: "pg_restore preview",
      path: backupPath,
      stdout: "",
      stderr: "",
      message: `Restore preview completed for ${settings.database.database}.`,
    } satisfies DatabaseCommandResult;
  }
  return invoke<DatabaseCommandResult>("restore_database_backup", { backupPath, settings });
}

export async function configureDatabaseBackupSchedule(settings: Settings) {
  if (!isTauriRuntime()) {
    return {
      attempted: false,
      skipped: true,
      success: true,
      command: "schtasks preview",
      path: null,
      stdout: "",
      stderr: "",
      message: settings.database.backupSchedule.enabled
        ? "Backup schedule preview configured."
        : "Backup schedule preview disabled.",
    } satisfies DatabaseCommandResult;
  }
  return invoke<DatabaseCommandResult>("configure_database_backup_schedule", { settings });
}
