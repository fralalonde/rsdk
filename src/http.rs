use anyhow::{Context, Result};
use log::debug;
use reqwest::blocking::Client;
use reqwest::header;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path};
use std::time::Duration;
use crate::args;
use crate::cache::{CacheManager,  CacheEntry};
use crate::http_utils::{extract_filename_from_disposition, initialize_progress_bar, read_proxy_from_env};

pub struct CachedHttpClient {
    cache: CacheManager,
    client: Client,
}

impl CachedHttpClient {
    pub fn new(cache_dir: &Path) -> Self {
        let mut client = Client::builder().timeout(Duration::from_secs(30));

        if let Some(proxy) = read_proxy_from_env() {
            client = client.proxy(proxy);
        }

        if args::insecure() {
            client = client.danger_accept_invalid_certs(true)
        }

        let client = client.build().expect("Failed to build reqwest client");
        Self {
            cache: CacheManager::new(cache_dir),
            client,
        }
    }

    pub fn get_text(&self, url: &str) -> Result<String> {
        debug!("getting text for {url}");
        let response = self.client.get(url).send()?;
        let content = response.text()?;
        Ok(content)
    }

    pub fn get_cached_file(&self, url: &str) -> Result<CacheEntry> {
        debug!("Getting file for {url}");
        let mut entry = self.cache.get_cache_entry(url);

       if !entry.is_valid() {
            debug!("Downloading file");
            let name = self.download_to_file(url, &entry.file_path())?;
            entry.metadata.file_name = name;
            entry.save()?;
        } else {
            debug!("File found in cache");
        };

        Ok(entry)
    }

    fn download_to_file(&self, url: &str, file_path: &Path) -> Result<String> {
        let head_response = self.client.head(url).send().context("Failed to send HEAD request")?;
        let headers = head_response.headers().clone();
        let total_size = headers.get(header::CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok()?.parse::<u64>().ok())
            .context("Failed to get content length")?;

        let file_name = headers.get(header::CONTENT_DISPOSITION)
            .and_then(|value| extract_filename_from_disposition(value.to_str().ok()?))
            .unwrap_or("file")
            .to_string();

        let pb = initialize_progress_bar(total_size, &file_name);

        let mut response = self.client.get(url).send().context("Failed to send GET request")?;
        debug!("HTTP response status {}", response.status());
        if response.status() == 304 {
            return Ok(file_name);
        }

        let mut cache_file = File::create(file_path).context("Failed to create cache file")?;
        let mut buffer = [0; 8192];
        while let Ok(bytes_read) = response.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }
            cache_file.write_all(&buffer[..bytes_read])?;
            pb.inc(bytes_read as u64);
        }
        pb.finish_with_message("Download completed");

        Ok(file_name)
    }
}
