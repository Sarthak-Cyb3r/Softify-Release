//! GitHub Releases auto-update checker and SemVer comparison engine.

use reqwest::Client;
use serde_json::Value;
use softify_models::UpdateInfo;

const DEFAULT_REPO: &str = "Sarthak-Cyb3r/Softify-Release";
const UPDATER_USER_AGENT: &str = "Softify-Updater/1.0 (Linux; Android)";

/// GitHub Releases update checker.
#[derive(Clone, Debug)]
pub struct UpdateChecker {
    client: Client,
    repo: String,
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateChecker {
    pub fn new() -> Self {
        Self::with_repo(DEFAULT_REPO)
    }

    pub fn with_repo(repo: &str) -> Self {
        let client = Client::builder()
            .user_agent(UPDATER_USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            repo: repo.to_string(),
        }
    }

    /// Queries GitHub API for the latest release and compares against `current_version`.
    pub async fn check_for_updates(&self, current_version: &str) -> Result<UpdateInfo, String> {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.repo
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", UPDATER_USER_AGENT)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("GitHub API returned status {status}: {body}"));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read GitHub response text: {e}"))?;

        Self::parse_release_response(&body, current_version)
    }

    /// Parses GitHub release JSON string and computes `UpdateInfo`.
    pub fn parse_release_response(json_str: &str, current_version: &str) -> Result<UpdateInfo, String> {
        let root: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid GitHub release JSON: {e}"))?;

        let tag_name = root
            .get("tag_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'tag_name' in GitHub release payload".to_string())?;

        let latest_version = tag_name.trim_start_matches('v').to_string();
        let has_update = Self::is_newer_version(&latest_version, current_version);

        let release_notes = root
            .get("body")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // Extract download URL: prioritize .apk, .AppImage, .tar.gz, .zip, or html_url
        let mut download_url = None;

        if let Some(assets) = root.get("assets").and_then(|a| a.as_array()) {
            // First priority: Android .apk
            for asset in assets {
                let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if name.ends_with(".apk") {
                    if let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                        download_url = Some(url.to_string());
                        break;
                    }
                }
            }

            // Second priority: Linux / Desktop AppImage or archive
            if download_url.is_none() {
                for asset in assets {
                    let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    if name.ends_with(".AppImage")
                        || name.ends_with(".tar.gz")
                        || name.ends_with(".zip")
                        || name.ends_with(".deb")
                    {
                        if let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                            download_url = Some(url.to_string());
                            break;
                        }
                    }
                }
            }
        }

        // Fallback: html_url (release page)
        if download_url.is_none() {
            download_url = root
                .get("html_url")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());
        }

        Ok(UpdateInfo {
            latest_version,
            current_version: current_version.trim_start_matches('v').to_string(),
            has_update,
            release_notes,
            download_url,
        })
    }

    /// Evaluates if `latest` version is strictly newer than `current` version using SemVer rules.
    pub fn is_newer_version(latest: &str, current: &str) -> bool {
        match (Self::parse_semver(latest), Self::parse_semver(current)) {
            (Some(l), Some(c)) => {
                if l.0 != c.0 {
                    return l.0 > c.0;
                }
                if l.1 != c.1 {
                    return l.1 > c.1;
                }
                l.2 > c.2
            }
            _ => false,
        }
    }

    /// Parses string into `(major, minor, patch)` tuple.
    pub fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
        let cleaned = v.trim().trim_start_matches('v');
        // Handle prerelease tags like "1.2.3-beta.1"
        let core_version = cleaned.split('-').next().unwrap_or(cleaned);
        let parts: Vec<&str> = core_version.split('.').collect();

        match parts.len() {
            1 => {
                let major = parts[0].parse().ok()?;
                Some((major, 0, 0))
            }
            2 => {
                let major = parts[0].parse().ok()?;
                let minor = parts[1].parse().ok()?;
                Some((major, minor, 0))
            }
            3.. => {
                let major = parts[0].parse().ok()?;
                let minor = parts[1].parse().ok()?;
                let patch = parts[2].parse().ok()?;
                Some((major, minor, patch))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_comparisons() {
        // Newer patch
        assert!(UpdateChecker::is_newer_version("1.0.1", "1.0.0"));
        // Newer minor
        assert!(UpdateChecker::is_newer_version("1.1.0", "1.0.9"));
        // Newer major
        assert!(UpdateChecker::is_newer_version("2.0.0", "1.9.9"));
        // With 'v' prefix
        assert!(UpdateChecker::is_newer_version("v1.2.0", "1.1.0"));
        assert!(UpdateChecker::is_newer_version("1.2.0", "v1.1.0"));

        // Equal versions
        assert!(!UpdateChecker::is_newer_version("1.0.0", "1.0.0"));
        assert!(!UpdateChecker::is_newer_version("v1.0.0", "1.0.0"));

        // Older versions
        assert!(!UpdateChecker::is_newer_version("0.9.0", "1.0.0"));
        assert!(!UpdateChecker::is_newer_version("1.0.0", "1.0.1"));
        assert!(!UpdateChecker::is_newer_version("1.0.0", "1.1.0"));
    }

    #[test]
    fn test_parse_release_with_apk_asset() {
        let release = serde_json::json!({
            "tag_name": "v1.2.0",
            "name": "Softify v1.2.0 Studio Release",
            "body": "## Release Notes\n- 320 Kbps studio master\n- Algorithmic mixes",
            "html_url": "https://github.com/Sarthak-Cyb3r/Softify-Release/releases/tag/v1.2.0",
            "assets": [
                {
                    "name": "softify-v1.2.0-universal.apk",
                    "browser_download_url": "https://github.com/Sarthak-Cyb3r/Softify-Release/releases/download/v1.2.0/softify.apk"
                }
            ]
        });
        let json = release.to_string();

        let info = UpdateChecker::parse_release_response(&json, "1.1.0").unwrap();
        assert_eq!(info.latest_version, "1.2.0");
        assert_eq!(info.current_version, "1.1.0");
        assert!(info.has_update);
        assert_eq!(
            info.download_url.as_deref(),
            Some("https://github.com/Sarthak-Cyb3r/Softify-Release/releases/download/v1.2.0/softify.apk")
        );
        assert!(info.release_notes.as_deref().unwrap().contains("Release Notes"));
    }

    #[test]
    fn test_parse_release_fallback_to_html_url() {
        let release = serde_json::json!({
            "tag_name": "v1.0.0",
            "body": "Initial release",
            "html_url": "https://github.com/Sarthak-Cyb3r/Softify-Release/releases/tag/v1.0.0",
            "assets": []
        });
        let json = release.to_string();

        let info = UpdateChecker::parse_release_response(&json, "1.0.0").unwrap();
        assert!(!info.has_update);
        assert_eq!(
            info.download_url.as_deref(),
            Some("https://github.com/Sarthak-Cyb3r/Softify-Release/releases/tag/v1.0.0")
        );
    }
}
