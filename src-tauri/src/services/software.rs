use crate::{
    domain::{SoftwareInstallResult, SoftwarePackageStatus},
    runtime::{command_version, display_path, node_command, npm_command, pm2_command},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) const SOFTWARE_NODEJS: &str = "nodejs";
pub(crate) const SOFTWARE_PM2: &str = "pm2";
pub(crate) const SOFTWARE_POSTGRESQL: &str = "postgresql";

const NODE_WINGET_ID: &str = "OpenJS.NodeJS.LTS";
const POSTGRESQL_WINGET_ID: &str = "PostgreSQL.PostgreSQL.18";
const POSTGRESQL_INSTALLER_URL: &str =
    "https://get.enterprisedb.com/postgresql/postgresql-18.3-3-windows-x64.exe";

struct CommandRunResult {
    success: bool,
    command: String,
    stdout: String,
    stderr: String,
}

pub(crate) fn list_software_packages() -> Vec<SoftwarePackageStatus> {
    [SOFTWARE_NODEJS, SOFTWARE_PM2, SOFTWARE_POSTGRESQL]
        .into_iter()
        .map(software_package_status)
        .collect()
}

pub(crate) fn install_software_package(package_id: &str) -> Result<SoftwareInstallResult, String> {
    let package_id = validate_software_package_id(package_id)?;
    if !cfg!(windows) {
        let status = software_package_status(package_id);
        return Ok(SoftwareInstallResult {
            package_id: package_id.to_string(),
            attempted: false,
            skipped: true,
            success: true,
            command: install_command_label(package_id),
            stdout: String::new(),
            stderr: String::new(),
            message: "Software installation is automated only on Windows Server.".to_string(),
            path_entries_added: Vec::new(),
            status,
        });
    }

    match package_id {
        SOFTWARE_NODEJS => install_nodejs(),
        SOFTWARE_POSTGRESQL => install_postgresql(),
        SOFTWARE_PM2 => install_pm2(),
        _ => unreachable!("software package id is already validated"),
    }
}

pub(crate) fn validate_software_package_id(package_id: &str) -> Result<&'static str, String> {
    match package_id.trim().to_lowercase().as_str() {
        SOFTWARE_NODEJS => Ok(SOFTWARE_NODEJS),
        SOFTWARE_PM2 => Ok(SOFTWARE_PM2),
        SOFTWARE_POSTGRESQL => Ok(SOFTWARE_POSTGRESQL),
        _ => Err("Unknown software package. Expected nodejs, pm2, or postgresql.".to_string()),
    }
}

fn software_package_status(package_id: &str) -> SoftwarePackageStatus {
    match package_id {
        SOFTWARE_NODEJS => {
            let path_entries = discover_node_path_entries();
            let executable_name = if cfg!(windows) { "node.exe" } else { "node" };
            let (version, error, executable) = version_with_fallback(
                node_command(),
                &["--version"],
                &path_entries,
                executable_name,
            );
            build_status(
                SOFTWARE_NODEJS,
                "Node.js",
                version,
                error,
                executable,
                path_entries,
            )
        }
        SOFTWARE_PM2 => {
            let path_entries = discover_pm2_path_entries();
            let executable_name = if cfg!(windows) { "pm2.cmd" } else { "pm2" };
            let (version, error, executable) = version_with_fallback(
                pm2_command(),
                &["--version"],
                &path_entries,
                executable_name,
            );
            build_status(
                SOFTWARE_PM2,
                "PM2",
                version,
                error,
                executable,
                path_entries,
            )
        }
        SOFTWARE_POSTGRESQL => {
            let path_entries = discover_postgresql_path_entries();
            let command = if cfg!(windows) { "psql.exe" } else { "psql" };
            let executable_name = if cfg!(windows) { "psql.exe" } else { "psql" };
            let (version, error, executable) =
                version_with_fallback(command, &["--version"], &path_entries, executable_name);
            build_status(
                SOFTWARE_POSTGRESQL,
                "PostgreSQL",
                version,
                error,
                executable,
                path_entries,
            )
        }
        _ => build_status(
            package_id,
            package_id,
            None,
            Some("Unknown software package.".to_string()),
            package_id.to_string(),
            Vec::new(),
        ),
    }
}

fn build_status(
    id: &str,
    name: &str,
    version: Option<String>,
    error: Option<String>,
    executable: String,
    path_entries: Vec<PathBuf>,
) -> SoftwarePackageStatus {
    let missing_path_entries = missing_path_entries(&path_entries);

    SoftwarePackageStatus {
        id: id.to_string(),
        name: name.to_string(),
        installed: version.is_some(),
        version,
        error,
        executable,
        path_entries: path_entries
            .iter()
            .map(|entry| display_path(entry))
            .collect(),
        missing_path_entries,
    }
}

