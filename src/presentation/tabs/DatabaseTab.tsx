import {
  Archive,
  CheckCircle2,
  Clock,
  Database,
  FolderOpen,
  Loader2,
  RefreshCw,
  RotateCcw,
  XCircle,
} from "lucide-react";
import { backupScheduleDays, backupScheduleFrequencies } from "../../domain/settings/defaults";
import { formatBytes, formatDate } from "../../shared/formatters";
import type {
  DatabaseBackupFile,
  DatabaseBackupScheduleDay,
  DatabaseBackupScheduleFrequency,
  DatabaseCommandResult,
  MigrationResult,
  Settings,
} from "../../types";

interface DatabaseTabProps {
  settings: Settings;
  setSettings: (settings: Settings) => void;
  migrationResult: MigrationResult | null;
  backupFiles: DatabaseBackupFile[];
  backupDir: string;
  backupResult: DatabaseCommandResult | null;
  restoreResult: DatabaseCommandResult | null;
  scheduleResult: DatabaseCommandResult | null;
  restorePath: string;
  setRestorePath: (value: string) => void;
  busy: string | null;
  onRunMigration: () => void;
  onRunBackup: () => void;
  onRestoreBackup: () => void;
  onConfigureSchedule: () => void;
  onBrowseRestoreFile: () => void;
  onRefreshBackups: () => void;
}

