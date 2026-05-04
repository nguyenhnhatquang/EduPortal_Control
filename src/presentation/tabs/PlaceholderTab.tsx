import { Database } from "lucide-react";

export function PlaceholderTab({ label }: { label: string }) {
  return (
    <div className="panel placeholder">
      <Database size={28} />
      <h2>{label}</h2>
      <span>Reserved for the next VPS hosting modules.</span>
    </div>
  );
}