fn install_nodejs() -> Result<SoftwareInstallResult, String> {
    let before = software_package_status(SOFTWARE_NODEJS);
    if before.installed {
        return already_installed_result(SOFTWARE_NODEJS, "Node.js");
    }

    if winget_available() {
        install_winget_package(SOFTWARE_NODEJS, "Node.js", NODE_WINGET_ID)
    } else {
        install_nodejs_from_official_msi()
    }
}

fn install_postgresql() -> Result<SoftwareInstallResult, String> {
    let before = software_package_status(SOFTWARE_POSTGRESQL);
    if before.installed {
        return already_installed_result(SOFTWARE_POSTGRESQL, "PostgreSQL");
    }

    if winget_available() {
        install_winget_package(SOFTWARE_POSTGRESQL, "PostgreSQL", POSTGRESQL_WINGET_ID)
    } else {
        install_postgresql_from_enterprisedb()
    }
}

fn install_winget_package(
    package_id: &'static str,
    package_name: &str,
    winget_id: &str,
) -> Result<SoftwareInstallResult, String> {
    let args = vec![
        "install".to_string(),
        "--id".to_string(),
        winget_id.to_string(),
        "--exact".to_string(),
        "--silent".to_string(),
        "--accept-source-agreements".to_string(),
        "--accept-package-agreements".to_string(),
    ];
    let run = run_command("winget", &args);
    let mut path_entries_added = Vec::new();
    if run.success {
        path_entries_added = ensure_package_path(package_id)?;
    }
    let status = software_package_status(package_id);
    let success = run.success && status.installed;
    let message = if success {
        format!("{package_name} installed and PATH checked.")
    } else if run.success {
        format!(
            "{package_name} installer finished, but the command was not found yet. Reopen the app or check installer output."
        )
    } else {
        format!("{package_name} installation failed.")
    };

    Ok(SoftwareInstallResult {
        package_id: package_id.to_string(),
        attempted: true,
        skipped: false,
        success,
        command: run.command,
        stdout: run.stdout,
        stderr: run.stderr,
        message,
        path_entries_added,
        status,
    })
}

fn install_nodejs_from_official_msi() -> Result<SoftwareInstallResult, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$versions = Invoke-RestMethod -Uri 'https://nodejs.org/dist/index.json'
$version = $versions | Where-Object { $_.lts -and ($_.files -contains 'win-x64-msi') } | Select-Object -First 1
if ($null -eq $version) { throw 'Could not find a Node.js LTS Windows x64 MSI package.' }
$fileName = "node-$($version.version)-x64.msi"
$url = "https://nodejs.org/dist/$($version.version)/$fileName"
$installer = Join-Path $env:TEMP $fileName
Invoke-WebRequest -Uri $url -OutFile $installer -UseBasicParsing
$process = Start-Process -FilePath 'msiexec.exe' -ArgumentList @('/i', $installer, '/qn', '/norestart') -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Node.js installer exited with code $($process.ExitCode)." }
"Installed Node.js $($version.version) from $url"
"#;
    let run = run_powershell_script("Install Node.js LTS MSI from nodejs.org", script);
    let mut path_entries_added = Vec::new();
    if run.success {
        path_entries_added = ensure_package_path(SOFTWARE_NODEJS)?;
    }
    let status = software_package_status(SOFTWARE_NODEJS);
    let success = run.success && status.installed;
    let message = if success {
        "Node.js installed from nodejs.org and PATH checked.".to_string()
    } else if run.success {
        "Node.js installer finished, but node.exe was not found yet. Reopen the app or check installer output."
            .to_string()
    } else {
        "Node.js installation failed.".to_string()
    };

    Ok(SoftwareInstallResult {
        package_id: SOFTWARE_NODEJS.to_string(),
        attempted: true,
        skipped: false,
        success,
        command: run.command,
        stdout: run.stdout,
        stderr: run.stderr,
        message,
        path_entries_added,
        status,
    })
}

