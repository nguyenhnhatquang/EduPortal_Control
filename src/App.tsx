import { useAppController } from "./application/useAppController";
import { Feedback } from "./presentation/components/Feedback";
import { Sidebar } from "./presentation/layout/Sidebar";
import { Topbar } from "./presentation/layout/Topbar";
import { futureTabs, subtitleForTab, titleForTab } from "./presentation/navigation";
import { DatabaseTab } from "./presentation/tabs/DatabaseTab";
import { CaddyTab } from "./presentation/tabs/CaddyTab";
import { DeploymentsTab } from "./presentation/tabs/DeploymentsTab";
import { LogsTab } from "./presentation/tabs/LogsTab";
import { OverviewTab } from "./presentation/tabs/OverviewTab";
import { PlaceholderTab } from "./presentation/tabs/PlaceholderTab";
import { SettingsTab } from "./presentation/tabs/SettingsTab";
import { SoftwareTab } from "./presentation/tabs/SoftwareTab";

function App() {
  const controller = useAppController();

  return (
    <div className="app-shell">
      <Sidebar activeTab={controller.activeTab} status={controller.status} onSelectTab={controller.setActiveTab} />

      <main className="main">
        <Topbar
          title={titleForTab(controller.activeTab)}
          subtitle={subtitleForTab(controller.activeTab)}
          version={controller.status?.appVersion ?? "0.1.0"}
          refreshing={controller.busy === "refresh"}
          managerUpdate={controller.managerUpdate}
          updateProgress={controller.managerUpdateProgress}
          checkingUpdate={controller.busy === "manager-update-check"}
          installingUpdate={controller.busy === "manager-update-install"}
          onRefresh={controller.refreshAll}
          onCheckUpdate={controller.handleCheckManagerUpdate}
          onInstallUpdate={controller.handleInstallManagerUpdate}
        />

        <Feedback notice={controller.notice} error={controller.error} onDismiss={controller.dismissFeedback} />

        {controller.activeTab === "overview" && (
          <OverviewTab
            status={controller.status}
            deployments={controller.deployments}
            activeDeployment={controller.activeDeployment}
            settings={controller.settings}
            pm2Processes={controller.pm2Processes}
            busy={controller.busy}
            onPm2Action={controller.handlePm2Action}
            onOpenLog={controller.handleOpenLog}
          />
        )}

        {controller.activeTab === "deployments" && (
          <DeploymentsTab
            packagePath={controller.packagePath}
            setPackagePath={controller.setPackagePath}
            validation={controller.validation}
            portalReleaseCheck={controller.portalReleaseCheck}
            portalReleaseReady={controller.settings.portalRelease.enabled && Boolean(controller.settings.portalRelease.token.trim())}
            deploySteps={controller.deploySteps}
            deployments={controller.deployments}
            busy={controller.busy}
            onBrowse={controller.handleBrowsePackage}
            onValidate={controller.handleValidatePackage}
            onDeploy={controller.handleDeploy}
            onCheckPortalRelease={controller.handleCheckPortalRelease}
            onDeployPortalRelease={controller.handleDeployPortalRelease}
            onRollback={controller.handleRollback}
          />
        )}

        {controller.activeTab === "logs" && (
          <LogsTab
            logApp={controller.logApp}
            setLogApp={controller.setLogApp}
            logLines={controller.logLines}
            setLogLines={controller.setLogLines}
            logResult={controller.logResult}
            busy={controller.busy}
            onRefresh={controller.refreshLogs}
          />
        )}

        {controller.activeTab === "database" && (
          <DatabaseTab
            settings={controller.settings}
            setSettings={controller.setSettings}
            migrationResult={controller.migrationResult}
            backupFiles={controller.databaseBackupFiles}
            backupDir={controller.databaseBackupDir}
            backupResult={controller.databaseBackupResult}
            restoreResult={controller.databaseRestoreResult}
            scheduleResult={controller.databaseScheduleResult}
            restorePath={controller.databaseRestorePath}
            setRestorePath={controller.setDatabaseRestorePath}
            busy={controller.busy}
            onRunMigration={controller.handleRunMigration}
            onRunBackup={controller.handleRunDatabaseBackup}
            onRestoreBackup={controller.handleRestoreDatabaseBackup}
            onConfigureSchedule={controller.handleConfigureDatabaseSchedule}
            onBrowseRestoreFile={controller.handleBrowseDatabaseRestoreFile}
            onRefreshBackups={controller.refreshDatabaseBackups}
          />
        )}

        {controller.activeTab === "caddy" && (
          <CaddyTab
            settings={controller.settings}
            setSettings={controller.setSettings}
            zipPath={controller.caddyZipPath}
            setZipPath={controller.setCaddyZipPath}
            status={controller.caddyStatus}
            process={controller.caddyProcess}
            installResult={controller.caddyInstallResult}
            applyResult={controller.caddyApplyResult}
            firewallResult={controller.caddyFirewallResult}
            busy={controller.busy}
            pm2Enabled={Boolean(controller.status?.pm2ExecutionEnabled)}
            onBrowseZip={controller.handleBrowseCaddyZip}
            onInstall={controller.handleInstallCaddyZip}
            onInstallBundled={controller.handleInstallBundledCaddy}
            onConfigureFirewall={controller.handleConfigureCaddyFirewall}
            onApply={controller.handleApplyCaddyConfig}
            onApplyPublishTest={controller.handleApplyCaddyPublishTestConfig}
            onPm2Action={controller.handlePm2Action}
            onOpenLog={controller.handleOpenLog}
            onRefresh={controller.refreshCaddyStatus}
          />
        )}

        {controller.activeTab === "software" && (
          <SoftwareTab
            packages={controller.softwarePackages}
            installResult={controller.softwareInstallResult}
            busy={controller.busy}
            onInstall={controller.handleInstallSoftwarePackage}
            onRefresh={controller.refreshSoftwarePackages}
          />
        )}

        {controller.activeTab === "settings" && (
          <SettingsTab
            settings={controller.settings}
            setSettings={controller.setSettings}
            busy={controller.busy}
            onBrowseBackupDir={controller.handleBrowseDatabaseBackupDir}
            onSave={controller.handleSaveSettings}
          />
        )}

        {futureTabs.some((tab) => tab.id === controller.activeTab) && (
          <PlaceholderTab label={titleForTab(controller.activeTab)} />
        )}
      </main>
    </div>
  );
}

export default App;
