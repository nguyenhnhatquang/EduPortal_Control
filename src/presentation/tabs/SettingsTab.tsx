import { Bot, Database, FolderOpen, Loader2, Save, Settings as SettingsIcon, Terminal } from "lucide-react";
import { EnvEditor } from "../components/EnvEditor";
import type { Settings } from "../../types";

interface SettingsTabProps {
  settings: Settings;
  setSettings: (settings: Settings) => void;
  busy: string | null;
  onBrowseBackupDir: () => void;
  onSave: () => void;
}

export function SettingsTab({ settings, setSettings, busy, onBrowseBackupDir, onSave }: SettingsTabProps) {
  const database = settings.database;
  const telegramBot = settings.telegramBot;

  function updateDatabase(next: Partial<typeof database>) {
    setSettings({
      ...settings,
      database: {
        ...database,
        ...next,
      },
    });
  }

  function updatePortalAssetCopy(next: Partial<typeof settings.portalAssetCopy>) {
    setSettings({
      ...settings,
      portalAssetCopy: {
        ...settings.portalAssetCopy,
        ...next,
      },
    });
  }

  function updatePortalRelease(next: Partial<typeof settings.portalRelease>) {
    setSettings({
      ...settings,
      portalRelease: {
        ...settings.portalRelease,
        ...next,
      },
    });
  }

  function updateTelegramBot(next: Partial<typeof settings.telegramBot>) {
    setSettings({
      ...settings,
      telegramBot: {
        ...settings.telegramBot,
        ...next,
      },
    });
  }

  return (
    <section className="stack settings-page">
      <div className="settings-overview">
        <div className="settings-summary-item">
          <span>Deploy root</span>
          <strong>{settings.deployRoot}</strong>
        </div>
        <div className="settings-summary-item">
          <span>Portal post deploy</span>
          <strong>
            {settings.portalInstallDependencies ? "npm install" : "Skip npm install"}
            {settings.portalAssetCopy.enabled ? " + copy assets" : ""}
          </strong>
        </div>
        <div className="settings-summary-item">
          <span>Portal release</span>
          <strong>
            {settings.portalRelease.enabled
              ? `${settings.portalRelease.owner}/${settings.portalRelease.repo}`
              : "Disabled"}
          </strong>
        </div>
        <div className="settings-summary-item">
          <span>Backend endpoint</span>
          <strong>{settings.migrationUrl}</strong>
        </div>
        <div className="settings-summary-item">
          <span>PostgreSQL target</span>
          <strong>{database.host}:{database.port}/{database.database}</strong>
        </div>
        <div className="settings-summary-item">
          <span>Telegram bot</span>
          <strong>{telegramBot.enabled ? "Enabled" : "Disabled"}</strong>
        </div>
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Deployment & Portal</h2>
            <span>Release location, retention, and Portal post-deploy steps</span>
          </div>
          <SettingsIcon size={20} />
        </div>

        <div className="settings-section-stack">
          <div className="settings-section">
            <div className="settings-section-title">
              <strong>Release storage</strong>
              <span>Controls where deployments are unpacked and how many releases stay available.</span>
            </div>

            <div className="settings-grid">
              <label>
                <span>Deploy root</span>
                <input
                  value={settings.deployRoot}
                  onChange={(event) => setSettings({ ...settings, deployRoot: event.target.value })}
                />
              </label>
              <label>
                <span>Retention</span>
                <input
                  type="number"
                  min={1}
                  max={50}
                  value={settings.retention}
                  onChange={(event) => setSettings({ ...settings, retention: Number(event.target.value) })}
                />
              </label>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-section-title">
              <strong>Portal post deploy</strong>
              <span>Runs after the package is copied and before PM2 reloads Portal.</span>
            </div>

            <div className="post-deploy-grid">
              <label className="toggle-row">
                <input
                  type="checkbox"
                  checked={settings.portalInstallDependencies}
                  onChange={(event) => setSettings({ ...settings, portalInstallDependencies: event.target.checked })}
                />
                <span>Run npm install --omit=dev in Portal</span>
              </label>

              <label className="toggle-row">
                <input
                  type="checkbox"
                  checked={settings.portalAssetCopy.enabled}
                  onChange={(event) => updatePortalAssetCopy({ enabled: event.target.checked })}
                />
                <span>Copy folder into Portal build/client</span>
              </label>

              <label>
                <span>Copy source, absolute or relative to deploy root</span>
                <input
                  value={settings.portalAssetCopy.source}
                  onChange={(event) => updatePortalAssetCopy({ source: event.target.value })}
                />
              </label>
              <label>
                <span>Copy destination, relative to Portal</span>
                <input
                  value={settings.portalAssetCopy.destination}
                  onChange={(event) => updatePortalAssetCopy({ destination: event.target.value })}
                />
              </label>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-section-title">
              <strong>Portal release source</strong>
              <span>Private GitHub release checked by the Deployments tab.</span>
            </div>

            <div className="release-settings-grid">
              <label className="toggle-row">
                <input
                  type="checkbox"
                  checked={settings.portalRelease.enabled}
                  onChange={(event) => updatePortalRelease({ enabled: event.target.checked })}
                />
                <span>Enable release updates</span>
              </label>
              <label>
                <span>Owner</span>
                <input
                  value={settings.portalRelease.owner}
                  onChange={(event) => updatePortalRelease({ owner: event.target.value })}
                />
              </label>
              <label>
                <span>Repository</span>
                <input
                  value={settings.portalRelease.repo}
                  onChange={(event) => updatePortalRelease({ repo: event.target.value })}
                />
              </label>
              <label className="span-2">
                <span>GitHub PAT</span>
                <input
                  type="password"
                  value={settings.portalRelease.token}
                  onChange={(event) => updatePortalRelease({ token: event.target.value })}
                />
              </label>
              <label>
                <span>Asset prefix</span>
                <input
                  value={settings.portalRelease.assetNamePrefix}
                  onChange={(event) => updatePortalRelease({ assetNamePrefix: event.target.value })}
                />
              </label>
              <label>
                <span>Asset suffix</span>
                <input
                  value={settings.portalRelease.assetNameSuffix}
                  onChange={(event) => updatePortalRelease({ assetNameSuffix: event.target.value })}
                />
              </label>
            </div>
          </div>
        </div>
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Remote Admin</h2>
            <span>Telegram bot access for controlled VPS operations</span>
          </div>
          <Bot size={20} />
        </div>

        <div className="settings-section-stack">
          <div className="settings-section">
            <div className="settings-section-title">
              <strong>Telegram bot</strong>
              <span>Uses numeric Telegram IDs for the bot allowlist.</span>
            </div>

            <div className="release-settings-grid">
              <label className="toggle-row">
                <input
                  type="checkbox"
                  checked={telegramBot.enabled}
                  onChange={(event) => updateTelegramBot({ enabled: event.target.checked })}
                />
                <span>Enable Telegram bot</span>
              </label>
              <label className="span-2">
                <span>Bot token</span>
                <input
                  type="password"
                  value={telegramBot.token}
                  onChange={(event) => updateTelegramBot({ token: event.target.value })}
                />
              </label>
              <label>
                <span>Allowed user IDs</span>
                <input
                  value={telegramBot.allowedUserIds}
                  onChange={(event) => updateTelegramBot({ allowedUserIds: event.target.value })}
                />
              </label>
              <label className="span-2">
                <span>Allowed chat IDs</span>
                <input
                  value={telegramBot.allowedChatIds}
                  onChange={(event) => updateTelegramBot({ allowedChatIds: event.target.value })}
                />
              </label>
              <label>
                <span>Last user ID</span>
                <input value={telegramBot.lastUserId} readOnly />
              </label>
              <label>
                <span>Last chat ID</span>
                <input value={telegramBot.lastChatId} readOnly />
              </label>
            </div>
          </div>
        </div>
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Backend & Database</h2>
            <span>Migration endpoint, PostgreSQL connection, and backup storage</span>
          </div>
          <Database size={20} />
        </div>

        <div className="settings-section-stack">
          <div className="settings-section">
            <div className="settings-section-title">
              <strong>Migration endpoint</strong>
              <span>Used when the Database tab runs the backend migration command.</span>
            </div>

            <div className="migration-settings-grid">
              <label>
                <span>Migration URL</span>
                <input
                  value={settings.migrationUrl}
                  onChange={(event) => setSettings({ ...settings, migrationUrl: event.target.value })}
                />
              </label>
              <label>
                <span>Timeout seconds</span>
                <input
                  type="number"
                  min={5}
                  max={600}
                  value={settings.migrationTimeoutSecs}
                  onChange={(event) =>
                    setSettings({ ...settings, migrationTimeoutSecs: Number(event.target.value) })
                  }
                />
              </label>
              <label className="span-2">
                <span>Migration key</span>
                <input
                  type="password"
                  value={settings.migrationKey}
                  onChange={(event) => setSettings({ ...settings, migrationKey: event.target.value })}
                />
              </label>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-section-title">
              <strong>PostgreSQL connection</strong>
              <span>Shared by backup, restore, and database maintenance actions.</span>
            </div>

            <div className="database-settings-grid">
              <label>
                <span>Host</span>
                <input value={database.host} onChange={(event) => updateDatabase({ host: event.target.value })} />
              </label>
              <label>
                <span>Port</span>
                <input
                  type="number"
                  min={1}
                  max={65535}
                  value={database.port}
                  onChange={(event) => updateDatabase({ port: Number(event.target.value) || 5432 })}
                />
              </label>
              <label>
                <span>Database</span>
                <input
                  value={database.database}
                  onChange={(event) => updateDatabase({ database: event.target.value })}
                />
              </label>
              <label>
                <span>Username</span>
                <input
                  value={database.username}
                  onChange={(event) => updateDatabase({ username: event.target.value })}
                />
              </label>
              <label>
                <span>Password</span>
                <input
                  type="password"
                  value={database.password}
                  onChange={(event) => updateDatabase({ password: event.target.value })}
                />
              </label>
              <label>
                <span>PostgreSQL bin directory</span>
                <input
                  value={database.binDir}
                  onChange={(event) => updateDatabase({ binDir: event.target.value })}
                  placeholder="C:\\Program Files\\PostgreSQL\\18\\bin"
                />
              </label>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-section-title">
              <strong>Backup policy</strong>
              <span>Used by manual backups and the scheduled PostgreSQL backup task.</span>
            </div>

            <div className="backup-settings-grid">
              <div className="field-group">
                <span>Backup directory, absolute or relative to deploy root</span>
                <div className="input-button-row">
                  <input
                    value={database.backupDir}
                    onChange={(event) => updateDatabase({ backupDir: event.target.value })}
                  />
                  <button className="secondary-button" onClick={onBrowseBackupDir}>
                    <FolderOpen size={17} />
                    Browse
                  </button>
                </div>
              </div>
              <label>
                <span>Retention</span>
                <input
                  type="number"
                  min={1}
                  max={365}
                  value={database.backupRetention}
                  onChange={(event) => updateDatabase({ backupRetention: Number(event.target.value) || 1 })}
                />
              </label>
            </div>
          </div>
        </div>
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Runtime Environment</h2>
            <span>Environment variables written into each deployment config</span>
          </div>
          <Terminal size={20} />
        </div>

        <div className="settings-env-grid">
          <EnvEditor
            title="Portal Env"
            env={settings.portalEnv}
            onChange={(portalEnv) => setSettings({ ...settings, portalEnv })}
          />
          <EnvEditor
            title="WebApi Env"
            env={settings.webApiEnv}
            onChange={(webApiEnv) => setSettings({ ...settings, webApiEnv })}
          />
        </div>
      </div>

      <div className="save-bar">
        <button className="primary-button" onClick={onSave} disabled={busy === "save"}>
          {busy === "save" ? <Loader2 className="spin" size={17} /> : <Save size={17} />}
          Save Settings
        </button>
      </div>
    </section>
  );
}