fn install_postgresql_from_enterprisedb() -> Result<SoftwareInstallResult, String> {
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$url = {url}
$fileName = Split-Path -Leaf $url
$installer = Join-Path $env:TEMP $fileName
Invoke-WebRequest -Uri $url -OutFile $installer -UseBasicParsing
$process = Start-Process -FilePath $installer -Wait -PassThru
if ($process.ExitCode -ne 0) {{ throw "PostgreSQL installer exited with code $($process.ExitCode)." }}
"PostgreSQL installer completed from $url"
"#,
        url = ps_single_quote(POSTGRESQL_INSTALLER_URL)
    );
    let run = run_powershell_script(
        "Download and run PostgreSQL 18 installer from EnterpriseDB",
        &script,
    );
    let mut path_entries_added = Vec::new();
    if run.success {
        path_entries_added = ensure_package_path(SOFTWARE_POSTGRESQL)?;
    }
    let status = software_package_status(SOFTWARE_POSTGRESQL);
    let success = run.success && status.installed;
    let message = if success {
        "PostgreSQL installer completed and PATH checked.".to_string()
    } else if run.success {
        "PostgreSQL installer closed, but psql.exe was not found yet. Reopen the app or check the installer choices."
            .to_string()
    } else {
        "PostgreSQL installation failed or was cancelled.".to_string()
    };

    Ok(SoftwareInstallResult {
        package_id: SOFTWARE_POSTGRESQL.to_string(),
        attempted: true,
        skipped: false,
        success,
        command: run.command,
        stdout: run.stdout,
        stderr: run.stderr,
        message,
        path_entries_added,
        status,
    })
}

fn install_pm2() -> Result<SoftwareInstallResult, String> {
    let before = software_package_status(SOFTWARE_PM2);
    if before.installed {
        return already_installed_result(SOFTWARE_PM2, "PM2");
    }

    let node_status = software_package_status(SOFTWARE_NODEJS);
    if !node_status.installed {
        let status = software_package_status(SOFTWARE_PM2);
        return Ok(SoftwareInstallResult {
            package_id: SOFTWARE_PM2.to_string(),
            attempted: false,
            skipped: false,
            success: false,
            command: install_command_label(SOFTWARE_PM2),
            stdout: String::new(),
            stderr: node_status.error.unwrap_or_default(),
            message: "Install Node.js before installing PM2.".to_string(),
            path_entries_added: Vec::new(),
            status,
        });
    }

    let _ = ensure_package_path(SOFTWARE_NODEJS)?;
    let npm = npm_executable();
    let args = vec!["install".to_string(), "-g".to_string(), "pm2".to_string()];
    let run = run_command(&npm, &args);
    let mut path_entries_added = Vec::new();
    if run.success {
        path_entries_added = ensure_package_path(SOFTWARE_PM2)?;
    }
    let status = software_package_status(SOFTWARE_PM2);
    let success = run.success && status.installed;
    let message = if success {
        "PM2 installed globally and PATH checked.".to_string()
    } else if run.success {
        "npm finished, but pm2.cmd was not found yet. Reopen the app or check npm output."
            .to_string()
    } else {
        "PM2 installation failed.".to_string()
    };

    Ok(SoftwareInstallResult {
        package_id: SOFTWARE_PM2.to_string(),
        attempted: true,
        skipped: false,
        success,
        command: run.command,
        stdout: run.stdout,
        stderr: run.stderr,
        message,
        path_entries_added,
        status,
    })
}

fn already_installed_result(
    package_id: &'static str,
    package_name: &str,
) -> Result<SoftwareInstallResult, String> {
    let path_entries_added = ensure_package_path(package_id)?;
    let status = software_package_status(package_id);
    let message = if path_entries_added.is_empty() {
        format!("{package_name} is already installed. PATH is already configured.")
    } else {
        format!("{package_name} is already installed. PATH was updated.")
    };

    Ok(SoftwareInstallResult {
        package_id: package_id.to_string(),
        attempted: false,
        skipped: true,
        success: true,
        command: install_command_label(package_id),
        stdout: String::new(),
        stderr: String::new(),
        message,
        path_entries_added,
        status,
    })
}

fn install_command_label(package_id: &str) -> String {
    match package_id {
        SOFTWARE_NODEJS => command_label(
            "winget",
            &[
                "install".to_string(),
                "--id".to_string(),
                NODE_WINGET_ID.to_string(),
                "--exact".to_string(),
                "--silent".to_string(),
            ],
        ),
        SOFTWARE_POSTGRESQL => command_label(
            "winget",
            &[
                "install".to_string(),
                "--id".to_string(),
                POSTGRESQL_WINGET_ID.to_string(),
                "--exact".to_string(),
                "--silent".to_string(),
            ],
        ),
        SOFTWARE_PM2 => command_label(
            npm_command(),
            &["install".to_string(), "-g".to_string(), "pm2".to_string()],
        ),
        _ => String::new(),
    }
}

