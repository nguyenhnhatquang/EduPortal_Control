import {
  CheckCircle2,
  Database,
  Download,
  Loader2,
  RefreshCw,
  Terminal,
  XCircle,
} from "lucide-react";
import type { SoftwareInstallResult, SoftwarePackageId, SoftwarePackageStatus } from "../../types";

interface SoftwareTabProps {
  packages: SoftwarePackageStatus[];
  installResult: SoftwareInstallResult | null;
  busy: string | null;
  onInstall: (packageId: SoftwarePackageId) => void;
  onRefresh: () => void;
}

export function SoftwareTab({ packages, installResult, busy, onInstall, onRefresh }: SoftwareTabProps) {
  const packageById = new Map(packages.map((item) => [item.id, item]));
  const nodeInstalled = Boolean(packageById.get("nodejs")?.installed);
  const orderedPackages: SoftwarePackageStatus[] = [
    packageById.get("nodejs") ?? emptyPackage("nodejs", "Node.js"),
    packageById.get("pm2") ?? emptyPackage("pm2", "PM2"),
    packageById.get("postgresql") ?? emptyPackage("postgresql", "PostgreSQL"),
  ];

  return (
    <section className="stack">
      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Software Environment</h2>
            <span>Windows Server runtime packages for deploy, PM2, and PostgreSQL operations.</span>
          </div>
          <button className="icon-button" onClick={onRefresh} disabled={busy === "software-list"} title="Refresh">
            {busy === "software-list" ? <Loader2 className="spin" size={18} /> : <RefreshCw size={18} />}
          </button>
        </div>

        <div className="software-grid">
          {orderedPackages.map((item) => (
            <SoftwareCard
              key={item.id}
              item={item}
              busy={busy}
              nodeInstalled={nodeInstalled}
              onInstall={onInstall}
            />
          ))}
        </div>
      </div>

      {installResult && <SoftwareResultPanel result={installResult} />}
    </section>
  );
}

function SoftwareCard({
  item,
  busy,
  nodeInstalled,
  onInstall,
}: {
  item: SoftwarePackageStatus;
  busy: string | null;
  nodeInstalled: boolean;
  onInstall: (packageId: SoftwarePackageId) => void;
}) {
  const isBusy = busy === `software:${item.id}`;
  const missingPath = item.missingPathEntries.length > 0;
  const pathState = missingPath ? "Missing PATH entry" : "PATH ready";
  const canInstall = item.id !== "pm2" || nodeInstalled || item.installed;
  const actionLabel = item.installed ? (missingPath ? "Repair PATH" : "Check PATH") : "Install";

  return (
    <div className="software-card">
      <div className="software-card-title">
        <div className="software-icon">{softwareIcon(item.id)}</div>
        <div>
          <strong>{item.name}</strong>
          <span>{item.version ?? item.error ?? "Not installed"}</span>
        </div>
        <span className={`pill ${item.installed ? (missingPath ? "amber" : "green") : "neutral"}`}>
          {item.installed ? "Installed" : "Missing"}
        </span>
      </div>

      <dl className="details software-details">
        <dt>Command</dt>
        <dd>{item.executable}</dd>
        <dt>PATH</dt>
        <dd>{pathState}</dd>
        <dt>Entries</dt>
        <dd>{item.pathEntries.length > 0 ? item.pathEntries.join("; ") : "-"}</dd>
      </dl>

      {missingPath && (
        <div className="software-path-warning">
          {item.missingPathEntries.map((entry) => (
            <code key={entry}>{entry}</code>
          ))}
        </div>
      )}

      <div className="button-row">
        <button className="primary-button" disabled={isBusy || !canInstall} onClick={() => onInstall(item.id)}>
          {isBusy ? <Loader2 className="spin" size={17} /> : item.installed ? <RefreshCw size={17} /> : <Download size={17} />}
          {actionLabel}
        </button>
      </div>
    </div>
  );
}

function SoftwareResultPanel({ result }: { result: SoftwareInstallResult }) {
  return (
    <div className={`panel operation-result ${result.success ? "valid" : "invalid"}`}>
      <div className="operation-result-heading">
        {result.success ? <CheckCircle2 size={17} /> : <XCircle size={17} />}
        <strong>{result.message}</strong>
      </div>
      <dl className="details software-result-details">
        <dt>Package</dt>
        <dd>{result.status.name}</dd>
        <dt>Command</dt>
        <dd>{result.command}</dd>
        <dt>PATH added</dt>
        <dd>{result.pathEntriesAdded.length > 0 ? result.pathEntriesAdded.join("; ") : "-"}</dd>
        <dt>Status</dt>
        <dd>{result.status.version ?? result.status.error ?? "-"}</dd>
      </dl>
      {(result.stdout || result.stderr) && <pre>{[result.stdout, result.stderr].filter(Boolean).join("\n")}</pre>}
    </div>
  );
}

function softwareIcon(packageId: SoftwarePackageId) {
  switch (packageId) {
    case "postgresql":
      return <Database size={19} />;
    case "pm2":
      return <RefreshCw size={19} />;
    case "nodejs":
      return <Terminal size={19} />;
  }
}

function emptyPackage(id: SoftwarePackageId, name: string): SoftwarePackageStatus {
  return {
    id,
    name,
    installed: false,
    version: null,
    error: "Not checked yet",
    executable: id,
    pathEntries: [],
    missingPathEntries: [],
  };
}
