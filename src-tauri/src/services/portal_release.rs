use std::{fs, path::PathBuf, time::Duration};

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::{
    domain::{PortalReleaseCheckResult, PortalReleaseInfo, Settings},
    runtime::display_path,
};

const GITHUB_API_VERSION: &str = "2022-11-28";
const GITHUB_ACCEPT_JSON: &str = "application/vnd.github+json";
const GITHUB_ACCEPT_ASSET: &str = "application/octet-stream";
const GITHUB_USER_AGENT: &str = "EduPortal-Control";

#[derive(Debug, Clone)]
pub(crate) struct PortalReleaseDownload {
    pub zip_path: PathBuf,
    pub release: PortalReleaseInfo,
}

#[derive(Debug, Clone)]
struct PortalReleaseAsset {
    info: PortalReleaseInfo,
    api_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    body: Option<String>,
    html_url: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    size: u64,
    url: String,
    digest: Option<String>,
}

pub(crate) fn check_latest_release(
    settings: &Settings,
    active_release_tag: Option<String>,
) -> Result<PortalReleaseCheckResult, String> {
    let latest = fetch_latest_release(settings)?.info;
    let update_available = active_release_tag.as_deref() != Some(latest.tag_name.as_str());
    let message = if update_available {
        format!("Portal release {} is available.", latest.tag_name)
    } else {
        format!("Portal release {} is already active.", latest.tag_name)
    };

    Ok(PortalReleaseCheckResult {
        update_available,
        active_release_tag,
        latest: Some(latest),
        message,
    })
}

pub(crate) fn download_latest_release(
    app: &AppHandle,
    settings: &Settings,
) -> Result<PortalReleaseDownload, String> {
    let mut release = fetch_latest_release(settings)?;
    let client = github_client()?;
    let response = client
        .get(&release.api_url)
        .headers(github_headers(settings, GITHUB_ACCEPT_ASSET)?)
        .send()
        .map_err(|err| format!("Failed to download Portal release asset: {err}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!(
            "GitHub release asset download failed with HTTP status {}. {}",
            status.as_u16(),
            body.trim()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|err| format!("Failed to read Portal release asset: {err}"))?;
    let actual_digest = format!("sha256:{}", sha256_hex(&bytes));
    verify_asset_digest(release.info.asset_digest.as_deref(), &actual_digest)?;
    release.info.asset_digest = Some(actual_digest);

    let releases_dir = app
        .path()
        .app_cache_dir()
        .map_err(|err| format!("Failed to resolve app cache directory: {err}"))?
        .join("portal-releases");
    fs::create_dir_all(&releases_dir).map_err(|err| {
        format!(
            "Failed to create Portal release cache {}: {err}",
            releases_dir.display()
        )
    })?;

    let zip_path = releases_dir.join(safe_download_file_name(
        &release.info.tag_name,
        &release.info.asset_name,
    ));
    fs::write(&zip_path, bytes).map_err(|err| {
        format!(
            "Failed to write Portal release zip {}: {err}",
            zip_path.display()
        )
    })?;

    Ok(PortalReleaseDownload {
        zip_path,
        release: release.info,
    })
}

fn fetch_latest_release(settings: &Settings) -> Result<PortalReleaseAsset, String> {
    validate_settings(settings)?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        settings.portal_release.owner, settings.portal_release.repo
    );
    let client = github_client()?;
    let response = client
        .get(url)
        .headers(github_headers(settings, GITHUB_ACCEPT_JSON)?)
        .send()
        .map_err(|err| format!("Failed to check Portal release: {err}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!(
            "GitHub latest release request failed with HTTP status {}. {}",
            status.as_u16(),
            body.trim()
        ));
    }

    let body = response
        .text()
        .map_err(|err| format!("Failed to read GitHub latest release response: {err}"))?;
    parse_latest_release(&body, settings)
}

fn validate_settings(settings: &Settings) -> Result<(), String> {
    let release = &settings.portal_release;
    if !release.enabled {
        return Err("Portal release updates are disabled.".to_string());
    }
    if release.owner.trim().is_empty() || release.repo.trim().is_empty() {
        return Err("Portal release repository is not configured.".to_string());
    }
    if release.token.trim().is_empty() {
        return Err("GitHub PAT is not configured for Portal releases.".to_string());
    }
    Ok(())
}

fn github_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|err| format!("Failed to create GitHub HTTP client: {err}"))
}