fn version_with_fallback(
    command: &str,
    args: &[&str],
    fallback_dirs: &[PathBuf],
    executable_name: &str,
) -> (Option<String>, Option<String>, String) {
    let initial = command_version(command, args);
    if initial.ok.is_some() {
        return (initial.ok, None, command.to_string());
    }

    for dir in fallback_dirs {
        let executable = dir.join(executable_name);
        if !executable.exists() {
            continue;
        }

        let executable_label = display_path(&executable);
        let result = command_version(&executable_label, args);
        if result.ok.is_some() {
            return (result.ok, None, executable_label);
        }
    }

    (None, initial.err, command.to_string())
}

fn npm_executable() -> String {
    let npm = npm_command();
    if command_version(npm, &["--version"]).ok.is_some() {
        return npm.to_string();
    }

    for dir in discover_node_path_entries() {
        let executable = dir.join(if cfg!(windows) { "npm.cmd" } else { "npm" });
        if executable.exists() {
            return display_path(&executable);
        }
    }

    npm.to_string()
}

fn discover_node_path_entries() -> Vec<PathBuf> {
    let mut entries = command_parent_dirs(if cfg!(windows) { "node.exe" } else { "node" });
    if cfg!(windows) {
        push_unique_path(&mut entries, PathBuf::from(r"C:\Program Files\nodejs"));
    }
    entries
}

fn discover_pm2_path_entries() -> Vec<PathBuf> {
    let mut entries = command_parent_dirs(if cfg!(windows) { "pm2.cmd" } else { "pm2" });
    if let Some(entry) = npm_global_bin_entry() {
        push_unique_path(&mut entries, entry);
    }
    entries
}

fn discover_postgresql_path_entries() -> Vec<PathBuf> {
    let mut entries = command_parent_dirs(if cfg!(windows) { "psql.exe" } else { "psql" });
    if cfg!(windows) {
        for entry in default_postgresql_bin_dirs() {
            push_unique_path(&mut entries, entry);
        }
    }
    entries
}

fn command_parent_dirs(command: &str) -> Vec<PathBuf> {
    let resolver = if cfg!(windows) { "where.exe" } else { "which" };
    let Ok(output) = Command::new(resolver).arg(command).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let path = PathBuf::from(line.trim());
            path.parent().map(Path::to_path_buf)
        })
        .fold(Vec::new(), |mut entries, path| {
            push_unique_path(&mut entries, path);
            entries
        })
}

fn npm_global_bin_entry() -> Option<PathBuf> {
    let prefix = command_stdout(&npm_executable(), &["config", "get", "prefix"])
        .filter(|value| !value.eq_ignore_ascii_case("undefined"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("APPDATA").map(|value| PathBuf::from(value).join("npm")));

    prefix.map(|path| {
        if cfg!(windows) {
            path
        } else {
            path.join("bin")
        }
    })
}

fn default_postgresql_bin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for root in [
        PathBuf::from(r"C:\Program Files\PostgreSQL"),
        PathBuf::from(r"C:\Program Files (x86)\PostgreSQL"),
    ] {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let bin_dir = entry.path().join("bin");
            if bin_dir.join("psql.exe").exists() {
                dirs.push(bin_dir);
            }
        }
    }

    dirs.sort_by(|left, right| compare_postgres_bin_dirs(right, left));
    dirs
}

fn compare_postgres_bin_dirs(left: &Path, right: &Path) -> std::cmp::Ordering {
    let left_version = postgres_dir_version(left);
    let right_version = postgres_dir_version(right);
    left_version.cmp(&right_version)
}

