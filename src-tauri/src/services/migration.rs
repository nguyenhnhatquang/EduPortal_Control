use std::time::Duration;

use crate::domain::{MigrationResult, Settings};

pub(crate) fn run_migration_request(settings: &Settings) -> Result<MigrationResult, String> {
    if settings.migration_url.trim().is_empty() {
        return Err("Migration URL is not configured.".to_string());
    }
    let url = reqwest::Url::parse(&settings.migration_url)
        .map_err(|err| format!("Invalid migration URL: {err}"))?;
    validate_local_migration_url(&url)?;
    let migration_key = settings.migration_key.trim();
    if migration_key.is_empty() {
        return Err("Migration key is not configured.".to_string());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(settings.migration_timeout_secs))
        .build()
        .map_err(|err| format!("Failed to create migration HTTP client: {err}"))?;
    let response = client
        .post(url.clone())
        .header("X-Migration-Key", migration_key)
        .send()
        .map_err(|err| format!("Migration request failed: {err}"))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|err| format!("Failed to read migration response: {err}"))?;
    let success = status.is_success();
    let message = if success {
        "Migration completed.".to_string()
    } else {
        format!("Migration failed with HTTP status {}", status.as_u16())
    };

    Ok(MigrationResult {
        success,
        url: settings.migration_url.clone(),
        status_code: Some(status.as_u16()),
        message,
        body,
    })
}

pub(crate) fn validate_local_migration_url(url: &reqwest::Url) -> Result<(), String> {
    let scheme = url.scheme();
    if scheme != "http" {
        return Err("Migration URL must use http.".to_string());
    }

    let host = url
        .host_str()
        .ok_or_else(|| "Migration URL must include a host.".to_string())?;
    if host != "localhost" {
        return Err("Migration URL must point to localhost.".to_string());
    }

    Ok(())
}
