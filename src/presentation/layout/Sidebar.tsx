import { ShieldCheck } from "lucide-react";
import { isTauriRuntime } from "../../api";
import type { SystemStatus } from "../../types";
import { NavButton } from "../components/NavButton";
import { futureTabs, primaryTabs } from "../navigation";
import type { TabId } from "../tabs/types";

interface SidebarProps {
  activeTab: TabId;
  status: SystemStatus | null;
  onSelectTab: (tab: TabId) => void;
}

export function Sidebar({ activeTab, status, onSelectTab }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">
          <ShieldCheck size={20} />
        </div>
        <div>
          <strong>EduPortal_Control</strong>
          <span>VPS Manager</span>
        </div>
      </div>

      <nav className="nav">
        {primaryTabs.map((tab) => (
          <NavButton key={tab.id} tab={tab} active={activeTab === tab.id} onClick={() => onSelectTab(tab.id)} />
        ))}
      </nav>

      <div className="sidebar-section">
        <span className="sidebar-caption">Next</span>
        {futureTabs.map((tab) => (
          <NavButton key={tab.id} tab={tab} active={false} disabled onClick={() => undefined} />
        ))}
      </div>

      <div className="runtime-box">
        <span className={isTauriRuntime() ? "dot online" : "dot muted"} />
        <div>
          <strong>{isTauriRuntime() ? "Desktop runtime" : "Browser preview"}</strong>
          <span>{status?.os ?? "loading"}</span>
        </div>
      </div>
    </aside>
  );
}
