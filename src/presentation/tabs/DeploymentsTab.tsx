import {
  Activity,
  CheckCircle2,
  CloudDownload,
  CircleOff,
  FolderOpen,
  History,
  Loader2,
  Play,
  RotateCcw,
  Upload,
  XCircle,
} from "lucide-react";
import { currentDeployStepLabel } from "../../domain/deploy/deploy-steps";
import type { DeployStepView } from "../../domain/deploy/types";
import { formatDate } from "../../shared/formatters";
import type { DeploymentRecord, DeploymentState, PackageValidation, PortalReleaseCheckResult } from "../../types";

interface DeploymentsTabProps {
  packagePath: string;
  setPackagePath: (value: string) => void;
  validation: PackageValidation | null;
  portalReleaseCheck: PortalReleaseCheckResult | null;
  portalReleaseReady: boolean;
  deploySteps: DeployStepView[];
  deployments: DeploymentState;
  busy: string | null;
  onBrowse: () => void;
  onValidate: () => void;
  onDeploy: () => void;
  onCheckPortalRelease: () => void;
  onDeployPortalRelease: () => void;
  onRollback: (deployment: DeploymentRecord) => void;
}

export function DeploymentsTab({
  packagePath,
  setPackagePath,
  validation,
  portalReleaseCheck,
  portalReleaseReady,
  deploySteps,
  deployments,
  busy,
  onBrowse,
  onValidate,
  onDeploy,
  onCheckPortalRelease,
  onDeployPortalRelease,
  onRollback,
}: DeploymentsTabProps) {
  return (
    <section className="stack">
      <PortalUpdatePanel
        releaseCheck={portalReleaseCheck}
        ready={portalReleaseReady}
        busy={busy}
        onCheck={onCheckPortalRelease}
        onDeploy={onDeployPortalRelease}
      />

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Deploy Package</h2>
            <span>Portal/build/index.js + WebApi/WebApi.exe</span>
          </div>
          <Upload size={20} />
        </div>

        <div className="package-row">
          <input
            value={packagePath}
            onChange={(event) => {
              setPackagePath(event.target.value);
            }}
            placeholder="C:\\path\\to\\release.zip"
          />
          <button className="secondary-button" onClick={onBrowse}>
            <FolderOpen size={17} />
            Browse
          </button>
          <button className="secondary-button" disabled={!packagePath || busy === "validate"} onClick={onValidate}>
            {busy === "validate" ? <Loader2 className="spin" size={17} /> : <CheckCircle2 size={17} />}
            Validate
          </button>
          <button className="primary-button" disabled={!packagePath || busy === "deploy"} onClick={onDeploy}>
            {busy === "deploy" ? <Loader2 className="spin" size={17} /> : <Upload size={17} />}
            Deploy
          </button>
        </div>

        {validation && (
          <div className={`validation-box ${validation.valid ? "valid" : "invalid"}`}>
            <strong>{validation.valid ? "Valid package" : "Invalid package"}</strong>
            <span>{validation.entriesChecked} zip entries checked</span>
            {!validation.valid && <code>{validation.missing.join(", ")}</code>}
          </div>
        )}
      </div>

      <DeployProgressPanel steps={deploySteps} />

      <div className="panel">
        <div className="panel-heading">
          <div>
            <h2>Deployment History</h2>
            <span>{deployments.deployments.length} saved release(s)</span>
          </div>
          <History size={20} />
        </div>
        <DeploymentTable deployments={deployments} busy={busy} onRollback={onRollback} />
      </div>
    </section>
  );
}

