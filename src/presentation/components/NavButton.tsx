import type { TabDefinition } from "../tabs/types";

interface NavButtonProps {
  tab: TabDefinition;
  active: boolean;
  disabled?: boolean;
  onClick: () => void;
}

export function NavButton({ tab, active, disabled, onClick }: NavButtonProps) {
  const Icon = tab.icon;
  return (
    <button className={`nav-button ${active ? "active" : ""}`} disabled={disabled} onClick={onClick}>
      <Icon size={18} />
      <span>{tab.label}</span>
    </button>
  );
}
