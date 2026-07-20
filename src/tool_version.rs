use std::fs::{create_dir_all};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use eyre::{bail};
use log::{debug};
use symlink::remove_symlink_dir;
use crate::{sdkman_client, shell};
use crate::rsdk_home::RsdkHome;

use crate::cache::CacheEntry;
use crate::archive::{extract_tgz, extract_zip};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
use std::{io};

#[derive(Debug, Eq, PartialEq)]
pub struct ToolVersion {
    rsdk: RsdkHome,
    pub tool: String,
    pub version: String,
}

impl Display for ToolVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.tool, self.version))
    }
}

impl ToolVersion {
    pub fn new(dir: &RsdkHome, tool: &str, version: &str) -> ToolVersion {
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
        home_env(&self.tool)
    }

    pub fn install(home: &RsdkHome, tool: &String, version : &Option<String>) -> color_eyre::Result<(ToolVersion, bool)> {
        let api = sdkman_client::SdkManClient::new(&home.cache());
        let version = match version {
            None => api.get_default_version(tool)?,
            Some(v) => v.clone(),
        };

        let tv = ToolVersion::new(home, tool, &version);
        if tv.is_installed() {
            return Ok((tv, false));
        }

        println!("Installing {tool} {version}");
        let temp_dir = home.temp();
        let work_dir = temp_dir.join("work");

        let archive = api.get_cached_file(tool, &version)?;
        debug!("archive is {:?}", archive.file_path());
        tv.install_from_file(&archive, &work_dir, true)?;
        Ok((tv, true))
    }

    /// Monitored variant: reports download progress via `on_progress(bytes, total)`
    /// and aborts when `cancel` is set. The caller should run this on a worker
    /// thread so the TUI can keep polling events.
    pub fn install_monitored(
        home: &RsdkHome,
        tool: &str,
        version: &str,
        on_progress: &mut dyn FnMut(u64, u64),
        cancel: &std::sync::atomic::AtomicBool,
    ) -> color_eyre::Result<(ToolVersion, bool)> {
        let api = sdkman_client::SdkManClient::new(&home.cache());
        let tv = ToolVersion::new(home, tool, version);
        if tv.is_installed() {
            return Ok((tv, false));
        }

        let temp_dir = home.temp();
        let work_dir = temp_dir.join("work");

        let archive = api.get_cached_file_monitored(tool, version, on_progress, cancel)?;
        debug!("archive is {:?}", archive.file_path());
        tv.install_from_file(&archive, &work_dir, true)?;
        Ok((tv, true))
    }

    fn install_from_file(&self, archive: &CacheEntry, work_dir: &Path, force: bool) -> color_eyre::Result<()> {
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

    pub fn uninstall(&self) -> color_eyre::Result<()> {
        let target_dir = self.path();
        debug!("deleting dir {:?}", target_dir);
        if !target_dir.exists() {
            bail!(format!("no tool {} version {}", self.tool, self.version))
        }
        // Remove the `current` symlink if it points at the version being removed,
        // so it is never left dangling.
        if self.is_current() {
            debug!("removing current symlink for deleted version");
            let _ = remove_symlink_dir(self.rsdk.current_symlink_path(&self.tool));
        }
        debug!("deleting all of {:?}", target_dir);
        Ok(fs::remove_dir_all(target_dir)?)
    }

    pub fn make_default(&self) -> color_eyre::Result<()> {
        point_symlink(&self.rsdk.default_symlink_path(&self.tool), &self.path())
    }

    /// Point the tool's `current` symlink at this version and emit the
    /// corresponding `*_HOME` environment variable for the wrapper to apply.
    /// `PATH` is intentionally left untouched: it already contains the stable
    /// `<tool>/current/bin` entry emitted by `rsdk init`, so flipping the
    /// symlink is all that is needed (same model as SDKMAN).
    pub fn make_current(&self) -> color_eyre::Result<()> {
        point_symlink(&self.rsdk.current_symlink_path(&self.tool), &self.path())?;
        shell::set_env_var_after_exit(&self.home(), &self.path().to_string_lossy())?;
        Ok(())
    }

    pub fn is_installed(&self) -> bool {
        self.path().exists()
    }

    pub fn is_current(&self) -> bool {
        self.rsdk
            .resolve_current(&self.tool)
            .is_some_and(|cur| path_eq(&cur, &self.path()))
    }

    pub fn is_default(&self) -> bool {
        symlink_points_at(&self.rsdk.default_symlink_path(&self.tool), &self.path())
    }
}

pub fn home_env(tool: &str) -> String {
    format!("{}_HOME", tool.to_uppercase())
}

/// Repoint the symlink at `link` so it targets `target`, replacing any
/// existing symlink (or stale file) in the way.
fn point_symlink(link: &Path, target: &Path) -> color_eyre::Result<()> {
    if fs::symlink_metadata(link).is_ok() {
        debug!("removing previous symlink {:?}", link);
        remove_symlink_dir(link)?;
    }
    debug!("creating symlink {:?} -> {:?}", link, target);
    Ok(symlink::symlink_dir(target, link)?)
}

/// True if `link` is a symlink that resolves to `target`.
fn symlink_points_at(link: &Path, target: &Path) -> bool {
    resolve_symlink(link).is_some_and(|resolved| path_eq(&resolved, target))
}

/// Resolve a symlink to the absolute path it points at, resolving relative
/// targets against the link's parent. Returns `None` if `link` is not a
/// readable symlink.
pub(crate) fn resolve_symlink(link: &Path) -> Option<PathBuf> {
    let raw = fs::read_link(link).ok()?;
    if raw.is_absolute() {
        Some(raw)
    } else {
        link.parent().map(|p| p.join(&raw)).or(Some(raw))
    }
}

/// Lexically normalize and compare two paths (handles `.` / `..` / duplicate
/// separators) without requiring the paths to exist.
fn path_eq(a: &Path, b: &Path) -> bool {
    normalize(a) == normalize(b)
}

fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
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
