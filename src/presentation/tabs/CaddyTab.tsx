import {
  CheckCircle2,
  Download,
  FileCog,
  FolderOpen,
  Loader2,
  Network,
  Play,
  RefreshCw,
  Save,
  ScrollText,
  Square,
  XCircle,
} from "lucide-react";
import { formatBytes, statusClassName } from "../../shared/formatters";
import type {
  CaddyCommandResult,
  CaddyStatus,
  ManagedAppName,
  Pm2Action,
  Pm2Process,
  Settings,
} from "../../types";

interface CaddyTabProps {
  settings: Settings;
  setSettings: (settings: Settings) => void;
  zipPath: string;
  setZipPath: (value: string) => void;
  status: CaddyStatus | null;
  process: Pm2Process | null;
  installResult: CaddyCommandResult | null;
  applyResult: CaddyCommandResult | null;
  busy: string | null;
  pm2Enabled: boolean;
  onBrowseZip: () => void;
  onInstall: () => void;
  onInstallBundled: () => void;
  onApply: () => void;
  onApplyPublishTest: () => void;
  onPm2Action: (appName: ManagedAppName, action: Pm2Action) => void;
  onOpenLog: (appName: ManagedAppName) => void;
  onRefresh: () => void;
}

export function CaddyTab({
  settings,
  setSettings,
  zipPath,
  setZipPath,
  status,
  process,
  installResult,
  applyResult,
  busy,
  pm2Enabled,
  onBrowseZip,
  onInstall,
  onInstallBundled,
  onApply,
  onApplyPublishTest,
  onPm2Action,
  onOpenLog,
  onRefresh,
}: CaddyTabProps) {
  const caddy = settings.caddy;

  function updateCaddy(next: Partial<typeof caddy>) {
    setSettings({
      ...settings,
      caddy: {
        ...caddy,
        ...next,
      },
    });
  }

  return (
    <section className="stack">
      <div className="content-grid caddy-grid">
        <div className="panel">
          <div className="panel-heading">
            <div>
              <h2>Caddy Runtime</h2>
              <span>{status?.executablePath ?? "caddy.exe"}</span>
            </div>
            <button className="icon-button" onClick={onRefresh} disabled={busy === "caddy-status"} title="Refresh">
              {busy === "caddy-status" ? <Loader2 className="spin" size={18} /> : <RefreshCw size={18} />}
            </button>
          </div>

          <dl className="details caddy-details">
            <dt>Status</dt>
            <dd>
              <span className={`pill ${status?.installed ? "green" : "neutral"}`}>
                {status?.installed ? "Installed" : "Missing"}
              </span>
            </dd>
            <dt>Version</dt>
            <dd>{status?.version ?? status?.error ?? "-"}</dd>
            <dt>Install dir</dt>
            <dd>{status?.installDir ?? caddy.installDir}</dd>
            <dt>Caddyfile</dt>
            <dd>{status?.configPath ?? caddy.configPath}</dd>
          </dl>
        </div>

        <div className="panel">
          <div className="panel-heading">
            <div>
              <h2>Install Zip</h2>
              <span>{zipPath || "No zip selected"}</span>
            </div>
            <Download size={20} />
          </div>

          <div className="input-button-row">
            <input value={zipPath} onChange={(event) => setZipPath(event.target.value)} placeholder="C:\\Downloads\\caddy.zip" />
            <button className="secondary-button" onClick={onBrowseZip}>
              <FolderOpen size={17} />
              Browse
            </button>
          </div>

          <div className="button-row">
            <button className="primary-button" onClick={onInstall} disabled={!zipPath.trim() || busy === "caddy-install"}>
              {busy === "caddy-install" ? <Loader2 className="spin" size={17} /> : <Download size={17} />}
              Install
            </button>
            <button className="secondary-button" onClick={onInstallBundled} disabled={busy === "caddy-install-bundled"}>
              {busy === "caddy-install-bundled" ? <Loader2 className="spin" size={17} /> : <Download size={17} />}
              Install Bundled
            </button>
          </div>
        </div>
      </div>

      {installResult && <CaddyResultPanel result={installResult} />}

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Caddyfile</h2>
            <span>{caddy.configPath}</span>
          </div>
          <FileCog size={20} />
        </div>

        <div className="caddy-settings-grid">
          <label>
            <span>Install dir</span>
            <input value={caddy.installDir} onChange={(event) => updateCaddy({ installDir: event.target.value })} />
          </label>
          <label>
            <span>Config path</span>
            <input value={caddy.configPath} onChange={(event) => updateCaddy({ configPath: event.target.value })} />
          </label>
          <label className="toggle-row">
            <input
              type="checkbox"
              checked={caddy.enabled}
              onChange={(event) => updateCaddy({ enabled: event.target.checked })}
            />
            <span>Manage Caddy with PM2</span>
          </label>
        </div>

        <textarea
          className="caddy-editor"
          spellCheck={false}
          value={caddy.config}
          onChange={(event) => updateCaddy({ config: event.target.value })}
        />

        <div className="button-row">
          <button className="primary-button" onClick={onApply} disabled={busy === "caddy-apply"}>
            {busy === "caddy-apply" ? <Loader2 className="spin" size={17} /> : <Save size={17} />}
            Use Main Caddyfile
          </button>
          <button className="secondary-button" onClick={onApplyPublishTest} disabled={busy === "caddy-publish-test"}>
            {busy === "caddy-publish-test" ? <Loader2 className="spin" size={17} /> : <Network size={17} />}
            Test Publish
          </button>
        </div>
      </div>

      {applyResult && <CaddyResultPanel result={applyResult} />}

      <CaddyProcessPanel
        process={process}
        busy={busy}
        pm2Enabled={pm2Enabled}
        onAction={onPm2Action}
        onOpenLog={onOpenLog}
      />
    </section>
  );
}

