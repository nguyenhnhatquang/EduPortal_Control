use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use tauri::AppHandle;

use crate::{
    deploy_portal_release_blocking,
    domain::{TelegramBotSettings, CADDY_APP_NAME},
    get_system_status_blocking, read_log_blocking, run_database_backup_blocking,
    services::pm2::{control_pm2_app_blocking, list_pm2_processes_blocking},
    storage::{load_settings, save_settings},
};

const TELEGRAM_API_BASE: &str = "https://api.telegram.org/bot";
const TELEGRAM_MESSAGE_LIMIT: usize = 3900;
const POLL_TIMEOUT_SECS: u64 = 30;
const CONFIRMATION_TTL_SECS: u64 = 120;

const ACTION_MENU: &str = "menu";
const ACTION_STATUS: &str = "status";
const ACTION_BACKUP_DB: &str = "backup_db";
const ACTION_DEPLOY_LATEST: &str = "deploy_latest";
const ACTION_LOGS_MENU: &str = "logs_menu";
const ACTION_PM2_MENU: &str = "pm2_menu";
const ACTION_LOG_PORTAL: &str = "log:Portal";
const ACTION_LOG_WEB_API: &str = "log:WebApi";
const ACTION_LOG_CADDY: &str = "log:Caddy";
const ACTION_RESTART_PORTAL: &str = "restart:Portal";
const ACTION_RESTART_WEB_API: &str = "restart:WebApi";
const ACTION_RESTART_CADDY: &str = "restart:Caddy";

#[derive(Debug, Clone, PartialEq, Eq)]
struct BotConfig {
    token: String,
    allowed_user_ids: Vec<i64>,
    allowed_chat_ids: Vec<i64>,
}

impl BotConfig {
    fn load(app: &AppHandle) -> Result<Option<Self>, String> {
        let settings = load_settings(app)?;
        Self::from_settings(&settings.telegram_bot)
    }

    fn from_settings(settings: &TelegramBotSettings) -> Result<Option<Self>, String> {
        if !settings.enabled {
            return Ok(None);
        }

        let token = settings.token.trim();
        if token.trim().is_empty() {
            return Ok(None);
        }

        let allowed_user_ids = parse_id_list(&settings.allowed_user_ids)?;
        let allowed_chat_ids = parse_id_list(&settings.allowed_chat_ids)?;

        Ok(Some(Self {
            token: token.trim().to_string(),
            allowed_user_ids,
            allowed_chat_ids,
        }))
    }

    fn is_authorized(&self, user_id: i64, chat_id: i64) -> bool {
        if !self.allowed_user_ids.contains(&user_id) {
            return false;
        }

        if self.allowed_chat_ids.is_empty() {
            chat_id == user_id
        } else {
            self.allowed_chat_ids.contains(&chat_id)
        }
    }

    fn uses_same_token(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
    callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Deserialize)]
struct Message {
    message_id: i64,
    chat: Chat,
    from: Option<User>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct User {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    id: String,
    from: User,
    message: Option<Message>,
    data: Option<String>,
}

struct TelegramClient {
    token: String,
    http: Client,
}

impl TelegramClient {
    fn new(token: String) -> Result<Self, String> {
        let http = Client::builder()
            .timeout(Duration::from_secs(POLL_TIMEOUT_SECS + 10))
            .build()
            .map_err(|err| format!("Failed to create Telegram HTTP client: {err}"))?;
        Ok(Self { token, http })
    }

    fn request<T: DeserializeOwned>(&self, method: &str, payload: Value) -> Result<T, String> {
        let url = format!("{TELEGRAM_API_BASE}{}/{method}", self.token);
        let body = serde_json::to_string(&payload)
            .map_err(|err| format!("Failed to serialize Telegram {method} payload: {err}"))?;
        let response = self
            .http
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .map_err(|err| format!("Telegram {method} request failed: {err}"))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|err| format!("Failed to read Telegram {method} response: {err}"))?;
        if !status.is_success() {
            return Err(format!(
                "Telegram {method} failed with HTTP status {}. {}",
                status.as_u16(),
                text.trim()
            ));
        }

        let api: ApiResponse<T> = serde_json::from_str(&text)
            .map_err(|err| format!("Failed to parse Telegram {method} response: {err}"))?;
        if api.ok {
            api.result
                .ok_or_else(|| format!("Telegram {method} response did not include a result."))
        } else {
            Err(api
                .description
                .unwrap_or_else(|| format!("Telegram {method} failed.")))
        }
    }

