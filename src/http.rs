use std::env;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::header;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::args;

pub struct CachedHttpClient {
    cache_dir: PathBuf,
    offline: bool,
    force: bool,
    client: Client,
}

impl CachedHttpClient {
    pub fn new(cache_dir: &Path, offline: bool, force: bool) -> Self {
        let mut client = Client::builder()
            .timeout(Duration::from_secs(30));

        if let Some(proxy) = Self::read_proxy_from_env() {
            client = client.proxy(proxy);
        }

        if args::insecure() {
            client = client.danger_accept_invalid_certs(true)
        }

        let client = client
            .build()
            .expect("Failed to build reqwest client");
        Self {
            cache_dir: cache_dir.to_path_buf(),
            force,
            offline,
            client,
        }
    }

    pub fn get_cached_file(&self, url: &str) -> Result<PathBuf> {
        let (cache_path, meta_path) = self.get_cache_paths(url);

        // If offline mode is enabled
        if self.offline {
            if cache_path.exists() {
                return Ok(cache_path);
            } else {
                return Err(anyhow::anyhow!("Offline mode is enabled, and no cached data is available for this URL."));
            }
        }

        // Force re-download if `use_force()` is true
        if self.force || !self.is_cache_valid(&cache_path, &meta_path)? {
            self.download_to_file(url, &cache_path, &cache_path)?;
            self.update_metadata(&meta_path, url)?;
        }
        Ok(cache_path)
    }

    pub fn get_text(&self, url: &str) -> Result<String> {
        let (cache_path, meta_path) = self.get_cache_paths(url);

        if self.force || !self.is_cache_valid(&cache_path, &meta_path)? {
            let content = self.download_text(url, &cache_path)?;
            self.update_metadata(&meta_path, url)?;
            return Ok(content);
        }

        // Otherwise, use the cached content if valid
        if let Some(content) = self.read_cache_if_valid(&cache_path, &meta_path)? {
            return Ok(content);
        }

        Err(anyhow::anyhow!("No valid cached data available."))
    }

    fn is_cache_valid(&self, cache_path: &Path, meta_path: &Path) -> Result<bool> {
        if cache_path.exists() && meta_path.exists() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_cache_paths(&self, url: &str) -> (PathBuf, PathBuf) {
        let hash = format!("{:x}", md5::compute(url));
        let cache_path = self.cache_dir.join(&hash);
        let meta_path = cache_path.with_extension("meta");
        (cache_path, meta_path)
    }

    fn download_text(&self, url: &str, cache_path: &Path) -> Result<String> {
        let response = self.client.get(url).send().context("Failed to send GET request")?;
        if response.status() == 304 {
            return self.read_from_cache(cache_path);
        }

        let content = response.text().context("Failed to read response as text")?;
        File::create(cache_path)?.write_all(content.as_bytes())?;
        Ok(content)
    }

    fn download_to_file(&self, url: &str, cache_path: &Path, output_path: &Path) -> Result<()> {
        let head_response = self.client.head(url).send().context("Failed to send HEAD request")?;
        let total_size = head_response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok()?.parse::<u64>().ok())
            .context("Failed to get content length")?;

        let file_name = head_response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|header| Self::extract_filename_from_disposition(header.to_str().ok()?))
            .unwrap_or("file");

        let pb = Self::initialize_progress_bar(total_size, file_name);

        let mut request = self.client.get(url);
        let (_, meta_path) = self.get_cache_paths(url);
        if let Ok((etag, last_modified)) = self.load_metadata(&meta_path) {
            if let Some(etag) = etag {
                request = request.header(header::IF_NONE_MATCH, etag);
            }
            if let Some(last_modified) = last_modified {
                request = request.header(header::IF_MODIFIED_SINCE, last_modified);
            }
        }

        let mut response = request.send().context("Failed to send GET request")?;
        if response.status() == 304 {
            fs::copy(cache_path, output_path).context("Failed to copy cached data to output file")?;
            return Ok(());
        }

        let mut cache_file = File::create(cache_path).context("Failed to create cache file")?;
        let mut output_file = File::create(output_path).context("Failed to create output file")?;

        // Read the response in chunks and write to both cache and output files
        let mut buffer = [0; 8192];
        while let Ok(bytes_read) = response.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }
            cache_file.write_all(&buffer[..bytes_read])?;
            output_file.write_all(&buffer[..bytes_read])?;
            pb.inc(bytes_read as u64);
        }
        pb.finish_with_message("Download completed");

        self.update_metadata(&meta_path, url)?;

        Ok(())
    }

    fn read_proxy_from_env() -> Option<reqwest::Proxy> {
        if let Ok(http_proxy) = env::var("http_proxy") {
            return reqwest::Proxy::http(&http_proxy).ok();
        }
        if let Ok(https_proxy) = env::var("https_proxy") {
            return reqwest::Proxy::https(&https_proxy).ok();
        }
        None
    }

    fn extract_filename_from_disposition(content_disposition: &str) -> Option<&str> {
        content_disposition
            .split(';')
            .find_map(|part| part.trim().strip_prefix("filename="))
            .map(|filename| filename.trim_matches('"'))
    }

    fn initialize_progress_bar(total_size: u64, file_name: &str) -> ProgressBar {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.green}} {file_name} {{bar:40.cyan/blue}} {{bytes}}/{{total_bytes}} ({{eta}})"
                ))
                .expect("Failed to set progress bar template")
                .progress_chars("#>-"),
        );
        pb
    }

    fn read_cache_if_valid(&self, cache_path: &Path, meta_path: &Path) -> Result<Option<String>> {
        if cache_path.exists() && meta_path.exists() {
            let mut contents = String::new();
            File::open(cache_path)?.read_to_string(&mut contents)?;
            Ok(Some(contents))
        } else {
            Ok(None)
        }
    }

    fn read_from_cache(&self, cache_path: &Path) -> Result<String> {
        let mut contents = String::new();
        File::open(cache_path)?.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn save_metadata(&self, meta_path: &Path, etag: &Option<String>, last_modified: &Option<String>) -> io::Result<()> {
        let mut file = File::create(meta_path)?;
        writeln!(file, "{}", etag.as_deref().unwrap_or(""))?;
        writeln!(file, "{}", last_modified.as_deref().unwrap_or(""))?;
        Ok(())
    }

    fn load_metadata(&self, meta_path: &Path) -> Result<(Option<String>, Option<String>)> {
        let file = File::open(meta_path)?;
        let mut lines = BufReader::new(file).lines();

        let etag = lines.next().transpose()?.map(|line| line.trim().to_string());
        let last_modified = lines.next().transpose()?.map(|line| line.trim().to_string());

        Ok((etag, last_modified))
    }

    fn update_metadata(&self, meta_path: &Path, url: &str) -> Result<()> {
        let response = self.client.head(url).send().context("Failed to send HEAD request")?;
        let etag = response.headers().get(header::ETAG).map(|s| s.to_str().unwrap().to_string());
        let last_modified = response.headers().get(header::LAST_MODIFIED).map(|s| s.to_str().unwrap().to_string());
        self.save_metadata(meta_path, &etag, &last_modified)?;
        Ok(())
    }
}
