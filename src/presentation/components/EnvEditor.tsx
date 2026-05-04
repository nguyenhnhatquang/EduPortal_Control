import { Plus, Trash2 } from "lucide-react";
import type { EnvMap } from "../../types";

interface EnvEditorProps {
  title: string;
  env: EnvMap;
  onChange: (env: EnvMap) => void;
}

export function EnvEditor({ title, env, onChange }: EnvEditorProps) {
  const entries = Object.entries(env);

  function changeKey(oldKey: string, nextKey: string) {
    const next = { ...env };
    const value = next[oldKey] ?? "";
    delete next[oldKey];
    next[nextKey] = value;
    onChange(next);
  }

  function changeValue(key: string, value: string) {
    onChange({ ...env, [key]: value });
  }

  function removeKey(key: string) {
    const next = { ...env };
    delete next[key];
    onChange(next);
  }

  function addKey() {
    let index = entries.length + 1;
    let key = `NEW_ENV_${index}`;
    while (env[key] !== undefined) {
      index += 1;
      key = `NEW_ENV_${index}`;
    }
    onChange({ ...env, [key]: "" });
  }

  return (
    <div className="env-editor-block">
      <div className="env-editor-heading">
        <div>
          <strong>{title}</strong>
          <span>{entries.length} variable(s)</span>
        </div>
        <button className="icon-button" onClick={addKey} title="Add env">
          <Plus size={17} />
        </button>
      </div>

      <div className="env-editor">
        {entries.map(([key, value], index) => (
          <div className="env-row" key={`${key}-${index}`}>
            <input value={key} onChange={(event) => changeKey(key, event.target.value)} />
            <input value={value} onChange={(event) => changeValue(key, event.target.value)} />
            <button className="icon-button" onClick={() => removeKey(key)} title="Remove env">
              <Trash2 size={16} />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
