
use std::path::{Path};
use anyhow::{Result};
use reqwest::{Client, IntoUrl};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use crate::download::download_with_progress;

pub struct Api {
    client: ClientWithMiddleware,
    base_url: String,
    platform: String,
}

impl Api {
    pub fn new(cache_dir: &Path) -> Api {
        let cache_manager = CACacheManager {
            path: cache_dir.to_path_buf(),
        };

        let cache = HttpCache {
            mode: CacheMode::Default,
            manager: cache_manager,
            options: HttpCacheOptions::default(),
        };

        let caching_client = ClientBuilder::new(Client::new())
            .with(Cache(cache))
            .build();

        Api {
            client: caching_client,
            base_url: "https://api.sdkman.io/2".to_string(),
            platform: "cygwin".to_string(),
        }
    }

    async fn get_text(&self, uri: impl IntoUrl) -> Result<String> {
        let response = self.client.get(uri).send().await?;
        Ok(response.text().await?)
    }

    pub async fn get_api_version(&self) -> Result<String> {
        let base_url = &self.base_url;
        Ok(self.get_text(&format!("{base_url}/broker/download/sdkman/version/stable")).await?)
    }

    pub async fn get_candidates(&self) -> Result<Vec<String>> {
        let base_url = &self.base_url;
        Ok(self.get_text(&format!("{base_url}/candidates/all")).await?
            .split(",")
            .map(|v| v.to_string())
            .collect())
    }

    pub async fn get_candidate_versions(&self, candidate: &str) -> Result<Vec<String>> {
        let mut sepcount = 0;
        let base_url = &self.base_url;
        let platform = &self.platform;
        let versions = self.get_text(&format!("{base_url}/candidates/{candidate}/{platform}/versions/list")).await?;
        let mut mmuh: Vec<Vec<&str>> = versions
            .lines()
            .filter(|l| {
                if l.starts_with("===") {
                    sepcount += 1;
                    false
                } else if sepcount > 1 && sepcount < 3 {
                    true
                } else {
                    false
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
        Ok(vaches)
    }

    pub async fn get_default_version(&self, candidate: &str) -> Result<String> {
        let base_url = &self.base_url;
        Ok(self.get_text(&format!("{base_url}/candidates/default/{candidate}")).await?)
    }

    // pub async fn get_file(&self, candidate: &str, version: &str) -> Result<Response> {
    //     let base_url = &self.base_url;
    //     let platform = &self.platform;
    //     let url = format!("{base_url}/broker/download/{candidate}/{version}/{platform}");
    //     Ok(self.client.get(url).send().await?)
    // }

    pub async fn get_file2(&self, candidate: &str, version: &str, file: &Path) -> Result<()> {
        let base_url = &self.base_url;
        let platform = &self.platform;
        let url = format!("{base_url}/broker/download/{candidate}/{version}/{platform}");
        Ok(download_with_progress(&url, file, &self.client).await?)
    }
}
