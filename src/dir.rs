use fs::create_dir_all;
use std::{fs, io};
use std::path::{PathBuf};
use directories::UserDirs;

use crate::version::CandidateVersion;

#[derive(Clone)]
pub struct RsdkDir {
    pub root: PathBuf,
}

impl RsdkDir {
    pub fn new() -> io::Result<RsdkDir> {
        let user_dirs = UserDirs::new().expect("Failed to get user directories");
        let home_dir = user_dirs.home_dir();

        let rsdk_dir = home_dir.join(".rsdk");

        let rsdk = RsdkDir { root: rsdk_dir.to_path_buf() };
        create_dir_all(rsdk.candidates())?;
        Ok(rsdk)
    }

    pub fn default_symlink_path(&self, candidate: &str) -> PathBuf {
        self.candidate_path(candidate).join("default")
    }

    pub fn current_default(&self, candidate: &str) -> anyhow::Result<Option<CandidateVersion>> {
        let def = self.default_symlink_path(candidate);
        if !def.exists() {
            return Ok(None);
        }
        let linked = fs::read_link(def)?;
        let version = linked.as_path().iter().last().unwrap().to_str().unwrap().to_string();
        // FIXME no unwrap, proper err handling
        Ok(Some(CandidateVersion::new(&self, candidate, &version)))
    }

    pub fn candidates(&self) -> PathBuf {
        self.root.join("candidates")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn candidate_path(&self, candidate: &str) -> PathBuf {
        self.candidates().join(candidate)
    }
}