fn github_headers(settings: &Settings, accept: &'static str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(accept));
    headers.insert(USER_AGENT, HeaderValue::from_static(GITHUB_USER_AGENT));
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static(GITHUB_API_VERSION),
    );
    let auth = format!("Bearer {}", settings.portal_release.token.trim());
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth)
            .map_err(|err| format!("Invalid GitHub PAT header value: {err}"))?,
    );
    Ok(headers)
}

fn parse_latest_release(body: &str, settings: &Settings) -> Result<PortalReleaseAsset, String> {
    let release: GithubRelease = serde_json::from_str(body)
        .map_err(|err| format!("Failed to parse GitHub release response: {err}"))?;
    let asset = select_release_asset(&release, settings).ok_or_else(|| {
        format!(
            "Release {} has no asset matching {}*{}.",
            release.tag_name,
            settings.portal_release.asset_name_prefix,
            settings.portal_release.asset_name_suffix
        )
    })?;
    let asset_name = asset.name.clone();
    let asset_size = asset.size;
    let asset_digest = asset.digest.clone();
    let api_url = asset.url.clone();

    Ok(PortalReleaseAsset {
        info: PortalReleaseInfo {
            tag_name: release.tag_name,
            release_name: release.name,
            published_at: release.published_at,
            body: release.body,
            html_url: release.html_url,
            asset_name,
            asset_size,
            asset_digest,
        },
        api_url,
    })
}

fn select_release_asset<'a>(
    release: &'a GithubRelease,
    settings: &Settings,
) -> Option<&'a GithubReleaseAsset> {
    let prefix = &settings.portal_release.asset_name_prefix;
    let suffix = &settings.portal_release.asset_name_suffix;
    let exact_name = format!("{prefix}{}{suffix}", release.tag_name);
    release
        .assets
        .iter()
        .find(|asset| asset.name == exact_name)
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|asset| asset.name.starts_with(prefix) && asset.name.ends_with(suffix))
        })
}

fn verify_asset_digest(expected: Option<&str>, actual: &str) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let expected = expected.trim();
    if expected.is_empty() {
        return Ok(());
    }
    if !expected
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("sha256:"))
    {
        return Err(format!(
            "Unsupported GitHub release asset digest format: {expected}"
        ));
    }
    if expected.eq_ignore_ascii_case(actual) {
        Ok(())
    } else {
        Err(format!(
            "Portal release digest mismatch. Expected {expected}, downloaded {actual}."
        ))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }
    output
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!(),
    }
}

fn safe_download_file_name(tag_name: &str, asset_name: &str) -> String {
    let raw = format!("{tag_name}-{asset_name}");
    let sanitized: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.to_lowercase().ends_with(".zip") {
        sanitized
    } else {
        format!("{sanitized}.zip")
    }
}

pub(crate) fn display_download_path(download: &PortalReleaseDownload) -> String {
    display_path(&download.zip_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::default_settings;

    #[test]
    fn parses_latest_release_and_prefers_exact_tag_asset() {
        let settings = default_settings();
        let body = r#"
        {
          "tag_name": "v1.2.3",
          "name": "Portal v1.2.3",
          "published_at": "2026-05-04T00:00:00Z",
          "body": "notes",
          "html_url": "https://github.com/owner/repo/releases/tag/v1.2.3",
          "assets": [
            {
              "name": "EduPortal_DiemSensei_old.zip",
              "size": 10,
              "url": "https://api.github.com/assets/1",
              "digest": null
            },
            {
              "name": "EduPortal_DiemSensei_v1.2.3.zip",
              "size": 20,
              "url": "https://api.github.com/assets/2",
              "digest": "sha256:abc"
            }
          ]
        }
        "#;

        let release = parse_latest_release(body, &settings).expect("release");
        assert_eq!(release.info.tag_name, "v1.2.3");
        assert_eq!(release.info.asset_name, "EduPortal_DiemSensei_v1.2.3.zip");
        assert_eq!(release.info.asset_size, 20);
        assert_eq!(release.api_url, "https://api.github.com/assets/2");
    }

    #[test]
    fn validates_sha256_digest_case_insensitively() {
        let actual = format!("sha256:{}", sha256_hex(b"portal"));
        let upper = actual.to_uppercase();
        verify_asset_digest(Some(&upper), &actual).expect("digest");
    }

    #[test]
    fn rejects_sha256_digest_mismatch() {
        let err = verify_asset_digest(
            Some("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .expect_err("mismatch");
        assert!(err.contains("digest mismatch"));
    }
}
