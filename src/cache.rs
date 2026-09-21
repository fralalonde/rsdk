use color_eyre::Result;

use log::debug;
use serde::Serialize;
use serde_derive::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Files smaller than this are never trusted as cached archives: real JDK/tool
/// archives are megabytes, while API error pages (nginx 503/404 HTML cached by
/// pre-status-check clients) are a few hundred bytes. `rsdk flush` remains the
/// way to clear the cache manually.
pub const MIN_ARCHIVE_BYTES: u64 = 1024;

#[derive(Debug)]
pub struct CacheEntry {
    cache_dir: PathBuf,
    url_hash: String,
    pub metadata: Metadata,
}

impl CacheEntry {
    pub fn file_path(&self) -> PathBuf {
        self.cache_dir.join(&self.url_hash)
    }

    fn metadata_path(&self) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", self.url_hash))
    }

    pub fn is_valid(&self) -> bool {
        if !(self.file_path().exists() && self.metadata_path().exists()) {
            return false;
        }
        // Guard against error bodies cached by pre-status-check clients: a
        // "valid" entry below the archive size floor is treated as absent so
        // it gets re-downloaded instead of failing extraction.
        match std::fs::metadata(self.file_path()) {
            Ok(meta) => meta.len() >= MIN_ARCHIVE_BYTES,
            Err(_) => false,
        }
    }

    pub fn save(&self) -> Result<()> {
        debug!("saving metadata to {:?}", self.metadata_path());
        let file = File::create(self.metadata_path())?;
        Ok(serde_ini::to_writer(&file, &self.metadata)?)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub file_name: String,
}

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager with a specific cache directory.
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    pub fn get_cache_entry(&self, url: &str) -> CacheEntry {
        let url_hash = format!("{:x}", md5::compute(url));
        let cache_path = self.cache_dir.join(&url_hash);
        let meta_path = cache_path.with_extension("meta");
        let metadata: Metadata = meta_path
            .exists()
            .then_some(meta_path)
            .and_then(|path| File::open(path).ok())
            .and_then(|file| serde_ini::from_read(&file).ok())
            .unwrap_or_default();

        CacheEntry {
            cache_dir: self.cache_dir.clone(),
            url_hash,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cache_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("rsdk-cache-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn undersized_entries_are_invalid() {
        let dir = temp_cache_dir("undersized");
        let mgr = CacheManager::new(&dir);
        let entry = mgr.get_cache_entry("https://example.test/tool.tgz");
        std::fs::write(entry.file_path(), b"<html>503</html>").unwrap();
        entry.save().unwrap();
        // 15-byte error page: rejected despite existing files, so the client
        // re-downloads instead of failing extraction later.
        assert!(!entry.is_valid());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_files_are_invalid() {
        let dir = temp_cache_dir("missing");
        let entry = CacheManager::new(&dir).get_cache_entry("https://example.test/none.tgz");
        assert!(!entry.is_valid());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