    fn get_updates(&self, offset: i64) -> Result<Vec<Update>, String> {
        self.request(
            "getUpdates",
            json!({
                "offset": offset,
                "timeout": POLL_TIMEOUT_SECS,
                "allowed_updates": ["message", "callback_query"],
            }),
        )
    }

    fn set_my_commands(&self) -> Result<(), String> {
        let _: bool = self.request(
            "setMyCommands",
            json!({
                "commands": [
                    { "command": "start", "description": "Open control panel" },
                    { "command": "menu", "description": "Show control menu" },
                    { "command": "status", "description": "Show app status" },
                    { "command": "backup", "description": "Run database backup" },
                    { "command": "logs", "description": "Choose a log target" },
                    { "command": "pm2", "description": "Choose a PM2 restart target" },
                    { "command": "deploy", "description": "Deploy latest Portal release" },
                    { "command": "help", "description": "Show commands" }
                ]
            }),
        )?;
        Ok(())
    }

    fn send_message(&self, chat_id: i64, text: impl Into<String>) -> Result<(), String> {
        let _: Value = self.request(
            "sendMessage",
            json!({
                "chat_id": chat_id,
                "text": clamp_message(text.into()),
            }),
        )?;
        Ok(())
    }

    fn send_menu(
        &self,
        chat_id: i64,
        text: impl Into<String>,
        reply_markup: Value,
    ) -> Result<(), String> {
        let _: Value = self.request(
            "sendMessage",
            json!({
                "chat_id": chat_id,
                "text": clamp_message(text.into()),
                "reply_markup": reply_markup,
            }),
        )?;
        Ok(())
    }