export function DatabaseTab({
  settings,
  setSettings,
  migrationResult,
  backupFiles,
  backupDir,
  backupResult,
  restoreResult,
  scheduleResult,
  restorePath,
  setRestorePath,
  busy,
  onRunMigration,
  onRunBackup,
  onRestoreBackup,
  onConfigureSchedule,
  onBrowseRestoreFile,
  onRefreshBackups,
}: DatabaseTabProps) {
  const database = settings.database;
  const schedule = database.backupSchedule;

  function updateDatabase(next: Partial<typeof database>) {
    setSettings({
      ...settings,
      database: {
        ...database,
        ...next,
      },
    });
  }

  function updateSchedule(next: Partial<typeof schedule>) {
    updateDatabase({
      backupSchedule: {
        ...schedule,
        ...next,
      },
    });
  }

  return (
    <section className="stack">
      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>PostgreSQL Database</h2>
            <span>{database.host}:{database.port}</span>
          </div>
          <Database size={20} />
        </div>

        <dl className="details database-summary">
          <dt>Engine</dt>
          <dd>PostgreSQL</dd>
          <dt>Database</dt>
          <dd>{database.database}</dd>
          <dt>Backup dir</dt>
          <dd>{backupDir}</dd>
          <dt>Schedule</dt>
          <dd>
            {schedule.enabled
              ? schedule.frequency === "weekly"
                ? `Weekly on ${schedule.dayOfWeek} at ${schedule.time}`
                : `Daily at ${schedule.time}`
              : "Disabled"}
          </dd>
        </dl>
      </div>

      <div className="database-action-grid">
        <div className="panel">
          <div className="panel-heading">
            <div>
              <h2>Backup</h2>
              <span>{database.backupDir}</span>
            </div>
            <Archive size={20} />
          </div>

          <dl className="details">
            <dt>Directory</dt>
            <dd>{database.backupDir}</dd>
            <dt>Retention</dt>
            <dd>{database.backupRetention} day(s)</dd>
          </dl>

          <div className="button-row">
            <button className="primary-button" onClick={onRunBackup} disabled={busy === "database-backup"}>
              {busy === "database-backup" ? <Loader2 className="spin" size={17} /> : <Archive size={17} />}
              Run Backup
            </button>
          </div>

          {backupResult && <DatabaseCommandResultPanel result={backupResult} />}
        </div>

        <div className="panel">
          <div className="panel-heading">
            <div>
              <h2>Restore</h2>
              <span>{database.database}</span>
            </div>
            <RotateCcw size={20} />
          </div>

          <div className="database-form-grid">
            <label className="span-2">
              <span>Backup file</span>
              <input
                value={restorePath}
                onChange={(event) => setRestorePath(event.target.value)}
                placeholder="C:\\deploy\\backups\\postgresql\\backup.dump"
              />
            </label>
            <button className="secondary-button field-button" onClick={onBrowseRestoreFile}>
              <FolderOpen size={17} />
              Browse
            </button>
            <button
              className="danger-button field-button"
              onClick={onRestoreBackup}
              disabled={!restorePath.trim() || busy === "database-restore"}
            >
              {busy === "database-restore" ? <Loader2 className="spin" size={17} /> : <RotateCcw size={17} />}
              Restore
            </button>
          </div>

          {restoreResult && <DatabaseCommandResultPanel result={restoreResult} />}
        </div>
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Scheduled Backup</h2>
            <span>Windows Task Scheduler</span>
          </div>
          <Clock size={20} />
        </div>

        <div className="database-schedule-grid">
          <label className="toggle-row">
            <input
              type="checkbox"
              checked={schedule.enabled}
              onChange={(event) => updateSchedule({ enabled: event.target.checked })}
            />
            <span>Enable scheduled PostgreSQL backup</span>
          </label>
          <label>
            <span>Frequency</span>
            <select
              value={schedule.frequency}
              onChange={(event) =>
                updateSchedule({ frequency: event.target.value as DatabaseBackupScheduleFrequency })
              }
            >
              {backupScheduleFrequencies.map((frequency) => (
                <option value={frequency} key={frequency}>
                  {frequency === "daily" ? "Daily" : "Weekly"}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span>Time</span>
            <input
              type="time"
              value={schedule.time}
              onChange={(event) => updateSchedule({ time: event.target.value })}
            />
          </label>
          <label>
            <span>Day</span>
            <select
              value={schedule.dayOfWeek}
              disabled={schedule.frequency !== "weekly"}
              onChange={(event) => updateSchedule({ dayOfWeek: event.target.value as DatabaseBackupScheduleDay })}
            >
              {backupScheduleDays.map((day) => (
                <option value={day} key={day}>
                  {day}
                </option>
              ))}
            </select>
          </label>
          <button
            className="primary-button field-button"
            onClick={onConfigureSchedule}
            disabled={busy === "database-schedule"}
          >
            {busy === "database-schedule" ? <Loader2 className="spin" size={17} /> : <Clock size={17} />}
            Apply Schedule
          </button>
        </div>

        {scheduleResult && <DatabaseCommandResultPanel result={scheduleResult} />}
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Backup Files</h2>
            <span>{backupFiles.length} file(s)</span>
          </div>
          <button className="icon-button" onClick={onRefreshBackups} disabled={busy === "database-list"} title="Refresh">
            {busy === "database-list" ? <Loader2 className="spin" size={18} /> : <RefreshCw size={18} />}
          </button>
        </div>

        <DatabaseBackupTable files={backupFiles} busy={busy} onRestore={setRestorePath} />
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Migration</h2>
            <span>{settings.migrationUrl}</span>
          </div>
          <Database size={20} />
        </div>

        <div className="migration-panel">
          <dl className="details">
            <dt>Endpoint</dt>
            <dd>{settings.migrationUrl}</dd>
            <dt>Timeout</dt>
            <dd>{settings.migrationTimeoutSecs}s</dd>
            <dt>Header</dt>
            <dd>X-Migration-Key</dd>
          </dl>

          <button className="primary-button" onClick={onRunMigration} disabled={busy === "migration"}>
            {busy === "migration" ? <Loader2 className="spin" size={17} /> : <Database size={17} />}
            Run Migration
          </button>
        </div>
      </div>

      {migrationResult && (
        <div className={`panel migration-result ${migrationResult.success ? "valid" : "invalid"}`}>
          <div className="panel-heading">
            <div>
              <h2>{migrationResult.message}</h2>
              <span>HTTP {migrationResult.statusCode ?? "-"}</span>
            </div>
            {migrationResult.success ? <CheckCircle2 size={20} /> : <XCircle size={20} />}
          </div>
          <pre>{migrationResult.body || "No response body."}</pre>
        </div>
      )}
    </section>
  );
}

function DatabaseCommandResultPanel({ result }: { result: DatabaseCommandResult }) {
  return (
    <div className={`operation-result ${result.success ? "valid" : "invalid"}`}>
      <div className="operation-result-heading">
        {result.success ? <CheckCircle2 size={17} /> : <XCircle size={17} />}
        <strong>{result.message}</strong>
      </div>
      <dl className="details">
        <dt>Path</dt>
        <dd>{result.path ?? "-"}</dd>
        <dt>Command</dt>
        <dd>{result.command}</dd>
      </dl>
      {(result.stdout || result.stderr) && <pre>{[result.stdout, result.stderr].filter(Boolean).join("\n")}</pre>}
    </div>
  );
}

function DatabaseBackupTable({
  files,
  busy,
  onRestore,
}: {
  files: DatabaseBackupFile[];
  busy: string | null;
  onRestore: (path: string) => void;
}) {
  if (files.length === 0) {
    return <div className="empty-state">No PostgreSQL backup files yet.</div>;
  }

  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>File</th>
            <th>Modified</th>
            <th>Size</th>
            <th>Path</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {files.map((file) => (
            <tr key={file.path}>
              <td>{file.fileName}</td>
              <td>{formatDate(file.modifiedAt)}</td>
              <td>{formatBytes(file.sizeBytes)}</td>
              <td className="path-cell">{file.path}</td>
              <td className="actions-cell">
                <button
                  className="secondary-button compact"
                  disabled={busy === "database-restore"}
                  onClick={() => onRestore(file.path)}
                >
                  <RotateCcw size={16} />
                  Use
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
