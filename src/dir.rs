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
        create_dir_all(rsdk.cache())?;
        create_dir_all(rsdk.temp())?;
        fs::remove_dir_all(rsdk.temp())?;
        Ok(rsdk)
    }

    pub fn default_symlink_path(&self, candidate: &str) -> PathBuf {
        self.candidate_dir(candidate).join("default")
    }

    pub fn current_default(&self, candidate: &str) -> anyhow::Result<Option<CandidateVersion>> {
        let def = self.default_symlink_path(candidate);
        if !def.exists() {
            return Ok(None);
        }
        let linked = fs::read_link(def)?;
        Ok(linked.as_path().iter().last()
            .and_then(|version| version.to_str().map(|version| version.to_owned()))
            .map(|version| CandidateVersion::new(self, candidate, &version)))
    }

    pub fn all_defaults(&self) -> anyhow::Result<Vec<CandidateVersion>> {
        let mut defaults = Vec::new();
        let candidates_dir = self.candidates();

        // Iterate over directories in the `candidates` path
        for entry in fs::read_dir(candidates_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(candidate) = entry.file_name().to_str() {
                    // Check if there is a default version for this candidate
                    if let Some(default_version) = self.current_default(candidate)? {
                        defaults.push(default_version);
                    }
                }
            }
        }
        Ok(defaults)
    }

    pub fn all_versions(&self) -> anyhow::Result<Vec<CandidateVersion>> {
        let mut versions = Vec::new();
        let candidates_dir = self.candidates();

        // Iterate over directories in the `candidates` path
        for entry in fs::read_dir(candidates_dir)? {
            let c_entry = entry?;
            if c_entry.file_type()?.is_dir() {
                if let Some(candidate) = c_entry.file_name().to_str() {
                    for v_entry in fs::read_dir(self.candidate_dir(candidate))? {
                        let vv = v_entry?;
                        if vv.file_type()?.is_dir() {
                            if let Some(version) = vv.file_name().to_str() {
                                let cv = CandidateVersion::new(&self, candidate, version);
                                versions.push(cv);
                            }
                        }
                    }
                }
            }
        }
        Ok(versions)
    }

    pub fn candidates(&self) -> PathBuf {
        self.root.join("candidates")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
    pub fn temp(&self) -> PathBuf {
        self.root.join("temp")
    }

    pub fn candidate_dir(&self, candidate: &str) -> PathBuf {
        self.candidates().join(candidate)
    }
}