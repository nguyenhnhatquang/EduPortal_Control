use crate::domain::{default_settings, sanitize_settings, DeploymentState, Settings};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "settings.json";
const STATE_FILE: &str = "deployments.json";

pub(crate) fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("Failed to resolve app config directory: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "Failed to create app config directory {}: {err}",
            dir.display()
        )
    })?;
    Ok(dir)
}

pub(crate) fn load_settings(app: &AppHandle) -> Result<Settings, String> {
    let path = config_dir(app)?.join(SETTINGS_FILE);
    if !path.exists() {
        return Ok(default_settings());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read settings {}: {err}", path.display()))?;
    let settings: Settings = serde_json::from_str(&contents)
        .map_err(|err| format!("Failed to parse settings {}: {err}", path.display()))?;
    Ok(sanitize_settings(settings))
}

pub(crate) fn save_settings(app: &AppHandle, settings: Settings) -> Result<Settings, String> {
    let path = config_dir(app)?.join(SETTINGS_FILE);
    let mut settings = sanitize_settings(settings);
    preserve_discovered_telegram_ids(&path, &mut settings);
    write_json(&path, &settings)?;
    Ok(settings)
}

fn preserve_discovered_telegram_ids(path: &Path, settings: &mut Settings) {
    if !settings.telegram_bot.last_user_id.is_empty()
        && !settings.telegram_bot.last_chat_id.is_empty()
    {
        return;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(existing) = serde_json::from_str::<Settings>(&contents) else {
        return;
    };
    let existing = sanitize_settings(existing);

    if settings.telegram_bot.last_user_id.is_empty() {
        settings.telegram_bot.last_user_id = existing.telegram_bot.last_user_id;
    }
    if settings.telegram_bot.last_chat_id.is_empty() {
        settings.telegram_bot.last_chat_id = existing.telegram_bot.last_chat_id;
    }
}

pub(crate) fn load_state(app: &AppHandle) -> Result<DeploymentState, String> {
    let path = config_dir(app)?.join(STATE_FILE);
    if !path.exists() {
        return Ok(DeploymentState::default());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read deployment state {}: {err}", path.display()))?;
    let state: DeploymentState = serde_json::from_str(&contents)
        .map_err(|err| format!("Failed to parse deployment state {}: {err}", path.display()))?;
    Ok(state)
}

pub(crate) fn save_state(app: &AppHandle, state: &DeploymentState) -> Result<(), String> {
    let path = config_dir(app)?.join(STATE_FILE);
    write_json(&path, state)
}

pub(crate) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|err| format!("Failed to create directory {}: {err}", parent.display()))?;
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| format!("Failed to serialize JSON for {}: {err}", path.display()))?;
    fs::write(path, json).map_err(|err| format!("Failed to write {}: {err}", path.display()))
}
