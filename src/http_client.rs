use color_eyre::Result;
use eyre::Context;
use log::debug;
use reqwest::blocking::Client;
use reqwest::header;

use crate::args;
use crate::cache::{CacheEntry, CacheManager};
use crate::http_utils::{
    extract_filename_from_disposition, initialize_progress_bar, read_proxy_from_env,
};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Reject non-success HTTP responses *before* anything is written to the
/// download cache, so error bodies (e.g. SDKMAN broker 503/404 pages) can
/// never poison the cache.
fn check_status(status: reqwest::StatusCode, url: &str) -> Result<()> {
    if !status.is_success() {
        eyre::bail!("HTTP {status} for {url}");
    }
    Ok(())
}

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
        // Reject error pages here too: otherwise a 503 HTML body reaches the
        // JSON/CSV decoders and surfaces as a confusing parse error.
        check_status(response.status(), url)?;
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

    /// Monitored variant: reports `(bytes_downloaded, total_size)` via the
    /// callback after each chunk, and aborts (cleaning up the partial file)
    /// when `cancel` is set. Used by the TUI for its progress modal.
    pub fn get_cached_file_monitored(
        &self,
        url: &str,
        on_progress: &mut dyn FnMut(u64, u64),
        cancel: &AtomicBool,
    ) -> Result<CacheEntry> {
        debug!("Getting (monitored) file for {url}");
        let mut entry = self.cache.get_cache_entry(url);

        if !entry.is_valid() {
            let name =
                self.download_to_file_monitored(url, &entry.file_path(), on_progress, cancel)?;
            entry.metadata.file_name = name;
            entry.save()?;
        } else {
            debug!("File found in cache");
            let len = std::fs::metadata(entry.file_path())?.len();
            on_progress(len, len);
        }

        Ok(entry)
    }

    /// Download `url` directly, bypassing the broker. Returns the entry and
    /// whether the file had to be downloaded (`true`) or was cached (`false`).
    pub fn get_cached_file_monitored_url(
        &self,
        url: &str,
        on_progress: &mut dyn FnMut(u64, u64),
        cancel: &AtomicBool,
    ) -> Result<CacheEntry> {
        self.get_cached_file_monitored(url, on_progress, cancel)
    }

    fn download_to_file(&self, url: &str, file_path: &Path) -> Result<String> {
        // Read length/filename from the GET response instead of a separate
        // HEAD: the SDKMAN broker answers HEAD with 503 while GET redirects
        // fine, and some vendor CDNs omit Content-Length on HEAD.
        let mut response = self
            .client
            .get(url)
            .send()
            .context("Failed to send GET request")?;
        debug!("HTTP response status {}", response.status());
        check_status(response.status(), url)?;

        let total_size = content_length(response.headers()).unwrap_or(0);
        let file_name = disposition_file_name(response.headers()).unwrap_or_default();
        let pb = initialize_progress_bar(total_size, &file_name);

        let mut cache_file = File::create(file_path).context("Failed to create cache file")?;
        let mut buffer = [0; 8192];
        loop {
            match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    cache_file.write_all(&buffer[..bytes_read])?;
                    pb.inc(bytes_read as u64);
                }
                // Truncated transfer: drop the partial body so a short archive
                // never ends up cached and extracted.
                Err(e) => {
                    drop(cache_file);
                    let _ = std::fs::remove_file(file_path);
                    return Err(e).context(format!("Failed to read body for {url}"));
                }
            }
        }
        pb.finish_with_message("Download completed");

        Ok(file_name)
    }

    fn download_to_file_monitored(
        &self,
        url: &str,
        file_path: &Path,
        on_progress: &mut dyn FnMut(u64, u64),
        cancel: &AtomicBool,
    ) -> Result<String> {
        let mut response = self.client.get(url).send().context("GET")?;
        debug!("HTTP response status {}", response.status());
        check_status(response.status(), url)?;

        // Progress can start as soon as the response headers land, so the
        // total is known before the first chunk without a HEAD round-trip.
        let total_size = content_length(response.headers()).unwrap_or(0);
        let file_name = disposition_file_name(response.headers()).unwrap_or_default();
        on_progress(0, total_size);

        let mut cache_file = File::create(file_path).context("create cache file")?;
        let mut buffer = [0u8; 8192];
        let mut downloaded: u64 = 0;
        loop {
            if cancel.load(Ordering::Relaxed) {
                drop(cache_file);
                let _ = std::fs::remove_file(file_path);
                eyre::bail!("download cancelled");
            }
            match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    cache_file.write_all(&buffer[..bytes_read])?;
                    downloaded += bytes_read as u64;
                    on_progress(
                        if total_size > 0 {
                            downloaded.min(total_size)
                        } else {
                            downloaded
                        },
                        total_size,
                    );
                }
                Err(e) => {
                    drop(cache_file);
                    let _ = std::fs::remove_file(file_path);
                    return Err(e).context(format!("Failed to read body for {url}"));
                }
            }
        }

        Ok(file_name)
    }
}

/// `Content-Length` as a byte count, when the server sends one.
fn content_length(headers: &header::HeaderMap) -> Option<u64> {
    headers
        .get(header::CONTENT_LENGTH)
        .and_then(|len| len.to_str().ok()?.parse::<u64>().ok())
}

/// Archive file name advertised by `Content-Disposition`.
fn disposition_file_name(headers: &header::HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| extract_filename_from_disposition(value.to_str().ok()?))
        .map(|name| name.to_string())
}
