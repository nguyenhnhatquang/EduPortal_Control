export type EnvMap = Record<string, string>;

export interface PortalAssetCopy {
  enabled: boolean;
  source: string;
  destination: string;
}

export type DatabaseBackupScheduleFrequency = "daily" | "weekly";

export type DatabaseBackupScheduleDay =
  | "Monday"
  | "Tuesday"
  | "Wednesday"
  | "Thursday"
  | "Friday"
  | "Saturday"
  | "Sunday";

export interface DatabaseBackupSchedule {
  enabled: boolean;
  frequency: DatabaseBackupScheduleFrequency;
  time: string;
  dayOfWeek: DatabaseBackupScheduleDay;
}

export interface DatabaseSettings {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  binDir: string;
  backupDir: string;
  backupRetention: number;
  backupSchedule: DatabaseBackupSchedule;
}

export interface CaddySettings {
  enabled: boolean;
  installDir: string;
  configPath: string;
  config: string;
}

export interface Settings {
  deployRoot: string;
  retention: number;
  portalEnv: EnvMap;
  webApiEnv: EnvMap;
  portalInstallDependencies: boolean;
  portalAssetCopy: PortalAssetCopy;
  migrationUrl: string;
  migrationKey: string;
  migrationTimeoutSecs: number;
  database: DatabaseSettings;
  caddy: CaddySettings;
}

export interface DeploymentRecord {
  id: string;
  createdAt: string;
  sourceZip: string;
  deploymentPath: string;
  configPath: string;
  portalEnv: EnvMap;
  webApiEnv: EnvMap;
}

export interface DeploymentState {
  activeDeploymentId: string | null;
  deployments: DeploymentRecord[];
}

export interface SystemStatus {
  appVersion: string;
  os: string;
  isWindows: boolean;
  deployRoot: string;
  deployRootExists: boolean;
  deployRootReadonly: boolean | null;
  pm2ExecutionEnabled: boolean;
  nodeVersion: string | null;
  nodeError: string | null;
  pm2Version: string | null;
  pm2Error: string | null;
  activeDeployment: DeploymentRecord | null;
}

export interface ManagerUpdateInfo {
  currentVersion: string;
  version: string;
  date: string | null;
  body: string | null;
}

export interface ManagerUpdateProgress {
  downloadedBytes: number;
  contentLength: number | null;
}

export type SoftwarePackageId = "nodejs" | "pm2" | "postgresql";

export interface SoftwarePackageStatus {
  id: SoftwarePackageId;
  name: string;
  installed: boolean;
  version: string | null;
  error: string | null;
  executable: string;
  pathEntries: string[];
  missingPathEntries: string[];
}

export interface SoftwareInstallResult {
  packageId: SoftwarePackageId;
  attempted: boolean;
  skipped: boolean;
  success: boolean;
  command: string;
  stdout: string;
  stderr: string;
  message: string;
  pathEntriesAdded: string[];
  status: SoftwarePackageStatus;
}

export interface CaddyStatus {
  installed: boolean;
  version: string | null;
  error: string | null;
  executablePath: string;
  installDir: string;
  configPath: string;
  configExists: boolean;
}

export interface CaddyCommandResult {
  attempted: boolean;
  skipped: boolean;
  success: boolean;
  command: string;
  path: string | null;
  stdout: string;
  stderr: string;
  message: string;
  status: CaddyStatus;
  pm2: Pm2CommandResult | null;
}

export interface CaddyFirewallResult {
  attempted: boolean;
  skipped: boolean;
  success: boolean;
  message: string;
  rules: CaddyFirewallRuleResult[];
}

export interface CaddyFirewallRuleResult {
  name: string;
  port: number;
  success: boolean;
  command: string;
  stdout: string;
  stderr: string;
  message: string;
}

export interface MigrationResult {
  success: boolean;
  url: string;
  statusCode: number | null;
  message: string;
  body: string;
}

export interface DatabaseCommandResult {
  attempted: boolean;
  skipped: boolean;
  success: boolean;
  command: string;
  path: string | null;
  stdout: string;
  stderr: string;
  message: string;
}

export interface DatabaseBackupFile {
  fileName: string;
  path: string;
  sizeBytes: number;
  modifiedAt: string;
}

export interface DatabaseBackupListing {
  backupDir: string;
  files: DatabaseBackupFile[];
}

export interface PackageValidation {
  valid: boolean;
  zipPath: string;
  entriesChecked: number;
  missing: string[];
}

export interface Pm2CommandResult {
  attempted: boolean;
  skipped: boolean;
  success: boolean;
  command: string;
  stdout: string;
  stderr: string;
  message: string;
}

export type Pm2Action = "start" | "stop" | "restart";

export type ManagedAppName = "Portal" | "WebApi" | "Caddy";

export interface Pm2Process {
  name: string;
  pmId: number | null;
  status: string;
  pid: number | null;
  restartTime: number | null;
  unstableRestarts: number | null;
  cpu: number | null;
  memory: number | null;
  uptime: number | null;
  scriptPath: string | null;
  cwd: string | null;
}

export interface PostDeployStepResult {
  name: string;
  success: boolean;
  skipped: boolean;
  message: string;
  command: string | null;
  stdout: string;
  stderr: string;
}

export interface DeployResult {
  deployment: DeploymentRecord;
  postDeploy: PostDeployStepResult[];
  pm2: Pm2CommandResult;
  removedDeployments: string[];
}

export interface DeployProgressEvent {
  stepId: string;
  label: string;
  state: "pending" | "running" | "done" | "failed" | "skipped";
  detail: string;
}

export interface RollbackResult {
  deployment: DeploymentRecord;
  pm2: Pm2CommandResult;
}

export interface LogReadResult {
  appName: ManagedAppName;
  path: string;
  lines: string[];
}
