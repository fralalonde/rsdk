use std::fs::{create_dir_all};
use std::fmt::{Display, Formatter};
use std::{env, fs};
use std::path::{Path, PathBuf};
use anyhow::{bail};
use log::{debug};
use symlink::remove_symlink_dir;
use crate::{shell};
use crate::home::RsdkHomeDir;

use crate::cache::CacheEntry;
use crate::extract::{extract_tgz, extract_zip};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
use std::{io};

#[derive(Debug, Eq, PartialEq)]
pub struct ToolVersion {
    rsdk: RsdkHomeDir,
    pub tool: String,
    pub version: String,
}

impl Display for ToolVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.tool, self.version))
    }
}

impl ToolVersion {
    pub fn new(dir: &RsdkHomeDir, tool: &str, version: &str) -> ToolVersion {
        ToolVersion {
            rsdk: dir.clone(),
            tool: tool.to_string(),
            version: version.to_string(),
        }
    }

    pub fn bin(&self) -> PathBuf {
        self.path().join("bin")
    }

    pub fn path(&self) -> PathBuf {
        self.rsdk.tool_dir(&self.tool).join(&self.version)
    }

    pub fn home(&self) -> String {
        format!("{}_HOME", self.tool.to_uppercase())
    }

    pub fn install_from_file(&self, archive: &CacheEntry, work_dir: &Path, force: bool) -> anyhow::Result<()> {
        if let Err(e) = extract_zip(&archive.file_path(), work_dir) {
            debug!("file is not a zip: {:?}", e);
            if let Err(e) = extract_tgz(&archive.file_path(), work_dir) {
                debug!("file is not a tgz: {:?}", e);
                bail!("file {:?} is neither a zip nor a tgz", archive.file_path())
            }
        }

        // extraction complete, proceed to move to final dest
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

        #[cfg(unix)]
        make_all_files_executable(&self.bin())?;
        Ok(())
    }

    pub fn uninstall(&self) -> anyhow::Result<()> {
        let target_dir = self.path();
        debug!("deleting dir {:?}", target_dir);
        if !target_dir.exists() {
            bail!(format!("no tool {} version {}", self.tool, self.version))
        }
        // TODO deal with default & env
        debug!("deleting all of {:?}", target_dir);
        Ok(fs::remove_dir_all(target_dir)?)
    }

    pub fn make_default(&self) -> anyhow::Result<()> {
        let default_symlink_path = self.rsdk.default_symlink_path(&self.tool);
        debug!("removing previous symlink {:?}", default_symlink_path);
        if default_symlink_path.exists() {
            remove_symlink_dir(&default_symlink_path)?;
        }
        debug!("symlinking {:?} to {:?}", self.path(), default_symlink_path);
        Ok(symlink::symlink_dir(self.path(), default_symlink_path)?)
    }

    pub fn make_current(&self) -> anyhow::Result<()> {
        let any_active = self.rsdk.tool_dir(&self.tool);
        let path = env::var_os("PATH").unwrap_or_default();

        // put bin first on path to take precedence over system-provided packages
        let mut paths = vec![self.bin()];
        env::split_paths(&path)
            .filter(|p| !p.starts_with(&any_active))
            .for_each(|p| paths.push(p));

        let new_path = env::join_paths(paths)?;
        shell::set_var("PATH", &new_path.to_string_lossy())?;
        shell::set_var(&self.home(), &self.path().to_string_lossy())?;
        Ok(())
    }

    pub fn is_installed(&self) -> bool {
        self.path().exists()
    }

    pub fn is_current(&self) -> bool {
        env::var(self.home())
            .map(|home| home.eq(&self.path().to_string_lossy()))
            .unwrap_or(false)
    }

    pub fn is_default(&self) -> bool {
        let cdef_path = self.rsdk.default_symlink_path(&self.tool);
        match fs::read_link(&cdef_path) {
            Ok(p) => p.eq(&self.path()),
            Err(_) => false
        }
    }
}

#[cfg(unix)]
fn make_all_files_executable(path: &Path) -> io::Result<()> {
    debug!("chmod files in {:?} to executable", path);
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        // Only proceed if it's a file (ignore directories)
        if file_type.is_file() {
            let metadata = entry.metadata()?;
            let mut permissions = metadata.permissions();

            // Set the executable bit (octal 0o111 adds exec for user, group, and others)
            debug!("chmod {:?} +x", entry.file_name());
            permissions.set_mode(permissions.mode() | 0o111);
            fs::set_permissions(entry.path(), permissions)?;
        }
    }
    Ok(())
}