function PortalUpdatePanel({
  releaseCheck,
  ready,
  busy,
  onCheck,
  onDeploy,
}: {
  releaseCheck: PortalReleaseCheckResult | null;
  ready: boolean;
  busy: string | null;
  onCheck: () => void;
  onDeploy: () => void;
}) {
  const latest = releaseCheck?.latest ?? null;
  const checking = busy === "portal-release-check";
  const deploying = busy === "portal-release-deploy";

  return (
    <div className="panel">
      <div className="panel-heading">
        <div>
          <h2>Portal Update</h2>
          <span>Private GitHub release package</span>
        </div>
        <CloudDownload size={20} />
      </div>

      <div className="release-update-grid">
        <div className="release-update-details">
          {latest ? (
            <>
              <div>
                <span>Latest</span>
                <strong>{latest.tagName}</strong>
              </div>
              <div>
                <span>Asset</span>
                <strong>{latest.assetName}</strong>
              </div>
              <div>
                <span>Size</span>
                <strong>{formatBytes(latest.assetSize)}</strong>
              </div>
              <div>
                <span>Status</span>
                <strong>{releaseCheck?.updateAvailable ? "Update available" : "Active release"}</strong>
              </div>
            </>
          ) : (
            <div>
              <span>Latest</span>
              <strong>{ready ? "Not checked" : "Configure GitHub PAT in Settings"}</strong>
            </div>
          )}
        </div>

        <div className="release-update-actions">
          <button className="secondary-button" onClick={onCheck} disabled={!ready || checking || deploying}>
            {checking ? <Loader2 className="spin" size={17} /> : <CheckCircle2 size={17} />}
            Check
          </button>
          <button className="primary-button" onClick={onDeploy} disabled={!ready || deploying || checking}>
            {deploying ? <Loader2 className="spin" size={17} /> : <CloudDownload size={17} />}
            Deploy Release
          </button>
        </div>
      </div>
    </div>
  );
}

function DeploymentTable({
  deployments,
  busy,
  onRollback,
}: {
  deployments: DeploymentState;
  busy: string | null;
  onRollback: (deployment: DeploymentRecord) => void;
}) {
  if (deployments.deployments.length === 0) {
    return <div className="empty-state">No deployments yet.</div>;
  }

  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Release</th>
            <th>Created</th>
            <th>Path</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {deployments.deployments.map((deployment) => {
            const active = deployments.activeDeploymentId === deployment.id;
            const rollbackBusy = busy === `rollback:${deployment.id}`;
            return (
              <tr key={deployment.id}>
                <td>{deployment.id}</td>
                <td>{formatDate(deployment.createdAt)}</td>
                <td className="path-cell">{deployment.deploymentPath}</td>
                <td>
                  <span className={`pill ${active ? "green" : "neutral"}`}>
                    {active ? "Active config" : deployment.releaseTag ?? "Ready"}
                  </span>
                </td>
                <td className="actions-cell">
                  <button
                    className="secondary-button compact"
                    disabled={rollbackBusy}
                    onClick={() => onRollback(deployment)}
                  >
                    {rollbackBusy ? (
                      <Loader2 className="spin" size={16} />
                    ) : active ? (
                      <Play size={16} />
                    ) : (
                      <RotateCcw size={16} />
                    )}
                    {active ? "Run PM2" : "Rollback"}
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (!bytes) return "Unknown";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

function DeployProgressPanel({ steps }: { steps: DeployStepView[] }) {
  return (
    <div className="panel">
      <div className="panel-heading">
        <div>
          <h2>Deploy Progress</h2>
          <span>{currentDeployStepLabel(steps)}</span>
        </div>
        <Activity size={20} />
      </div>

      <div className="deploy-steps">
        {steps.map((step) => (
          <div className={`deploy-step ${step.state}`} key={step.id}>
            <div className="deploy-step-marker">
              {step.state === "running" && <Loader2 className="spin" size={15} />}
              {step.state === "done" && <CheckCircle2 size={15} />}
              {step.state === "failed" && <XCircle size={15} />}
              {(step.state === "pending" || step.state === "skipped") && <CircleOff size={15} />}
            </div>
            <div>
              <strong>{step.label}</strong>
              <span>{step.detail}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
