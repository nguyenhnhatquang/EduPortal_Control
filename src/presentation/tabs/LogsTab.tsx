import { Loader2, RefreshCw, ScrollText } from "lucide-react";
import type { LogReadResult, ManagedAppName } from "../../types";

interface LogsTabProps {
  logApp: ManagedAppName;
  setLogApp: (value: ManagedAppName) => void;
  logLines: number;
  setLogLines: (value: number) => void;
  logResult: LogReadResult | null;
  busy: string | null;
  onRefresh: () => void;
}

export function LogsTab({
  logApp,
  setLogApp,
  logLines,
  setLogLines,
  logResult,
  busy,
  onRefresh,
}: LogsTabProps) {
  return (
    <section className="stack">
      <div className="panel log-toolbar">
        <div className="segmented">
          <button className={logApp === "Portal" ? "selected" : ""} onClick={() => setLogApp("Portal")}>
            Portal
          </button>
          <button className={logApp === "WebApi" ? "selected" : ""} onClick={() => setLogApp("WebApi")}>
            WebApi
          </button>
          <button className={logApp === "Caddy" ? "selected" : ""} onClick={() => setLogApp("Caddy")}>
            Caddy
          </button>
        </div>
        <label className="inline-field">
          <span>Lines</span>
          <input
            type="number"
            min={50}
            max={2000}
            value={logLines}
            onChange={(event) => setLogLines(Number(event.target.value))}
          />
        </label>
        <button className="primary-button" onClick={onRefresh} disabled={busy === "logs"}>
          {busy === "logs" ? <Loader2 className="spin" size={17} /> : <RefreshCw size={17} />}
          Refresh
        </button>
      </div>

      <div className="panel log-panel">
        <div className="panel-heading">
          <div>
            <h2>{logResult?.appName ?? logApp}.log</h2>
            <span>{logResult?.path ?? "Log path will appear after refresh"}</span>
          </div>
          <ScrollText size={20} />
        </div>
        <pre className="log-output">{(logResult?.lines.length ? logResult.lines : ["No log lines."]).join("\n")}</pre>
      </div>
    </section>
  );
}
