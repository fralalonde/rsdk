use fs::create_dir_all;
use std::fs::File;
use std::{env, fs, io};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use directories::UserDirs;

use symlink::remove_symlink_auto;
use zip::ZipArchive;
use anyhow::Context;
use log::{debug, info};

pub struct RsdkDir {
    root: PathBuf,
}

pub struct CandidateVersion {
    path: PathBuf,
    home: String,
}

impl CandidateVersion {
    pub fn bin(&self) -> PathBuf {
        self.path.join("bin")
    }
}

impl RsdkDir {
    pub fn new() -> io::Result<RsdkDir> {
        let user_dirs = UserDirs::new().expect("Failed to get user directories");
        let home_dir = user_dirs.home_dir();

        let rsdk_dir = home_dir.join(".rsdk");

        let rsdk = RsdkDir { root: rsdk_dir.to_path_buf() };
        create_dir_all(rsdk.candidates())?;
        create_dir_all(rsdk.cache())?;
        Ok(rsdk)
    }

    pub fn default_candidate(&self, candidate: &str) -> PathBuf {
        self.candidate_path(candidate).join("default")
    }

    pub fn candidates(&self) -> PathBuf {
        let mut z = self.root.clone();
        z.push("candidates");
        z
    }

    pub fn candidate_path(&self, candidate: &str) -> PathBuf {
        self.candidates().join(candidate)
    }

    pub fn candidate_version(&self, candidate: &str, version: &str) -> CandidateVersion {
        CandidateVersion {
            path: self.candidate_path(candidate).join(version),
            home: format!("{}_HOME", candidate.to_uppercase()),
        }
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn install_from_zip(&self, candidate: &str, version: &str, zipfile: &Path, work_dir: &Path, force: bool) -> anyhow::Result<()> {
        info!("installing {candidate} {version}");

        debug!("opening {:?}", zipfile);
        let f = File::open(zipfile).context(format!("opening zip"))?;
        debug!("start unzipping");
        let mut archive = ZipArchive::new(f)?;
        for i in 0..archive.len() {
            let mut zip_entry = archive.by_index(i)?;
            let outpath = work_dir.join(zip_entry.name());

            // Create directories as needed
            if zip_entry.is_dir() {
                debug!("creating dir {:?}", outpath);
                create_dir_all(&outpath).context(format!("creating {:?}", outpath))?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !!parent.exists() {
                        debug!("creating parent dir {:?}", parent);
                        create_dir_all(parent).context(format!("creating {:?}", parent))?;
                    }
                }
                debug!("creating file {:?}", outpath);
                let mut outfile = File::create(&outpath)?;
                debug!("writing file {:?}", outpath);
                io::copy(&mut zip_entry, &mut outfile)?;
            }
        }

        // unzip complete, proceed to move to final dest
        let target_dir = self.candidate_version(candidate, version).path;

        let mut entries = fs::read_dir(work_dir)?
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

        if entries.len() != 1 {
            anyhow::bail!(format!("Expected exactly one entry in {:?}, found {}", work_dir, entries.len()));
        }

        let entry = &entries[0];
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            anyhow::bail!(format!("{:?} is not a directory", entry_path));
        }

        if target_dir.exists() {
             if force {
                 debug!("removing previous {:?}", target_dir);
                 fs::remove_dir_all(&target_dir)?;
             } else {
                 anyhow::bail!(format!("{:?} already exists", target_dir));
             }
        }

        debug!("renaming {:?} to {:?}", entry_path, target_dir);
        fs::rename(&entry_path, target_dir).expect("bouzouki");
        Ok(())
    }

    pub fn uninstall(&self, candidate: &str, version: &str) -> anyhow::Result<()> {
        let target_dir = self.candidate_version(candidate, version).path;
        Ok(fs::remove_dir_all(target_dir)?)
    }

    pub fn set_default(&self, candidate: &str, version: &str) -> anyhow::Result<()> {
        let default_dir = self.default_candidate(candidate);
        remove_symlink_auto(&default_dir)?;
        debug!("symlinking {:?} to {:?}", default_dir, self.candidate_version(candidate, version).path);
        Ok(symlink::symlink_dir(default_dir, self.candidate_version(candidate, version).path)?)
    }

    pub fn is_default(&self, candidate: &str, version: &str) -> bool {
        let v = &self.candidate_version(candidate, version).path;
        match fs::read_link(self.default_candidate(candidate)) {
            Ok(p) => p.eq(v),
            Err(_) => false
        }
    }

    pub fn adjust_env(&self, candidate: &str, version: &str) -> anyhow::Result<()> {
        let any_active = self.candidate_path(candidate);
        let path = env::var_os("PATH").unwrap_or(OsString::new());
        let mut paths: Vec<_> = env::split_paths(&path)
            .filter(|p| !p.starts_with(&any_active))
            .collect();

        let new_candidate = self.candidate_version(candidate, version);
        paths.push(new_candidate.bin());
        let new_path = env::join_paths(paths).expect("Failed to join paths");
        unsafe {
            env::set_var("PATH", &new_path);
            env::set_var(new_candidate.home, &new_candidate.path);
        }
        Ok(())
    }
}