import type { ManagedAppName, Pm2Action, Pm2Process } from "../types";

export const DEPLOY_PM2_APPS = ["Portal", "WebApi"] as const satisfies readonly ManagedAppName[];
export const CADDY_PM2_APP = "Caddy" as const satisfies ManagedAppName;

export function pm2BusyKey(appName: ManagedAppName, action: Pm2Action) {
  return `pm2:${appName}:${action}`;
}

export function findPm2Process(processes: Pm2Process[], appName: ManagedAppName) {
  return processes.find((process) => process.name === appName) ?? null;
}
