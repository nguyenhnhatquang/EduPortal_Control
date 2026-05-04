import {
  Activity,
  CheckCircle2,
  CircleOff,
  HardDrive,
  Server,
  Terminal,
} from "lucide-react";
import { DEPLOY_PM2_APPS, findPm2Process } from "../../domain/pm2";
import { Pm2ProcessCard } from "../components/Pm2ProcessCard";
import { EnvPreview } from "../components/EnvPreview";
import { formatDate } from "../../shared/formatters";
import type {
  DeploymentRecord,
  DeploymentState,
  ManagedAppName,
  Pm2Action,
  Pm2Process,
  Settings,
  SystemStatus,
} from "../../types";

interface OverviewTabProps {
  status: SystemStatus | null;
  deployments: DeploymentState;
  activeDeployment: DeploymentRecord | null;
  settings: Settings;
  pm2Processes: Pm2Process[];
  busy: string | null;
  onPm2Action: (appName: ManagedAppName, action: Pm2Action) => void;
  onOpenLog: (appName: ManagedAppName) => void;
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
  onAction: (appName: ManagedAppName, action: Pm2Action) => void;
  onOpenLog: (appName: ManagedAppName) => void;
}) {
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
        {DEPLOY_PM2_APPS.map((appName) => (
          <Pm2ProcessCard
            key={appName}
            appName={appName}
            process={findPm2Process(processes, appName)}
            busy={busy}
            pm2Enabled={pm2Enabled}
            missingPathLabel="Deploy first to register this app in PM2."
            onAction={onAction}
            onOpenLog={onOpenLog}
          />
        ))}
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
