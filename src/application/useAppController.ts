import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  applyCaddyConfig,
  applyCaddyPublishTestConfig,
  checkManagerUpdate,
  configureCaddyFirewall,
  controlPm2App,
  configureDatabaseBackupSchedule,
  deployPackage,
  getCaddyStatus,
  getSettings,
  getSystemStatus,
  installBundledCaddy,
  installCaddyZip,
  installManagerUpdate,
  installSoftwarePackage,
  isTauriRuntime,
  listDatabaseBackups,
  listDeployments,
  listPm2Processes,
  listSoftwarePackages,
  pickCaddyZipPackage,
  pickDatabaseBackupDirectory,
  pickDatabaseBackupFile,
  pickZipPackage,
  readLog,
  restoreDatabaseBackup,
  rollbackDeployment,
  runDatabaseBackup,
  runMigration,
  saveSettings,
  validatePackage,
} from "../api";
import {
  applyDeployProgressEvent,
  buildDeployStepsFromResult,
  createDeploySteps,
  failActiveDeployStep,
  markDeployStep,
} from "../domain/deploy/deploy-steps";
import { CADDY_PM2_APP, findPm2Process, pm2BusyKey } from "../domain/pm2";
import type { DeployStepView } from "../domain/deploy/types";
import { fallbackSettings } from "../domain/settings/defaults";
import { errorMessage } from "../shared/errors";
import type {
  CaddyCommandResult,
  CaddyFirewallResult,
  CaddyStatus,
  DatabaseBackupFile,
  DatabaseCommandResult,
  DeploymentRecord,
  DeploymentState,
  DeployProgressEvent,
  LogReadResult,
  ManagerUpdateInfo,
  ManagerUpdateProgress,
  ManagedAppName,
  MigrationResult,
  PackageValidation,
  Pm2Action,
  Pm2Process,
  Settings,
  SoftwareInstallResult,
  SoftwarePackageId,
  SoftwarePackageStatus,
  SystemStatus,
} from "../types";
import type { TabId } from "../presentation/tabs/types";

function sortDeploymentState(state: DeploymentState): DeploymentState {
  return {
    ...state,
    deployments: [...state.deployments].sort((a, b) => b.id.localeCompare(a.id)),
  };
}