    fn edit_menu(
        &self,
        chat_id: i64,
        message_id: i64,
        text: impl Into<String>,
        reply_markup: Value,
    ) -> Result<(), String> {
        let _: Value = self.request(
            "editMessageText",
            json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "text": clamp_message(text.into()),
                "reply_markup": reply_markup,
            }),
        )?;
        Ok(())
    }

    fn answer_callback(
        &self,
        callback_query_id: &str,
        text: Option<&str>,
        show_alert: bool,
    ) -> Result<(), String> {
        let mut payload = json!({
            "callback_query_id": callback_query_id,
            "show_alert": show_alert,
        });
        if let Some(text) = text {
            payload["text"] = json!(text);
        }
        let _: bool = self.request("answerCallbackQuery", payload)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum PendingAction {
    DeployLatestRelease,
    RestartPm2(String),
}

impl PendingAction {
    fn label(&self) -> String {
        match self {
            Self::DeployLatestRelease => "deploy the latest Portal release".to_string(),
            Self::RestartPm2(app_name) => format!("restart {app_name} in PM2"),
        }
    }

    fn started_message(&self) -> String {
        match self {
            Self::DeployLatestRelease => "Portal release deploy started.".to_string(),
            Self::RestartPm2(app_name) => format!("PM2 restart started for {app_name}."),
        }
    }
}

struct PendingConfirmation {
    action: PendingAction,
    expires_at: Instant,
}

struct BotRunner {
    app: AppHandle,
    config: BotConfig,
    client: TelegramClient,
    offset: i64,
    pending: HashMap<String, PendingConfirmation>,
}

impl BotRunner {
    fn new(app: AppHandle, config: BotConfig) -> Result<Self, String> {
        let client = TelegramClient::new(config.token.clone())?;
        Ok(Self {
            app,
            config,
            client,
            offset: 0,
            pending: HashMap::new(),
        })
    }

    fn run(&mut self) {
        let _ = self.client.set_my_commands();

        loop {
            if !self.refresh_config() {
                return;
            }

            match self.client.get_updates(self.offset) {
                Ok(updates) => {
                    if !self.refresh_config() {
                        return;
                    }

                    for update in updates {
                        self.offset = update.update_id + 1;
                        self.handle_update(update);
                    }
                }
                Err(_) => thread::sleep(Duration::from_secs(5)),
            }
            self.cleanup_pending();
        }
    }

    fn refresh_config(&mut self) -> bool {
        match BotConfig::load(&self.app) {
            Ok(Some(config)) if self.config.uses_same_token(&config) => {
                self.config = config;
                true
            }
            _ => false,
        }
    }

    fn handle_update(&mut self, update: Update) {
        if let Some(message) = update.message {
            self.handle_message(message);
        }
        if let Some(callback) = update.callback_query {
            self.handle_callback(callback);
        }
    }

    fn handle_message(&mut self, message: Message) {
        let Some(user) = message.from else {
            return;
        };
        let text = message.text.unwrap_or_default();
        if is_command(&text, "start") {
            self.handle_start(message.chat.id, user.id);
            return;
        }

        if !self.config.is_authorized(user.id, message.chat.id) {
            let _ = self
                .client
                .send_message(message.chat.id, "Unauthorized Telegram user.");
            return;
        }

        if is_command(&text, "menu") {
            let _ = self
                .client
                .send_menu(message.chat.id, main_menu_text(), main_menu_keyboard());
        } else if is_command(&text, "help") {
            let _ = self
                .client
                .send_message(message.chat.id, command_help_text());
        } else if is_command(&text, "status") {
            self.send_status(message.chat.id);
        } else if is_command(&text, "backup") {
            self.run_backup(message.chat.id);
        } else if is_command(&text, "logs") {
            let _ = self.client.send_menu(
                message.chat.id,
                "Choose a log target.",
                logs_menu_keyboard(),
            );
        } else if is_command(&text, "pm2") {
            let _ = self.client.send_menu(
                message.chat.id,
                "Choose a PM2 restart target.",
                pm2_menu_keyboard(),
            );
        } else if is_command(&text, "deploy") {
            self.send_confirmation(message.chat.id, PendingAction::DeployLatestRelease);
        } else {
            let _ = self
                .client
                .send_menu(message.chat.id, main_menu_text(), main_menu_keyboard());
        }
    }

    fn handle_start(&self, chat_id: i64, user_id: i64) {
        let save_result = record_last_seen_ids(&self.app, user_id, chat_id);
        let ids_text = format!("Telegram IDs\nuser_id: {user_id}\nchat_id: {chat_id}");

        if self.config.is_authorized(user_id, chat_id) {
            let saved_suffix = if save_result.is_ok() {
                "\n\nIDs saved to Settings."
            } else {
                "\n\nIDs detected, but saving to Settings failed."
            };
            let _ = self.client.send_menu(
                chat_id,
                format!("{ids_text}{saved_suffix}\n\n{}", main_menu_text()),
                main_menu_keyboard(),
            );
            return;
        }

        let saved_line = if save_result.is_ok() {
            "IDs saved to Settings."
        } else {
            "IDs detected, but saving to Settings failed."
        };
        let _ = self.client.send_message(
            chat_id,
            format!(
                "{ids_text}\n\n{saved_line} Add this user_id to Allowed user IDs and this chat_id to Allowed chat IDs, save Settings, then send /start again."
            ),
        );
    }

    fn handle_callback(&mut self, callback: CallbackQuery) {
        let Some(message) = callback.message else {
            let _ = self.client.answer_callback(
                &callback.id,
                Some("Message is no longer available."),
                true,
            );
            return;
        };
        if !self.config.is_authorized(callback.from.id, message.chat.id) {
            let _ = self.client.answer_callback(
                &callback.id,
                Some("Unauthorized Telegram user."),
                true,
            );
            return;
        }

        let data = callback.data.unwrap_or_default();
        let _ = self.client.answer_callback(&callback.id, None, false);

        match data.as_str() {
            ACTION_MENU => {
                let _ = self.client.edit_menu(
                    message.chat.id,
                    message.message_id,
                    main_menu_text(),
                    main_menu_keyboard(),
                );
            }
            ACTION_STATUS => self.send_status(message.chat.id),
            ACTION_BACKUP_DB => self.run_backup(message.chat.id),
            ACTION_DEPLOY_LATEST => self.ask_confirmation(
                message.chat.id,
                message.message_id,
                PendingAction::DeployLatestRelease,
            ),
            ACTION_LOGS_MENU => {
                let _ = self.client.edit_menu(
                    message.chat.id,
                    message.message_id,
                    "Choose a log target.",
                    logs_menu_keyboard(),
                );
            }
            ACTION_PM2_MENU => {
                let _ = self.client.edit_menu(
                    message.chat.id,
                    message.message_id,
                    "Choose a PM2 restart target.",
                    pm2_menu_keyboard(),
                );
            }
            ACTION_LOG_PORTAL => self.send_logs(message.chat.id, "Portal"),
            ACTION_LOG_WEB_API => self.send_logs(message.chat.id, "WebApi"),
            ACTION_LOG_CADDY => self.send_logs(message.chat.id, CADDY_APP_NAME),
            ACTION_RESTART_PORTAL => self.ask_confirmation(
                message.chat.id,
                message.message_id,
                PendingAction::RestartPm2("Portal".to_string()),
            ),
            ACTION_RESTART_WEB_API => self.ask_confirmation(
                message.chat.id,
                message.message_id,
                PendingAction::RestartPm2("WebApi".to_string()),
            ),
            ACTION_RESTART_CADDY => self.ask_confirmation(
                message.chat.id,
                message.message_id,
                PendingAction::RestartPm2(CADDY_APP_NAME.to_string()),
            ),
            _ if data.starts_with("confirm:") => {
                let nonce = data.trim_start_matches("confirm:");
                self.run_confirmed_action(message.chat.id, message.message_id, nonce);
            }
            _ if data.starts_with("cancel:") => {
                let nonce = data.trim_start_matches("cancel:");
                self.pending.remove(nonce);
                let _ = self.client.edit_menu(
                    message.chat.id,
                    message.message_id,
                    main_menu_text(),
                    main_menu_keyboard(),
                );
            }
            _ => {
                let _ = self
                    .client
                    .send_message(message.chat.id, "Unknown bot action.");
            }
        }
    }

    fn ask_confirmation(&mut self, chat_id: i64, message_id: i64, action: PendingAction) {
        let (nonce, label) = self.add_pending_confirmation(action);
        let _ = self.client.edit_menu(
            chat_id,
            message_id,
            format!("Confirm {label}?"),
            confirmation_keyboard(&nonce),
        );
    }

    fn send_confirmation(&mut self, chat_id: i64, action: PendingAction) {
        let (nonce, label) = self.add_pending_confirmation(action);
        let _ = self.client.send_menu(
            chat_id,
            format!("Confirm {label}?"),
            confirmation_keyboard(&nonce),
        );
    }

    fn add_pending_confirmation(&mut self, action: PendingAction) -> (String, String) {
        let nonce = create_nonce();
        let label = action.label();
        self.pending.insert(
            nonce.clone(),
            PendingConfirmation {
                action,
                expires_at: Instant::now() + Duration::from_secs(CONFIRMATION_TTL_SECS),
            },
        );
        (nonce, label)
    }

    fn run_confirmed_action(&mut self, chat_id: i64, message_id: i64, nonce: &str) {
        let Some(pending) = self.pending.remove(nonce) else {
            let _ = self.client.edit_menu(
                chat_id,
                message_id,
                "Confirmation expired.",
                main_menu_keyboard(),
            );
            return;
        };

        if Instant::now() > pending.expires_at {
            let _ = self.client.edit_menu(
                chat_id,
                message_id,
                "Confirmation expired.",
                main_menu_keyboard(),
            );
            return;
        }

        let _ = self
            .client
            .send_message(chat_id, pending.action.started_message());
        match pending.action {
            PendingAction::DeployLatestRelease => self.run_deploy_latest(chat_id),
            PendingAction::RestartPm2(app_name) => self.run_pm2_restart(chat_id, app_name),
        }
    }

    fn send_status(&self, chat_id: i64) {
        let text = match build_status_text(self.app.clone()) {
            Ok(text) => text,
            Err(err) => format!("Status failed: {err}"),
        };
        let _ = self.client.send_message(chat_id, text);
    }

    fn run_backup(&self, chat_id: i64) {
        let _ = self
            .client
            .send_message(chat_id, "Database backup started.");
        let result = load_settings(&self.app)
            .and_then(|settings| run_database_backup_blocking(self.app.clone(), settings));

        let text = match result {
            Ok(result) if result.success => {
                let path = result
                    .path
                    .unwrap_or_else(|| "No backup path returned.".to_string());
                format!(
                    "Database backup completed.\nPath: {path}\n{}",
                    result.message
                )
            }
            Ok(result) => format!("Database backup failed: {}", result.message),
            Err(err) => format!("Database backup failed: {err}"),
        };
        let _ = self.client.send_message(chat_id, text);
    }

    fn run_deploy_latest(&self, chat_id: i64) {
        let result = load_settings(&self.app)
            .and_then(|settings| deploy_portal_release_blocking(self.app.clone(), settings));

        let text = match result {
            Ok(result) => format!(
                "Portal release deployed.\nDeployment: {}\nRelease: {}\nPM2: {}",
                result.deployment.id,
                result
                    .deployment
                    .release_tag
                    .unwrap_or_else(|| "unknown".to_string()),
                result.pm2.message
            ),
            Err(err) => format!("Portal release deploy failed: {err}"),
        };
        let _ = self.client.send_message(chat_id, text);
    }

    fn send_logs(&self, chat_id: i64, app_name: &str) {
        let text = match read_log_blocking(self.app.clone(), app_name.to_string(), 120) {
            Ok(result) if result.lines.is_empty() => {
                format!("No log lines found for {}.", result.app_name)
            }
            Ok(result) => {
                let body = result.lines.join("\n");
                format!(
                    "{} logs\nPath: {}\n\n{}",
                    result.app_name, result.path, body
                )
            }
            Err(err) => format!("Failed to read logs: {err}"),
        };
        let _ = self.client.send_message(chat_id, text);
    }

    fn run_pm2_restart(&self, chat_id: i64, app_name: String) {
        let result = control_pm2_app_blocking(app_name.clone(), "restart".to_string());
        let text = match result {
            Ok(result) if result.success => {
                format!("PM2 restart completed for {app_name}. {}", result.message)
            }
            Ok(result) => format!("PM2 restart failed for {app_name}: {}", result.message),
            Err(err) => format!("PM2 restart failed for {app_name}: {err}"),
        };
        let _ = self.client.send_message(chat_id, text);
    }

    fn cleanup_pending(&mut self) {
        let now = Instant::now();
        self.pending
            .retain(|_, confirmation| confirmation.expires_at > now);
    }
}

pub(crate) fn start(app: AppHandle) {
    let _ = thread::Builder::new()
        .name("educlasscontrol-telegram-bot".to_string())
        .spawn(move || loop {
            match BotConfig::load(&app) {
                Ok(Some(config)) => {
                    if let Ok(mut runner) = BotRunner::new(app.clone(), config) {
                        runner.run();
                    } else {
                        thread::sleep(Duration::from_secs(5));
                    }
                }
                Ok(None) | Err(_) => thread::sleep(Duration::from_secs(5)),
            }
        });
}

fn build_status_text(app: AppHandle) -> Result<String, String> {
    let status = get_system_status_blocking(app)?;
    let pm2 = list_pm2_processes_blocking().unwrap_or_default();
    let pm2_summary = if pm2.is_empty() {
        "No PM2 process data.".to_string()
    } else {
        pm2.iter()
            .map(|process| format!("{}: {}", process.name, process.status))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let active = status
        .active_deployment
        .map(|deployment| {
            let release = deployment
                .release_tag
                .map(|tag| format!(" ({tag})"))
                .unwrap_or_default();
            format!("{}{}", deployment.id, release)
        })
        .unwrap_or_else(|| "none".to_string());

    Ok(format!(
        "EduClassControl status\nVersion: {}\nOS: {}\nDeploy root: {} ({})\nActive deployment: {}\nNode: {}\nPM2: {}\nPM2 apps: {}",
        status.app_version,
        status.os,
        status.deploy_root,
        if status.deploy_root_exists { "exists" } else { "missing" },
        active,
        status.node_version.or(status.node_error).unwrap_or_else(|| "unknown".to_string()),
        status.pm2_version.or(status.pm2_error).unwrap_or_else(|| "unknown".to_string()),
        pm2_summary
    ))
}

fn record_last_seen_ids(app: &AppHandle, user_id: i64, chat_id: i64) -> Result<(), String> {
    let mut settings = load_settings(app)?;
    settings.telegram_bot.last_user_id = user_id.to_string();
    settings.telegram_bot.last_chat_id = chat_id.to_string();
    save_settings(app, settings)?;
    Ok(())
}

fn main_menu_text() -> &'static str {
    "EduClassControl remote admin"
}

fn command_help_text() -> &'static str {
    "EduClassControl commands\n/menu - Show control menu\n/status - Show app status\n/backup - Run database backup\n/logs - Choose a log target\n/pm2 - Choose a PM2 restart target\n/deploy - Confirm latest Portal deploy\n/help - Show commands"
}

