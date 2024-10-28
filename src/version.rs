use std::fs::{create_dir_all, File};
use std::{env, fs, io};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use anyhow::{bail, Context};
use log::{debug};
use symlink::remove_symlink_dir;
use crate::{shell};
use crate::dir::RsdkDir;

#[cfg(unix)]
use tar::Archive;
#[cfg(unix)]
use flate2::bufread::GzDecoder;

#[cfg(windows)]
use zip::ZipArchive;

pub struct CandidateVersion {
    rsdk: RsdkDir,
    candidate: String,
    version: String,
}

impl CandidateVersion {
    pub fn new(dir: &RsdkDir, candidate: &str, version: &str) -> CandidateVersion {
        CandidateVersion {
            rsdk: dir.clone(),
            candidate: candidate.to_string(),
            version: version.to_string(),
        }
    }

    pub fn bin(&self) -> PathBuf {
        self.path().join("bin")
    }

    pub fn path(&self) -> PathBuf {
        self.rsdk.candidate_path(&self.candidate).join(&self.version)
    }

    pub fn home(&self) -> String {
        format!("{}_HOME", self.candidate.to_uppercase())
    }

    pub fn make_current(&self) -> anyhow::Result<()> {
        let any_active = self.rsdk.candidate_path(&self.candidate);
        let path = env::var_os("PATH").unwrap_or_default();
        let mut paths: Vec<_> = env::split_paths(&path)
            .filter(|p| !p.starts_with(&any_active))
            .collect();

        paths.push(self.bin());
        let new_path = env::join_paths(paths).expect("Failed to join paths");
        shell::set_var("PATH", &new_path.to_string_lossy())?;
        shell::set_var(&self.home(), &self.path().to_string_lossy())?;
        Ok(())
    }

    pub fn install_from_file(&self, zipfile: &Path, work_dir: &Path, force: bool) -> anyhow::Result<()> {
        let archive = File::open(zipfile).context("opening zip")?;
        Self::extract(&archive, work_dir)?;

        // unzip complete, proceed to move to final dest
        let target_dir = &self.path();

        let entries = fs::read_dir(work_dir)?
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

        if entries.len() != 1 {
            bail!(format!("Expected exactly one entry in {:?}, found {}", work_dir, entries.len()));
        }

        let entry = &entries[0];
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            bail!(format!("{:?} is not a directory", entry_path));
        }

        if target_dir.exists() {
            if force {
                debug!("removing previous {:?}", target_dir);
                fs::remove_dir_all(target_dir)?;
            } else {
                bail!(format!("{:?} already exists", target_dir));
            }
        }

        if let Some(parent) = target_dir.parent() {
            create_dir_all(parent)?;
        }

        debug!("renaming {:?} to {:?}", entry_path, target_dir);
        fs::rename(&entry_path, target_dir)?;
        Ok(())
    }

    #[cfg(unix)]
    fn extract(file: &File, work_dir: &Path) -> anyhow::Result<()> {
        let decompressed = GzDecoder::new(BufReader::new(file));
        let mut archive = Archive::new(decompressed);
        archive.unpack(work_dir)?;
        Ok(())
    }

    #[cfg(windows)]
    fn extract(file: &File, work_dir: &Path) -> anyhow::Result<()> {
        debug!("unzipping");
        let mut archive = ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut zip_entry = archive.by_index(i)?;
            let outpath = work_dir.join(zip_entry.name());

            // Create directories as needed
            if zip_entry.is_dir() {
                debug!("creating dir {:?}", outpath);
                create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !!parent.exists() {
                        debug!("creating parent dir {:?}", parent);
                        create_dir_all(parent)?;
                    }
                }
                debug!("creating file {:?}", outpath);
                let mut outfile = File::create(&outpath)?;
                debug!("writing file {:?}", outpath);
                io::copy(&mut zip_entry, &mut outfile)?;
            }
        }
        Ok(())
    }

    pub fn uninstall(&self) -> anyhow::Result<()> {
        let target_dir = self.path();
        if !target_dir.exists() {
            bail!(format!("no candidate {} version {}", self.candidate, self.version))
        }
        // TODO deal with default & env
        Ok(fs::remove_dir_all(target_dir)?)
    }

    pub fn set_default(&self) -> anyhow::Result<()> {
        let current_version = self.rsdk.current_default(&self.candidate)?;
        let default_symlink_path = self.rsdk.default_symlink_path(&self.candidate);
        if let Some(current) = current_version {
            debug!("removing previous symlink {:?} to version {}", default_symlink_path, current.version);
            remove_symlink_dir(&default_symlink_path)?;
        }
        debug!("symlinking {:?} to {:?}", self.path(), default_symlink_path);
        Ok(symlink::symlink_dir(self.path(), default_symlink_path)?)
    }

    // pub fn is_default(&self) -> bool {
    //     let cdef_path = self.rsdk.candidate_path(&self.candidate).join("default");
    //     match fs::read_link(&cdef_path) {
    //         Ok(p) => p.eq(&self.path()),
    //         Err(_) => false
    //     }
    // }
}
