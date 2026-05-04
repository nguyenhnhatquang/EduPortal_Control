import { Loader2, Play, RefreshCw, ScrollText, Square } from "lucide-react";
import { pm2BusyKey } from "../../domain/pm2";
import { formatBytes, statusClassName } from "../../shared/formatters";
import type { ManagedAppName, Pm2Action, Pm2Process } from "../../types";

interface Pm2ProcessCardProps {
  appName: ManagedAppName;
  process: Pm2Process | null;
  busy: string | null;
  pm2Enabled: boolean;
  missingPathLabel: string;
  pathField?: "cwd" | "scriptPath";
  className?: string;
  onAction: (appName: ManagedAppName, action: Pm2Action) => void;
  onOpenLog: (appName: ManagedAppName) => void;
}

export function Pm2ProcessCard({
  appName,
  process,
  busy,
  pm2Enabled,
  missingPathLabel,
  pathField = "cwd",
  className,
  onAction,
  onOpenLog,
}: Pm2ProcessCardProps) {
  const status = process?.status ?? "not found";
  const normalizedStatus = status.toLowerCase();
  const canControl = pm2Enabled && process !== null;
  const startBusy = busy === pm2BusyKey(appName, "start");
  const stopBusy = busy === pm2BusyKey(appName, "stop");
  const restartBusy = busy === pm2BusyKey(appName, "restart");
  const processPath = process?.[pathField] ?? missingPathLabel;

  return (
    <div className={["process-card", className].filter(Boolean).join(" ")}>
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

      <div className="process-path">{processPath}</div>

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
}
