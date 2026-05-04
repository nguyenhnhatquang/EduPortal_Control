import {
  Archive,
  Database,
  Download,
  Globe,
  Network,
  ScrollText,
  Server,
  Settings as SettingsIcon,
  Upload,
} from "lucide-react";
import type { TabDefinition, TabId } from "./tabs/types";

export const primaryTabs: TabDefinition[] = [
  { id: "overview", label: "Overview", icon: Server },
  { id: "deployments", label: "Deployments", icon: Upload },
  { id: "logs", label: "Logs", icon: ScrollText },
  { id: "database", label: "Database", icon: Database },
  { id: "caddy", label: "Caddy", icon: Network },
  { id: "software", label: "Install", icon: Download },
  { id: "settings", label: "Settings", icon: SettingsIcon },
];

export const futureTabs: TabDefinition[] = [
  { id: "domains", label: "Domains", icon: Globe },
  { id: "backups", label: "Backups", icon: Archive },
];

export function titleForTab(tab: TabId) {
  switch (tab) {
    case "overview":
      return "Overview";
    case "deployments":
      return "Deployments";
    case "logs":
      return "Logs";
    case "settings":
      return "Settings";
    case "database":
      return "Database";
    case "caddy":
      return "Caddy";
    case "software":
      return "Install";
    case "domains":
      return "Domains";
    case "backups":
      return "Backups";
  }
}

export function subtitleForTab(tab: TabId) {
  switch (tab) {
    case "overview":
      return "Local VPS process and deployment state.";
    case "deployments":
      return "Upload a release zip, generate PM2 config, and rollback releases.";
    case "logs":
      return "Tail PM2 logs for Portal and WebApi.";
    case "database":
      return "Back up, restore, schedule, and migrate the PostgreSQL database.";
    case "caddy":
      return "Install caddy.exe, edit Caddyfile, and manage the Caddy PM2 process.";
    case "software":
      return "Install and verify Node.js, PM2, and PostgreSQL on the VPS.";
    case "settings":
      return "Deployment root, retention, and PM2 environment variables.";
    default:
      return "Prepared for future hosting operations.";
  }
}
