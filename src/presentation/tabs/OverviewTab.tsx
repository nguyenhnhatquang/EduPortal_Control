import {
  Activity,
  CheckCircle2,
  CircleOff,
  HardDrive,
  Loader2,
  Play,
  RefreshCw,
  ScrollText,
  Server,
  Square,
  Terminal,
} from "lucide-react";
import { EnvPreview } from "../components/EnvPreview";
import { formatBytes, formatDate, statusClassName } from "../../shared/formatters";
import type { DeploymentRecord, DeploymentState, Pm2Action, Pm2Process, Settings, SystemStatus } from "../../types";

interface OverviewTabProps {
  status: SystemStatus | null;
  deployments: DeploymentState;
  activeDeployment: DeploymentRecord | null;
  settings: Settings;
  pm2Processes: Pm2Process[];
  busy: string | null;
  onPm2Action: (appName: "Portal" | "WebApi", action: Pm2Action) => void;
  onOpenLog: (appName: "Portal" | "WebApi") => void;
}

export function OverviewTab({
  status,
  deployments,
  activeDeployment,
  settings,
  pm2Processes,
  busy,
  onPm2Action,
  onOpenLog,
}: OverviewTabProps) {
  return (
    <section className="content-grid overview-grid">
      <div className="panel span-2">
        <div className="panel-heading">
          <div>
            <h2>Runtime Status</h2>
            <span>{settings.deployRoot}</span>
          </div>
          <Activity size={20} />
        </div>

        <div className="status-list">
          <StatusRow
            label="PM2"
            value={status?.pm2Version ?? status?.pm2Error ?? "checking"}
            ok={Boolean(status?.pm2Version)}
          />
          <StatusRow
            label="Node"
            value={status?.nodeVersion ?? status?.nodeError ?? "checking"}
            ok={Boolean(status?.nodeVersion)}
          />
          <StatusRow
            label="Deploy root"
            value={status?.deployRootExists ? "Available" : "Not created yet"}
            ok={Boolean(status?.deployRootExists)}
          />
          <StatusRow
            label="PM2 execution"
            value={status?.pm2ExecutionEnabled ? "Enabled" : "Skipped outside Windows"}
            ok={Boolean(status?.pm2ExecutionEnabled)}
          />
        </div>
      </div>

      <Pm2ProcessPanel
        processes={pm2Processes}
        busy={busy}
        pm2Enabled={Boolean(status?.pm2ExecutionEnabled)}
        onAction={onPm2Action}
        onOpenLog={onOpenLog}
      />

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Active Release</h2>
            <span>{activeDeployment?.id ?? "none"}</span>
          </div>
          <HardDrive size={20} />
        </div>
        <dl className="details">
          <dt>Created</dt>
          <dd>{activeDeployment ? formatDate(activeDeployment.createdAt) : "No deployment"}</dd>
          <dt>Path</dt>
          <dd>{activeDeployment?.deploymentPath ?? "No active path"}</dd>
          <dt>History</dt>
          <dd>{deployments.deployments.length} deployment(s)</dd>
        </dl>
      </div>

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Portal Env</h2>
            <span>SvelteKit adapter-node</span>
          </div>
          <Terminal size={20} />
        </div>
        <EnvPreview env={settings.portalEnv} />
      </div>
    </section>
  );
}

function Pm2ProcessPanel({
  processes,
  busy,
  pm2Enabled,
  onAction,
  onOpenLog,
}: {
  processes: Pm2Process[];
  busy: string | null;
  pm2Enabled: boolean;
  onAction: (appName: "Portal" | "WebApi", action: Pm2Action) => void;
  onOpenLog: (appName: "Portal" | "WebApi") => void;
}) {
  const processByName = new Map(processes.map((process) => [process.name, process]));
  const managedApps: Array<"Portal" | "WebApi"> = ["Portal", "WebApi"];

  return (
    <div className="panel span-2">
      <div className="panel-heading">
        <div>
          <h2>PM2 Processes</h2>
          <span>{pm2Enabled ? "Start, stop, restart, and inspect app logs." : "PM2 controls are active on Windows VPS."}</span>
        </div>
        <Server size={20} />
      </div>

      <div className="process-grid">
        {managedApps.map((appName) => {
          const process = processByName.get(appName) ?? null;
          const status = process?.status ?? "not found";
          const normalizedStatus = status.toLowerCase();
          const canControl = pm2Enabled && process !== null;
          const startBusy = busy === `pm2:${appName}:start`;
          const stopBusy = busy === `pm2:${appName}:stop`;
          const restartBusy = busy === `pm2:${appName}:restart`;

          return (
            <div className="process-card" key={appName}>
              <div className="process-title">
                <div>
                  <strong>{appName}</strong>
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

              <div className="process-path">{process?.cwd ?? "Deploy first to register this app in PM2."}</div>

              <div className="process-actions">
                <button
                  className="secondary-button compact"
                  disabled={!canControl || normalizedStatus === "online" || startBusy}
                  onClick={() => onAction(appName, "start")}
                >
                  {startBusy ? <Loader2 className="spin" size={16} /> : <Play size={16} />}
                  Start
                </button>
                <button
                  className="secondary-button compact"
                  disabled={!canControl || normalizedStatus !== "online" || stopBusy}
                  onClick={() => onAction(appName, "stop")}
                >
                  {stopBusy ? <Loader2 className="spin" size={16} /> : <Square size={16} />}
                  Stop
                </button>
                <button
                  className="secondary-button compact"
                  disabled={!canControl || restartBusy}
                  onClick={() => onAction(appName, "restart")}
                >
                  {restartBusy ? <Loader2 className="spin" size={16} /> : <RefreshCw size={16} />}
                  Restart
                </button>
                <button className="secondary-button compact" onClick={() => onOpenLog(appName)}>
                  <ScrollText size={16} />
                  Log
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function StatusRow({ label, value, ok }: { label: string; value: string; ok: boolean }) {
  return (
    <div className="status-row">
      <div>
        {ok ? <CheckCircle2 size={17} className="status-ok" /> : <CircleOff size={17} className="status-warn" />}
        <span>{label}</span>
      </div>
      <strong>{value}</strong>
    </div>
  );
}
