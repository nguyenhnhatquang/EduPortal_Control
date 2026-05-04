import type { EnvMap } from "../../types";

export function EnvPreview({ env }: { env: EnvMap }) {
  return (
    <div className="env-preview">
      {Object.entries(env).map(([key, value]) => (
        <div key={key}>
          <code>{key}</code>
          <span>{value}</span>
        </div>
      ))}
    </div>
  );
}