function CaddyProcessPanel({
  process,
  busy,
  pm2Enabled,
  onAction,
  onOpenLog,
}: {
  process: Pm2Process | null;
  busy: string | null;
  pm2Enabled: boolean;
  onAction: (appName: ManagedAppName, action: Pm2Action) => void;
  onOpenLog: (appName: ManagedAppName) => void;
}) {
  const status = process?.status ?? "not found";
  const normalizedStatus = status.toLowerCase();
  const canControl = pm2Enabled && process !== null;
  const startBusy = busy === "pm2:Caddy:start";
  const stopBusy = busy === "pm2:Caddy:stop";
  const restartBusy = busy === "pm2:Caddy:restart";

  return (
    <div className="panel">
      <div className="panel-heading">
        <div>
          <h2>PM2 Process</h2>
          <span>{process?.cwd ?? "Apply the Caddyfile to register Caddy in PM2."}</span>
        </div>
        <Network size={20} />
      </div>

      <div className="process-card caddy-process-card">
        <div className="process-title">
          <div>
            <strong>Caddy</strong>
            <span>PM2 id {process?.pmId ?? "-"}</span>
          </div>
          <span className={`pill ${statusClassName(normalizedStatus)}`}>{status}</span>
        </div>

        <dl className="process-metrics">
          <dt>PID</dt>
          <dd>{process?.pid ?? "-"}</dd>
          <dt>CPU</dt>
          <dd>{process?.cpu == null ? "-" : `${process.cpu.toFixed(1)}%`}</dd>
          <dt>Memory</dt>
          <dd>{formatBytes(process?.memory)}</dd>
          <dt>Restarts</dt>
          <dd>{process?.restartTime ?? "-"}</dd>
        </dl>

        <div className="process-path">{process?.scriptPath ?? "caddy.exe is not registered in PM2 yet."}</div>

        <div className="process-actions">
          <button
            className="secondary-button compact"
            disabled={!canControl || normalizedStatus === "online" || startBusy}
            onClick={() => onAction("Caddy", "start")}
          >
            {startBusy ? <Loader2 className="spin" size={16} /> : <Play size={16} />}
            Start
          </button>
          <button
            className="secondary-button compact"
            disabled={!canControl || normalizedStatus !== "online" || stopBusy}
            onClick={() => onAction("Caddy", "stop")}
          >
            {stopBusy ? <Loader2 className="spin" size={16} /> : <Square size={16} />}
            Stop
          </button>
          <button
            className="secondary-button compact"
            disabled={!canControl || restartBusy}
            onClick={() => onAction("Caddy", "restart")}
          >
            {restartBusy ? <Loader2 className="spin" size={16} /> : <RefreshCw size={16} />}
            Restart
          </button>
          <button className="secondary-button compact" onClick={() => onOpenLog("Caddy")}>
            <ScrollText size={16} />
            Log
          </button>
        </div>
      </div>
    </div>
  );
}

function CaddyResultPanel({ result }: { result: CaddyCommandResult }) {
  return (
    <div className={`panel operation-result ${result.success ? "valid" : "invalid"}`}>
      <div className="operation-result-heading">
        {result.success ? <CheckCircle2 size={17} /> : <XCircle size={17} />}
        <strong>{result.message}</strong>
      </div>
      <dl className="details caddy-result-details">
        <dt>Path</dt>
        <dd>{result.path ?? "-"}</dd>
        <dt>Command</dt>
        <dd>{result.command}</dd>
        <dt>Version</dt>
        <dd>{result.status.version ?? result.status.error ?? "-"}</dd>
      </dl>
      {(result.stdout || result.stderr) && <pre>{[result.stdout, result.stderr].filter(Boolean).join("\n")}</pre>}
    </div>
  );
}
