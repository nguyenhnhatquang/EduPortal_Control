import { Download, Loader2, RefreshCw } from "lucide-react";
import type { ManagerUpdateInfo, ManagerUpdateProgress } from "../../types";

interface TopbarProps {
  title: string;
  subtitle: string;
  version: string;
  refreshing: boolean;
  managerUpdate: ManagerUpdateInfo | null;
  updateProgress: ManagerUpdateProgress | null;
  checkingUpdate: boolean;
  installingUpdate: boolean;
  onRefresh: () => void;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
}

export function Topbar({
  title,
  subtitle,
  version,
  refreshing,
  managerUpdate,
  updateProgress,
  checkingUpdate,
  installingUpdate,
  onRefresh,
  onCheckUpdate,
  onInstallUpdate,
}: TopbarProps) {
  const updatePercent =
    updateProgress?.contentLength && updateProgress.contentLength > 0
      ? Math.min(100, Math.round((updateProgress.downloadedBytes / updateProgress.contentLength) * 100))
      : null;

  return (
    <header className="topbar">
      <div>
        <h1>{title}</h1>
        <p>{subtitle}</p>
      </div>
      <div className="topbar-actions">
        <span className="version-badge">v{version}</span>
        {managerUpdate ? (
          <button
            className="secondary-button compact update-button"
            onClick={onInstallUpdate}
            disabled={installingUpdate}
            title={`Install manager update v${managerUpdate.version}`}
          >
            {installingUpdate ? <Loader2 className="spin" size={17} /> : <Download size={17} />}
            {installingUpdate ? (updatePercent === null ? "Installing" : `${updatePercent}%`) : `Install v${managerUpdate.version}`}
          </button>
        ) : (
          <button
            className="icon-button"
            onClick={onCheckUpdate}
            disabled={checkingUpdate}
            title="Check for manager update"
          >
            {checkingUpdate ? <Loader2 className="spin" size={18} /> : <Download size={18} />}
          </button>
        )}
        <button className="icon-button" onClick={onRefresh} disabled={refreshing} title="Refresh">
          {refreshing ? <Loader2 className="spin" size={18} /> : <RefreshCw size={18} />}
        </button>
      </div>
    </header>
  );
}
