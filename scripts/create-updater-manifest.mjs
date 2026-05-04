import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = resolve(fileURLToPath(new URL("..", import.meta.url)));
const tauriConfigPath = join(rootDir, "src-tauri", "tauri.conf.json");
const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, "utf8"));

const version = process.env.UPDATER_VERSION || tauriConfig.version;
const repository = process.env.GITHUB_REPOSITORY || "nguyenhnhatquang/EduPortal_Control";
const releaseTag = process.env.GITHUB_REF_NAME || `v${version}`;
const target = process.env.UPDATER_TARGET || "windows-x86_64";
const notes = process.env.UPDATER_NOTES || `EduPortal_Control manager ${version}`;
const pubDate = process.env.UPDATER_PUB_DATE || new Date().toISOString();

const configuredBundleDir = process.env.TAURI_NSIS_BUNDLE_DIR
  ? [resolve(rootDir, process.env.TAURI_NSIS_BUNDLE_DIR)]
  : [];
const candidateBundleDirs = [
  ...configuredBundleDir,
  join(rootDir, "src-tauri", "target", "x86_64-pc-windows-msvc", "release", "bundle", "nsis"),
  join(rootDir, "src-tauri", "target", "release", "bundle", "nsis"),
];

function findSetupExe(bundleDir) {
  return readdirSync(bundleDir)
    .filter((name) => name.toLowerCase().endsWith(".exe"))
    .filter((name) => !name.toLowerCase().includes("uninstall"))
    .sort((left, right) => statSync(join(bundleDir, right)).size - statSync(join(bundleDir, left)).size)[0];
}

function findSignaturePath(bundleDir, exeName) {
  const exePath = join(bundleDir, exeName);
  if (existsSync(`${exePath}.sig`)) {
    return `${exePath}.sig`;
  }
  return readdirSync(bundleDir)
    .filter((name) => name.toLowerCase().endsWith(".exe.sig"))
    .map((name) => join(bundleDir, name))[0];
}

const bundle = candidateBundleDirs
  .filter((candidate) => existsSync(candidate))
  .map((bundleDir) => {
    const exeName = process.env.UPDATER_EXE || findSetupExe(bundleDir);
    if (!exeName) return null;
    const sigPath = findSignaturePath(bundleDir, exeName);
    if (!sigPath || !existsSync(sigPath)) return null;
    return {
      bundleDir,
      exeName,
      exePath: join(bundleDir, exeName),
      sigPath,
    };
  })
  .find(Boolean);

if (!bundle) {
  throw new Error(`No signed NSIS setup was found. Checked: ${candidateBundleDirs.join(", ")}`);
}

const { bundleDir, exePath, sigPath } = bundle;
const assetBaseUrl =
  process.env.UPDATER_ASSET_BASE_URL ||
  `https://github.com/${repository}/releases/download/${encodeURIComponent(releaseTag)}`;
const manifestPath = resolve(bundleDir, process.env.UPDATER_MANIFEST_NAME || "latest.json");
const manifest = {
  version,
  notes,
  pub_date: pubDate,
  platforms: {
    [target]: {
      signature: readFileSync(sigPath, "utf8").trim(),
      url: `${assetBaseUrl.replace(/\/$/, "")}/${encodeURIComponent(basename(exePath))}`,
    },
  },
};

writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Updater manifest written to ${manifestPath}`);
