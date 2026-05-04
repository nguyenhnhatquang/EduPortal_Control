import type { LucideIcon } from "lucide-react";

export type TabId =
  | "overview"
  | "deployments"
  | "logs"
  | "database"
  | "caddy"
  | "software"
  | "settings"
  | "domains"
  | "backups";

export interface TabDefinition {
  id: TabId;
  label: string;
  icon: LucideIcon;
}