fn postgres_dir_version(path: &Path) -> Vec<u32> {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

fn ensure_package_path(package_id: &str) -> Result<Vec<String>, String> {
    let entries = match package_id {
        SOFTWARE_NODEJS => discover_node_path_entries(),
        SOFTWARE_PM2 => discover_pm2_path_entries(),
        SOFTWARE_POSTGRESQL => discover_postgresql_path_entries(),
        _ => Vec::new(),
    };

    ensure_path_entries(&entries)
}

fn ensure_path_entries(entries: &[PathBuf]) -> Result<Vec<String>, String> {
    let entries: Vec<PathBuf> =
        entries
            .iter()
            .filter(|entry| entry.is_dir())
            .fold(Vec::new(), |mut unique, entry| {
                push_unique_path(&mut unique, entry.clone());
                unique
            });

    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let missing_before = missing_path_entries(&entries);
    if missing_before.is_empty() {
        return Ok(Vec::new());
    }

    append_to_process_path(&entries);
    if cfg!(windows) {
        persist_windows_path_entries(&entries)
    } else {
        Ok(missing_before)
    }
}

fn missing_path_entries(entries: &[PathBuf]) -> Vec<String> {
    let path = env::var("PATH").unwrap_or_default();
    entries
        .iter()
        .filter(|entry| !path_env_contains(&path, entry))
        .map(|entry| display_path(entry))
        .collect()
}

fn append_to_process_path(entries: &[PathBuf]) {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut current = env::var("PATH").unwrap_or_default();
    for entry in entries {
        if path_env_contains(&current, entry) {
            continue;
        }
        if !current.is_empty() {
            current.push(separator);
        }
        current.push_str(&display_path(entry));
    }
    env::set_var("PATH", current);
}

fn persist_windows_path_entries(entries: &[PathBuf]) -> Result<Vec<String>, String> {
    let path_literals = entries
        .iter()
        .map(|entry| ps_single_quote(&display_path(entry)))
        .collect::<Vec<_>>()
        .join(", ");
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
$entries = @({path_literals})
function Normalize-PathValue([string]$value) {{
  return $value.Trim().TrimEnd([char[]]@('\','/'))
}}
function Add-PathEntries([string]$target) {{
  $current = [Environment]::GetEnvironmentVariable('Path', $target)
  if ($null -eq $current) {{ $current = '' }}
  $parts = @($current -split ';' | Where-Object {{ $_.Trim().Length -gt 0 }})
  $added = @()
  foreach ($entry in $entries) {{
    $matching = @($parts | Where-Object {{ (Normalize-PathValue $_) -ieq (Normalize-PathValue $entry) }})
    if ($matching.Count -eq 0) {{
      $parts += $entry
      $added += $entry
    }}
  }}
  if ($added.Count -gt 0) {{
    [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), $target)
  }}
  foreach ($item in $added) {{ "ADDED:$item" }}
}}
try {{
  Add-PathEntries 'Machine'
}} catch {{
  Add-PathEntries 'User'
}}
"#
    );

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|err| format!("Failed to update Windows PATH: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(if stderr.is_empty() {
            format!(
                "PowerShell PATH update exited with status {}",
                output.status
            )
        } else {
            stderr
        });
    }

    Ok(stdout
        .lines()
        .filter_map(|line| line.strip_prefix("ADDED:").map(ToOwned::to_owned))
        .collect())
}

fn path_env_contains(path: &str, entry: &Path) -> bool {
    let expected = normalize_path_value(&display_path(entry));
    let separator = if cfg!(windows) { ';' } else { ':' };
    path.split(separator)
        .map(normalize_path_value)
        .any(|value| value == expected)
}

fn normalize_path_value(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches(['\\', '/']);
    if cfg!(windows) {
        trimmed.to_lowercase()
    } else {
        trimmed.to_string()
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    let normalized = normalize_path_value(&display_path(&path));
    let exists = paths
        .iter()
        .any(|existing| normalize_path_value(&display_path(existing)) == normalized);
    if !exists {
        paths.push(path);
    }
}

fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

fn winget_available() -> bool {
    command_version("winget", &["--version"]).ok.is_some()
}

fn run_powershell_script(label: &str, script: &str) -> CommandRunResult {
    let args = vec![
        "-NoProfile".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-Command".to_string(),
        script.to_string(),
    ];
    let mut result = run_command("powershell.exe", &args);
    result.command = label.to_string();
    result
}

fn run_command(command: &str, args: &[String]) -> CommandRunResult {
    let command_label = command_label(command, args);
    match Command::new(command).args(args).output() {
        Ok(output) => CommandRunResult {
            success: output.status.success(),
            command: command_label,
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        },
        Err(err) => CommandRunResult {
            success: false,
            command: command_label,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

fn command_label(command: &str, args: &[String]) -> String {
    std::iter::once(quote_command_arg(command))
        .chain(args.iter().map(|arg| quote_command_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_command_arg(value: &str) -> String {
    if value.is_empty() || value.chars().any(char::is_whitespace) || value.contains('"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_software_package_ids() {
        assert_eq!(
            validate_software_package_id("nodejs").unwrap(),
            SOFTWARE_NODEJS
        );
        assert_eq!(validate_software_package_id("PM2").unwrap(), SOFTWARE_PM2);
        assert_eq!(
            validate_software_package_id("postgresql").unwrap(),
            SOFTWARE_POSTGRESQL
        );
        assert!(validate_software_package_id("powershell").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn path_matching_is_case_insensitive_on_windows() {
        let existing = r"C:\Program Files\nodejs;C:\Windows\System32";
        assert!(path_env_contains(
            existing,
            &PathBuf::from(r"c:\program files\nodejs\")
        ));
    }
}