fn main_menu_keyboard() -> Value {
    keyboard(vec![
        vec![
            button("Status", ACTION_STATUS),
            button("Backup DB", ACTION_BACKUP_DB),
        ],
        vec![button("Deploy latest Portal", ACTION_DEPLOY_LATEST)],
        vec![
            button("Logs", ACTION_LOGS_MENU),
            button("PM2 restart", ACTION_PM2_MENU),
        ],
    ])
}

fn logs_menu_keyboard() -> Value {
    keyboard(vec![
        vec![
            button("Portal", ACTION_LOG_PORTAL),
            button("WebApi", ACTION_LOG_WEB_API),
        ],
        vec![button("Caddy", ACTION_LOG_CADDY)],
        vec![button("Back", ACTION_MENU)],
    ])
}

fn pm2_menu_keyboard() -> Value {
    keyboard(vec![
        vec![button("Restart Portal", ACTION_RESTART_PORTAL)],
        vec![button("Restart WebApi", ACTION_RESTART_WEB_API)],
        vec![button("Restart Caddy", ACTION_RESTART_CADDY)],
        vec![button("Back", ACTION_MENU)],
    ])
}

fn confirmation_keyboard(nonce: &str) -> Value {
    keyboard(vec![vec![
        ("Confirm".to_string(), format!("confirm:{nonce}")),
        ("Cancel".to_string(), format!("cancel:{nonce}")),
    ]])
}

