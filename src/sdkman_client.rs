use crate::cache::CacheEntry;
use crate::http_client::CachedHttpClient;
use crate::sdkman_decode::{decode_java_versions, decode_versions};
use crate::state_client::{display_names, StateClient};
use color_eyre::Result;
use std::path::Path;
use std::str;
use std::sync::atomic::AtomicBool;

#[cfg(target_os = "windows")]
pub static PLATFORM: &str = "windowsx64";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub static PLATFORM: &str = "linuxx64";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub static PLATFORM: &str = "linuxarm64";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub static PLATFORM: &str = "darwinx64";

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
pub static PLATFORM: &str = "darwinarm64";

pub struct SdkManClient {
    http_client: CachedHttpClient,
    base_url: String,
    platform: &'static str,
    state: StateClient,
}

impl SdkManClient {
    pub fn new(cache_dir: &Path) -> Self {
        // Allow overriding the API base (used by the VHS demo and offline
        // tests); defaults to the real SDKMAN API.
        let base_url = std::env::var("RSDK_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.sdkman.io/2".to_string());
        Self {
            http_client: CachedHttpClient::new(cache_dir),
            base_url,
            platform: PLATFORM,
            state: StateClient::new(cache_dir),
        }
    }

    pub fn get_text(&self, uri: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, uri);
        self.http_client.get_text(&url)
    }

    #[allow(unused)]
    pub fn get_api_version(&self) -> Result<String> {
        let base_url = &self.base_url;
        self.get_text("/broker/download/sdkman/version/stable")
    }

    pub fn get_tools_list_text(&self) -> Result<String> {
        self.get_text("/candidates/list")
    }

    pub fn get_tools_text(&self) -> Result<String> {
        self.get_text("/candidates/all")
    }

    pub fn get_tools(&self) -> Result<Vec<String>> {
        // State API is primary; legacy CSV is the fallback.
        if let Ok(candidates) = self.state.get_candidates() {
            return Ok(candidates.into_iter().map(|c| c.candidate).collect());
        }
        Ok(self
            .get_tools_text()?
            .split(",")
            .map(|v| v.to_string())
            .collect())
    }

    /// Per-tool display metadata (name, description lines, website). Served
    /// natively by the State API; `None` when it's unavailable and callers
    /// should fall back to the legacy text scrape.
    pub fn get_candidate_info(
        &self,
    ) -> Option<std::collections::HashMap<String, crate::state_client::Candidate>> {
        self.state
            .get_candidates()
            .ok()
            .map(|list| list.into_iter().map(|c| (c.candidate.clone(), c)).collect())
    }

    pub fn get_tool_versions_text(&self, tool: &str) -> Result<String> {
        let platform = &self.platform;
        self.get_text(&format!(
            "/candidates/{tool}/{platform}/versions/list?installed="
        ))
    }

    pub fn get_tool_versions(&self, tool: &str) -> Result<Vec<String>> {
        // State API is primary: JSON records collapsed into legacy-style
        // display identifiers (version + vendor suffix). Legacy text-table
        // decoding stays as fallback.
        if let Ok(records) = self.state.get_versions(tool) {
            return Ok(display_names(records, self.platform));
        }
        let versions = self.get_tool_versions_text(tool)?;

        let versions = match tool {
            "java" => decode_java_versions(&versions),
            _ => decode_versions(&versions),
        };
        Ok(versions)
    }

    pub fn get_default_version(&self, tool: &str) -> Result<String> {
        // The State API only carries a `default` for single-vendor candidates;
        // multi-vendor ones like java have none, so fall back to the legacy
        // default endpoint for those (and whenever State has no answer).
        if let Ok(candidates) = self.state.get_candidates() {
            if let Some(c) = candidates.iter().find(|c| c.candidate == tool) {
                if let Some(default) = c.default.clone() {
                    return Ok(default);
                }
            }
        }
        self.get_text(&format!("/candidates/default/{tool}"))
    }

    pub fn get_cached_file(&self, tool: &str, version: &str) -> Result<CacheEntry> {
        let platform = &self.platform;
        let url = format!(
            "{}/broker/download/{tool}/{version}/{platform}",
            self.base_url
        );
        self.http_client.get_cached_file(&url)
    }

    /// Download `url` directly (State API vendor URL), with progress/cancel.
    /// Same caching semantics as the broker path.
    pub fn get_cached_file_monitored_url(
        &self,
        url: &str,
        on_progress: &mut dyn FnMut(u64, u64),
        cancel: &AtomicBool,
    ) -> Result<CacheEntry> {
        self.http_client
            .get_cached_file_monitored(url, on_progress, cancel)
    }

    /// Broker download for `tool/version` — legacy fallback when the State
    /// API has no usable record.
    pub fn get_cached_file_monitored(
        &self,
        tool: &str,
        version: &str,
        on_progress: &mut dyn FnMut(u64, u64),
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<CacheEntry> {
        let platform = &self.platform;
        let url = format!(
            "{}/broker/download/{tool}/{version}/{platform}",
            self.base_url
        );
        self.http_client
            .get_cached_file_monitored(&url, on_progress, cancel)
    }

    #[allow(unused)]
    pub fn get_post_install(&self, tool: &str, version: &str) -> Result<String> {
        let platform = &self.platform;
        self.get_text(&format!("/hooks/post/{tool}/{version}/{platform}"))
    }
}