export function useAppController() {
  const [activeTab, setActiveTab] = useState<TabId>("overview");
  const [settings, setSettings] = useState<Settings>(fallbackSettings);
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [deployments, setDeployments] = useState<DeploymentState>({
    activeDeploymentId: null,
    deployments: [],
  });
  const [pm2Processes, setPm2Processes] = useState<Pm2Process[]>([]);
  const [packagePath, setPackagePath] = useState("");
  const [validation, setValidation] = useState<PackageValidation | null>(null);
  const [deploySteps, setDeploySteps] = useState<DeployStepView[]>(createDeploySteps());
  const [logApp, setLogApp] = useState<ManagedAppName>("Portal");
  const [logLines, setLogLines] = useState(300);
  const [logResult, setLogResult] = useState<LogReadResult | null>(null);
  const [migrationResult, setMigrationResult] = useState<MigrationResult | null>(null);
  const [databaseBackupFiles, setDatabaseBackupFiles] = useState<DatabaseBackupFile[]>([]);
  const [databaseBackupDir, setDatabaseBackupDir] = useState(fallbackSettings.database.backupDir);
  const [databaseBackupResult, setDatabaseBackupResult] = useState<DatabaseCommandResult | null>(null);
  const [databaseRestoreResult, setDatabaseRestoreResult] = useState<DatabaseCommandResult | null>(null);
  const [databaseScheduleResult, setDatabaseScheduleResult] = useState<DatabaseCommandResult | null>(null);
  const [databaseRestorePath, setDatabaseRestorePath] = useState("");
  const [caddyZipPath, setCaddyZipPath] = useState("");
  const [caddyStatus, setCaddyStatus] = useState<CaddyStatus | null>(null);
  const [caddyInstallResult, setCaddyInstallResult] = useState<CaddyCommandResult | null>(null);
  const [caddyApplyResult, setCaddyApplyResult] = useState<CaddyCommandResult | null>(null);
  const [caddyFirewallResult, setCaddyFirewallResult] = useState<CaddyFirewallResult | null>(null);
  const [softwarePackages, setSoftwarePackages] = useState<SoftwarePackageStatus[]>([]);
  const [softwareInstallResult, setSoftwareInstallResult] = useState<SoftwareInstallResult | null>(null);
  const [managerUpdate, setManagerUpdate] = useState<ManagerUpdateInfo | null>(null);
  const [managerUpdateProgress, setManagerUpdateProgress] = useState<ManagerUpdateProgress | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const activeDeployment = useMemo(() => {
    if (!deployments.activeDeploymentId) return null;
    return deployments.deployments.find((deployment) => deployment.id === deployments.activeDeploymentId) ?? null;
  }, [deployments]);

  const caddyProcess = useMemo(
    () => findPm2Process(pm2Processes, CADDY_PM2_APP),
    [pm2Processes],
  );

  const refreshAll = useCallback(async () => {
    setBusy("refresh");
    setError(null);
    try {
      const [nextSettings, nextStatus, nextDeployments, nextPm2Processes, nextSoftwarePackages, nextCaddyStatus] = await Promise.all([
        getSettings(),
        getSystemStatus(),
        listDeployments(),
        listPm2Processes(),
        listSoftwarePackages(),
        getCaddyStatus(),
      ]);
      setSettings(nextSettings);
      setStatus(nextStatus);
      setDeployments(sortDeploymentState(nextDeployments));
      setPm2Processes(nextPm2Processes);
      setSoftwarePackages(nextSoftwarePackages);
      setCaddyStatus(nextCaddyStatus);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  const refreshLogs = useCallback(async () => {
    setBusy("logs");
    setError(null);
    try {
      setLogResult(await readLog(logApp, logLines));
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [logApp, logLines]);

  const refreshDatabaseBackups = useCallback(async () => {
    setBusy("database-list");
    setError(null);
    try {
      const listing = await listDatabaseBackups();
      setDatabaseBackupDir(listing.backupDir);
      setDatabaseBackupFiles(listing.files);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  const refreshSoftwarePackages = useCallback(async () => {
    setBusy("software-list");
    setError(null);
    try {
      setSoftwarePackages(await listSoftwarePackages());
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  const refreshCaddyStatus = useCallback(async () => {
    setBusy("caddy-status");
    setError(null);
    try {
      const [nextCaddyStatus, nextPm2Processes] = await Promise.all([getCaddyStatus(), listPm2Processes()]);
      setCaddyStatus(nextCaddyStatus);
      setPm2Processes(nextPm2Processes);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  useEffect(() => {
    if (!isTauriRuntime() || import.meta.env.DEV) return;

    let disposed = false;
    void checkManagerUpdate()
      .then((update) => {
        if (disposed || !update) return;
        setManagerUpdate(update);
        setNotice(`Manager update v${update.version} is available.`);
      })
      .catch(() => {
        // A background update check should not interrupt VPS operations.
      });

    return () => {
      disposed = true;
    };
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) return;

    let disposed = false;
    let unlistenFn: (() => void) | null = null;
    void listen<DeployProgressEvent>("deploy-progress", (event) => {
      if (disposed) return;
      setDeploySteps((steps) => applyDeployProgressEvent(steps, event.payload));
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
      } else {
        unlistenFn = unlisten;
      }
    });

    return () => {
      disposed = true;
      unlistenFn?.();
    };
  }, []);

  useEffect(() => {
    if (activeTab === "logs") {
      void refreshLogs();
    }
  }, [activeTab, refreshLogs]);

  useEffect(() => {
    if (activeTab === "database") {
      void refreshDatabaseBackups();
    }
  }, [activeTab, refreshDatabaseBackups]);

  useEffect(() => {
    if (activeTab === "software") {
      void refreshSoftwarePackages();
    }
  }, [activeTab, refreshSoftwarePackages]);

  useEffect(() => {
    if (activeTab === "caddy") {
      void refreshCaddyStatus();
    }
  }, [activeTab, refreshCaddyStatus]);

  const dismissFeedback = useCallback(() => {
    setNotice(null);
    setError(null);
  }, []);

  const handleBrowsePackage = useCallback(async () => {
    const selected = await pickZipPackage();
    if (selected) {
      setPackagePath(selected);
      setValidation(null);
    }
  }, []);

  const handleValidatePackage = useCallback(async () => {
    if (!packagePath.trim()) return;
    setBusy("validate");
    setError(null);
    setNotice(null);
    try {
      setDeploySteps(markDeployStep(createDeploySteps(), "validate", "running"));
      const result = await validatePackage(packagePath.trim());
      setValidation(result);
      setDeploySteps(markDeployStep(createDeploySteps(), "validate", result.valid ? "done" : "failed"));
      setNotice(result.valid ? "Package is valid." : "Package is missing required files.");
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [packagePath]);

  const handleDeploy = useCallback(async () => {
    if (!packagePath.trim()) return;
    const runningSteps = createDeploySteps(settings);
    setDeploySteps(markDeployStep(runningSteps, "validate", "running"));
    setBusy("deploy");
    setError(null);
    setNotice(null);
    try {
      const result = await deployPackage(packagePath.trim(), settings);
      const postDeploySummary = result.postDeploy
        .filter((step) => !step.skipped)
        .map((step) => step.message)
        .join(" ");
      setNotice([postDeploySummary, result.pm2.message].filter(Boolean).join(" "));
      setDeploySteps(buildDeployStepsFromResult(settings, result));
      setValidation(null);
      await refreshAll();
    } catch (err) {
      setDeploySteps((steps) => failActiveDeployStep(steps.length ? steps : runningSteps));
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [packagePath, refreshAll, settings]);

  const handleSaveSettings = useCallback(async () => {
    setBusy("save");
    setError(null);
    setNotice(null);
    try {
      const saved = await saveSettings(settings);
      setSettings(saved);
      setNotice("Settings saved.");
      await refreshAll();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [refreshAll, settings]);

  const handleCheckManagerUpdate = useCallback(async () => {
    setBusy("manager-update-check");
    setError(null);
    setNotice(null);
    setManagerUpdateProgress(null);
    try {
      const update = await checkManagerUpdate();
      setManagerUpdate(update);
      setNotice(update ? `Manager update v${update.version} is available.` : "Manager is up to date.");
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  const handleInstallManagerUpdate = useCallback(async () => {
    if (!managerUpdate) return;
    const confirmed = window.confirm(
      `Install EduPortal_Control manager v${managerUpdate.version}? The manager window will close while the installer runs. Portal and WebApi stay online under PM2.`,
    );
    if (!confirmed) return;

    setBusy("manager-update-install");
    setError(null);
    setNotice(null);
    setManagerUpdateProgress({ downloadedBytes: 0, contentLength: null });
    try {
      await installManagerUpdate(setManagerUpdateProgress);
      setNotice("Manager update installed. Restarting the app.");
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [managerUpdate]);

  const handleRollback = useCallback(
    async (deployment: DeploymentRecord) => {
      setBusy(`rollback:${deployment.id}`);
      setError(null);
      setNotice(null);
      try {
        const result = await rollbackDeployment(deployment.id);
        setNotice(result.pm2.message);
        await refreshAll();
      } catch (err) {
        setError(errorMessage(err));
      } finally {
        setBusy(null);
      }
    },
    [refreshAll],
  );

  const handlePm2Action = useCallback(
    async (appName: ManagedAppName, action: Pm2Action) => {
      setBusy(pm2BusyKey(appName, action));
      setError(null);
      setNotice(null);
      try {
        const result = await controlPm2App(appName, action);
        if (!result.success) {
          throw new Error(result.message);
        }
        setNotice(result.message);
        await refreshAll();
      } catch (err) {
        setError(errorMessage(err));
      } finally {
        setBusy(null);
      }
    },
    [refreshAll],
  );

  const handleOpenLog = useCallback((appName: ManagedAppName) => {
    setLogApp(appName);
    setActiveTab("logs");
  }, []);

  const handleBrowseCaddyZip = useCallback(async () => {
    const selected = await pickCaddyZipPackage();
    if (selected) {
      setCaddyZipPath(selected);
      setCaddyInstallResult(null);
    }
  }, []);

  const handleInstallCaddyZip = useCallback(async () => {
    if (!caddyZipPath.trim()) return;
    setBusy("caddy-install");
    setError(null);
    setNotice(null);
    setCaddyInstallResult(null);
    try {
      const result = await installCaddyZip(caddyZipPath.trim(), settings);
      setCaddyInstallResult(result);
      setCaddyStatus(result.status);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
      await refreshCaddyStatus();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [caddyZipPath, refreshCaddyStatus, settings]);

  const handleInstallBundledCaddy = useCallback(async () => {
    setBusy("caddy-install-bundled");
    setError(null);
    setNotice(null);
    setCaddyInstallResult(null);
    try {
      const result = await installBundledCaddy(settings);
      setCaddyInstallResult(result);
      setCaddyStatus(result.status);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
      await refreshCaddyStatus();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [refreshCaddyStatus, settings]);

  const handleConfigureCaddyFirewall = useCallback(async () => {
    setBusy("caddy-firewall");
    setError(null);
    setNotice(null);
    setCaddyFirewallResult(null);
    try {
      const result = await configureCaddyFirewall();
      setCaddyFirewallResult(result);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  const handleApplyCaddyConfig = useCallback(async () => {
    setBusy("caddy-apply");
    setError(null);
    setNotice(null);
    setCaddyApplyResult(null);
    try {
      const result = await applyCaddyConfig(settings);
      setCaddyApplyResult(result);
      setCaddyStatus(result.status);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
      await refreshAll();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [refreshAll, settings]);

  const handleApplyCaddyPublishTestConfig = useCallback(async () => {
    setBusy("caddy-publish-test");
    setError(null);
    setNotice(null);
    setCaddyApplyResult(null);
    try {
      const result = await applyCaddyPublishTestConfig(settings);
      setCaddyApplyResult(result);
      setCaddyStatus(result.status);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(`${result.message} Test with the VPS public IP over HTTP while DNS still points elsewhere.`);
      await refreshCaddyStatus();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [refreshCaddyStatus, settings]);

  const handleInstallSoftwarePackage = useCallback(
    async (packageId: SoftwarePackageId) => {
      setBusy(`software:${packageId}`);
      setError(null);
      setNotice(null);
      setSoftwareInstallResult(null);
      try {
        const result = await installSoftwarePackage(packageId);
        setSoftwareInstallResult(result);
        if (!result.success) {
          throw new Error(result.message);
        }
        setNotice(result.message);
        await refreshAll();
      } catch (err) {
        setError(errorMessage(err));
        try {
          setSoftwarePackages(await listSoftwarePackages());
        } catch {
          // Keep the original installation error visible.
        }
      } finally {
        setBusy(null);
      }
    },
    [refreshAll],
  );

  const handleRunMigration = useCallback(async () => {
    setBusy("migration");
    setError(null);
    setNotice(null);
    try {
      const result = await runMigration();
      setMigrationResult(result);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, []);

  const handleBrowseDatabaseBackupDir = useCallback(async () => {
    const selected = await pickDatabaseBackupDirectory();
    if (!selected) return;
    setSettings((current) => ({
      ...current,
      database: {
        ...current.database,
        backupDir: selected,
      },
    }));
  }, []);

  const handleBrowseDatabaseRestoreFile = useCallback(async () => {
    const selected = await pickDatabaseBackupFile();
    if (selected) {
      setDatabaseRestorePath(selected);
    }
  }, []);

  const handleRunDatabaseBackup = useCallback(async () => {
    setBusy("database-backup");
    setError(null);
    setNotice(null);
    setDatabaseBackupResult(null);
    try {
      const result = await runDatabaseBackup(settings);
      setDatabaseBackupResult(result);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
      await refreshDatabaseBackups();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [refreshDatabaseBackups, settings]);

  const handleRestoreDatabaseBackup = useCallback(async () => {
    const backupPath = databaseRestorePath.trim();
    if (!backupPath) return;
    const confirmed = window.confirm(
      `Restore ${backupPath} into PostgreSQL database ${settings.database.database}? Existing objects may be replaced.`,
    );
    if (!confirmed) return;

    setBusy("database-restore");
    setError(null);
    setNotice(null);
    setDatabaseRestoreResult(null);
    try {
      const result = await restoreDatabaseBackup(backupPath, settings);
      setDatabaseRestoreResult(result);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [databaseRestorePath, settings]);

  const handleConfigureDatabaseSchedule = useCallback(async () => {
    setBusy("database-schedule");
    setError(null);
    setNotice(null);
    setDatabaseScheduleResult(null);
    try {
      const result = await configureDatabaseBackupSchedule(settings);
      setDatabaseScheduleResult(result);
      if (!result.success) {
        throw new Error(result.message);
      }
      setNotice(result.message);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(null);
    }
  }, [settings]);

  return {
    activeDeployment,
    activeTab,
    busy,
    caddyApplyResult,
    caddyFirewallResult,
    caddyInstallResult,
    caddyProcess,
    caddyStatus,
    caddyZipPath,
    databaseBackupDir,
    databaseBackupFiles,
    databaseBackupResult,
    databaseRestorePath,
    databaseRestoreResult,
    databaseScheduleResult,
    deploySteps,
    deployments,
    error,
    logApp,
    logLines,
    logResult,
    managerUpdate,
    managerUpdateProgress,
    migrationResult,
    notice,
    packagePath,
    pm2Processes,
    settings,
    softwareInstallResult,
    softwarePackages,
    status,
    validation,
    dismissFeedback,
    handleBrowseDatabaseBackupDir,
    handleBrowseCaddyZip,
    handleBrowseDatabaseRestoreFile,
    handleBrowsePackage,
    handleConfigureDatabaseSchedule,
    handleConfigureCaddyFirewall,
    handleApplyCaddyConfig,
    handleApplyCaddyPublishTestConfig,
    handleDeploy,
    handleInstallBundledCaddy,
    handleInstallCaddyZip,
    handleCheckManagerUpdate,
    handleInstallManagerUpdate,
    handleInstallSoftwarePackage,
    handleOpenLog,
    handlePm2Action,
    handleRestoreDatabaseBackup,
    handleRollback,
    handleRunDatabaseBackup,
    handleRunMigration,
    handleSaveSettings,
    handleValidatePackage,
    refreshAll,
    refreshCaddyStatus,
    refreshDatabaseBackups,
    refreshLogs,
    refreshSoftwarePackages,
    setActiveTab,
    setCaddyZipPath,
    setDatabaseRestorePath,
    setLogApp,
    setLogLines,
    setPackagePath,
    setSettings,
  };
}
