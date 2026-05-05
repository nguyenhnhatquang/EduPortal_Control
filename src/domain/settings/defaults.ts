import type {
  DatabaseBackupScheduleDay,
  DatabaseBackupScheduleFrequency,
  Settings,
} from "../../types";

export const backupScheduleFrequencies: DatabaseBackupScheduleFrequency[] = ["daily", "weekly"];

export const backupScheduleDays: DatabaseBackupScheduleDay[] = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
];

export const fallbackSettings: Settings = {
  deployRoot: "C:\\deploy",
  retention: 5,
  portalEnv: {
    NODE_ENV: "production",
    PORT: "8080",
    BODY_SIZE_LIMIT: "10M",
  },
  webApiEnv: {
    DOTNET_ENVIRONMENT: "Production",
    ASPNETCORE_URLS: "http://localhost:7000",
  },
  portalInstallDependencies: true,
  portalAssetCopy: {
    enabled: true,
    source: "kanji",
    destination: "build/client/kanji",
  },
  migrationUrl: "",
  migrationKey: "",
  migrationTimeoutSecs: 120,
  database: {
    host: "localhost",
    port: 5432,
    database: "eduportal_control",
    username: "postgres",
    password: "",
    binDir: "",
    backupDir: "backups/postgresql",
    backupRetention: 14,
    backupSchedule: {
      enabled: false,
      frequency: "daily",
      time: "02:00",
      dayOfWeek: "Monday",
    },
  },
  caddy: {
    enabled: true,
    installDir: "caddy",
    configPath: "caddy/Caddyfile",
    config: `:80 {
    encode gzip

    handle /api/* {
        reverse_proxy localhost:7000
    }

    reverse_proxy localhost:8080
}
`,
  },
  portalRelease: {
    enabled: true,
    owner: "nguyenhnhatquang",
    repo: "EduPortal_DiemSensei",
    token: "",
    assetNamePrefix: "EduPortal_DiemSensei_",
    assetNameSuffix: ".zip",
  },
  telegramBot: {
    enabled: false,
    token: "",
    allowedUserIds: "",
    allowedChatIds: "",
    lastUserId: "",
    lastChatId: "",
  },
};
