import { CheckCircle2, XCircle } from "lucide-react";

interface FeedbackProps {
  notice: string | null;
  error: string | null;
  onDismiss: () => void;
}

export function Feedback({ notice, error, onDismiss }: FeedbackProps) {
  if (!notice && !error) return null;

  return (
    <div className={`feedback ${error ? "error" : "notice"}`}>
      {error ? <XCircle size={18} /> : <CheckCircle2 size={18} />}
      <span>{error ?? notice}</span>
      <button className="icon-button small" onClick={onDismiss} title="Dismiss">
        <XCircle size={16} />
      </button>
    </div>
  );
}