fn button(text: &str, callback_data: &str) -> (String, String) {
    (text.to_string(), callback_data.to_string())
}

fn keyboard(rows: Vec<Vec<(String, String)>>) -> Value {
    let rows: Vec<Vec<Value>> = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|(text, callback_data)| {
                    json!({
                        "text": text,
                        "callback_data": callback_data,
                    })
                })
                .collect()
        })
        .collect();
    json!({ "inline_keyboard": rows })
}

fn is_command(text: &str, command: &str) -> bool {
    let Some(first) = text.split_whitespace().next() else {
        return false;
    };
    first == format!("/{command}") || first.starts_with(&format!("/{command}@"))
}

fn clamp_message(text: String) -> String {
    if text.chars().count() <= TELEGRAM_MESSAGE_LIMIT {
        return text;
    }

    let tail: String = text
        .chars()
        .rev()
        .take(TELEGRAM_MESSAGE_LIMIT - 20)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("... output trimmed ...\n{tail}")
}

fn create_nonce() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{millis:x}")
}

fn parse_id_list(value: &str) -> Result<Vec<i64>, String> {
    let mut ids = Vec::new();
    for raw in
        value.split(|character: char| matches!(character, ',' | ';' | ' ' | '\n' | '\r' | '\t'))
    {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let id = trimmed
            .parse::<i64>()
            .map_err(|err| format!("Invalid Telegram numeric ID '{trimmed}': {err}"))?;
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_id_lists_with_common_separators() {
        assert_eq!(
            parse_id_list("123, 456;789").expect("ids"),
            vec![123, 456, 789]
        );
    }

    #[test]
    fn recognizes_plain_and_mentioned_commands() {
        assert!(is_command("/menu", "menu"));
        assert!(is_command("/status@EduClass_Control_bot", "status"));
        assert!(is_command("/deploy latest", "deploy"));
        assert!(!is_command("/deployment", "deploy"));
    }

    #[test]
    fn default_authorization_requires_private_chat() {
        let config = BotConfig {
            token: "token".to_string(),
            allowed_user_ids: vec![7],
            allowed_chat_ids: Vec::new(),
        };

        assert!(config.is_authorized(7, 7));
        assert!(!config.is_authorized(7, -100));
        assert!(!config.is_authorized(8, 8));
    }

    #[test]
    fn builds_config_from_settings() {
        let settings = TelegramBotSettings {
            enabled: true,
            token: " token ".to_string(),
            allowed_user_ids: "7, 8".to_string(),
            allowed_chat_ids: "-100".to_string(),
            last_user_id: String::new(),
            last_chat_id: String::new(),
        };

        let config = BotConfig::from_settings(&settings)
            .expect("settings")
            .expect("enabled config");

        assert_eq!(config.token, "token");
        assert_eq!(config.allowed_user_ids, vec![7, 8]);
        assert_eq!(config.allowed_chat_ids, vec![-100]);
    }

    #[test]
    fn starts_with_empty_allowlist_for_id_discovery() {
        let settings = TelegramBotSettings {
            enabled: true,
            token: "token".to_string(),
            allowed_user_ids: String::new(),
            allowed_chat_ids: String::new(),
            last_user_id: String::new(),
            last_chat_id: String::new(),
        };

        let config = BotConfig::from_settings(&settings)
            .expect("settings")
            .expect("enabled config");

        assert!(config.allowed_user_ids.is_empty());
        assert!(!config.is_authorized(7, 7));
    }
}
