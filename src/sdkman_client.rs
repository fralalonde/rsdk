use std::path::{Path};
use std::str;
use anyhow::{Result};
use crate::sdkman_decode::{decode_java_versions, decode_versions};
use crate::cache::{CacheEntry};
use crate::http_client::{CachedHttpClient};

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
}

impl SdkManClient {
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            http_client: CachedHttpClient::new(cache_dir),
            base_url: "https://api.sdkman.io/2".to_string(),
            platform: PLATFORM,
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
        Ok(self.get_tools_text()?
            .split(",")
            .map(|v| v.to_string())
            .collect())
    }

    pub fn get_tool_versions_text(&self, tool: &str) -> Result<String> {
        let platform = &self.platform;
        self.get_text(&format!("/candidates/{tool}/{platform}/versions/list?installed="))
    }

    pub fn get_tool_versions(&self, tool: &str) -> Result<Vec<String>> {
        let versions = self.get_tool_versions_text(tool)?;

        let versions = match tool {
            "java" => decode_java_versions(&versions),
            _ => decode_versions(&versions)
        };
        Ok(versions)
    }

    pub fn get_default_version(&self, tool: &str) -> Result<String> {
        self.get_text(&format!("/candidates/default/{tool}"))
    }

    pub fn get_cached_file(&self, tool: &str, version: &str) -> Result<CacheEntry> {
        let platform = &self.platform;
        let url = format!("{}/broker/download/{tool}/{version}/{platform}", self.base_url);
        self.http_client.get_cached_file(&url)
    }

    #[allow(unused)]
    pub fn get_post_install(&self, tool: &str, version: &str) -> Result<String> {
        let platform = &self.platform;
        self.get_text(&format!("/hooks/post/{tool}/{version}/{platform}"))
    }
}

