use std::path::{Path, PathBuf};
use std::str;
use anyhow::{Result};
use crate::{args};
use crate::http::CachedHttpClient;

pub struct Api {
    client: CachedHttpClient,
    base_url: String,
    platform: &'static str,
}

#[cfg(target_os = "windows")]
pub static PLATFORM: &str = "windowsx64";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub static PLATFORM: &str = "linuxx64";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub static PLATFORM: &str = "linuxarm64";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub static PLATFORM: &str = "darwinx64";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub static PLATFORM: &str = "darwinarm64";

impl Api {
    pub fn new(cache_dir: &Path, force: bool) -> Self {
        Self {
            client: CachedHttpClient::new(cache_dir, args::offline(), force),
            base_url: "https://api.sdkman.io/2".to_string(),
            platform: PLATFORM,
        }
    }

    pub fn get_text(&self, uri: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, uri);
        self.client.get_text(&url)
    }

    // pub fn get_api_version(&self) -> Result<String> {
    //     let base_url = &self.base_url;
    //     Ok(self.get_text(&format!("/broker/download/sdkman/version/stable"))?)
    // }

    pub fn get_tools(&self) -> Result<Vec<String>> {
        Ok(self.get_text("/candidates/all")?
            .split(",")
            .map(|v| v.to_string())
            .collect())
    }

    pub fn get_tool_versions(&self, tool: &str) -> Result<Vec<String>> {
        let platform = &self.platform;
        let versions = self.get_text(&format!("/candidates/{tool}/{platform}/versions/list?installed="))?;

        let versions = match tool {
            "java" => decode_java_versions(&versions),
            _ => decode_versions(&versions)
        };
        Ok(versions)
    }

    pub fn get_default_version(&self, tool: &str) -> Result<String> {
        self.get_text(&format!("/candidates/default/{tool}"))
    }

    pub fn get_cached_file(&self, tool: &str, version: &str) -> Result<PathBuf> {
        let platform = &self.platform;
        let url = format!("{}/broker/download/{tool}/{version}/{platform}", self.base_url);
        self.client.get_cached_file(&url)
    }
}

fn decode_versions(versions: &str) -> Vec<String> {
    let mut sepcount = 0;
    let mut mmuh: Vec<Vec<&str>> = versions
        .lines()
        .filter(|l| {
            if l.starts_with("===") {
                sepcount += 1;
                false
            } else {
                sepcount > 1 && sepcount < 3
            }
        })
        .map(|l| l.split(" ")
            .filter(|x| !x.trim().is_empty())
            .collect::<Vec<_>>())
        .filter(|v| !v.is_empty())
        .collect();

    let mut vaches = Vec::new();
    'zz: loop {
        for v in &mut mmuh {
            if v.is_empty() { break 'zz; }
            vaches.push(v.remove(0).to_string());
        }
    }
    vaches
}

fn decode_java_versions(versions: &str) -> Vec<String> {
    let mut dash_lines = 0;
    let mut eq_lines = 0;

    let mut mmuh: Vec<Vec<&str>> = versions
        .lines()
        .filter(|l| {
            if l.starts_with("---") {
                dash_lines += 1;
                false
            } else if l.starts_with("===") {
                eq_lines += 1;
                false
            } else  {
                dash_lines == 1 && eq_lines < 3
            }
        })
        .map(|l| l.split("|")
            .map(|x| x.trim())
            .collect::<Vec<_>>())
        .filter(|v| !v.is_empty())
        .collect();

    let mut vaches = Vec::new();
    for v in &mut mmuh {
        vaches.push(v[5].to_string());
    }
    vaches
}
