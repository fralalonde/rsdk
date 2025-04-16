use color_eyre::{Result};

use std::fs::File;
use std::path::{Path, PathBuf};
use log::debug;
use serde::Serialize;
use serde_derive::Deserialize;

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
        self.file_path().exists() && self.metadata_path().exists()
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
        let metadata: Metadata = meta_path.exists().then_some(meta_path)
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
