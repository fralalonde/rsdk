use fs::create_dir_all;
use std::{fs, io};
use std::path::{PathBuf};
use directories::UserDirs;
use crate::tool_version::{resolve_symlink, ToolVersion};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RsdkHome {
    pub root: PathBuf,
}

impl RsdkHome {
    pub fn new() -> io::Result<RsdkHome> {
        let user_dirs = UserDirs::new().expect("Failed to get user directories");
        let home_dir = user_dirs.home_dir();

        let rsdk_dir = home_dir.join(".rsdk");

        RsdkHome::at(rsdk_dir)
    }

    /// Create an `RsdkHome` rooted at an arbitrary directory. Used by tests to
    /// avoid touching the real `~/.rsdk`.
    pub fn at(rsdk_dir: PathBuf) -> io::Result<RsdkHome> {
        let rsdk = RsdkHome { root: rsdk_dir };
        create_dir_all(rsdk.tools())?;
        create_dir_all(rsdk.cache())?;
        create_dir_all(rsdk.temp())?;
        fs::remove_dir_all(rsdk.temp())?;
        Ok(rsdk)
    }

    pub fn default_symlink_path(&self, tool: &str) -> PathBuf {
        self.tool_dir(tool).join("default")
    }

    pub fn current_symlink_path(&self, tool: &str) -> PathBuf {
        self.tool_dir(tool).join("current")
    }

    #[allow(unused)]
    pub fn default_version(&self, tool: &str) -> color_eyre::Result<Option<ToolVersion>> {
        Ok(self
            .installed_versions(tool)?
            .find(|version| version.is_default()))
    }

    /// The version currently pointed at by the tool's `current` symlink, if any.
    pub fn current_version(&self, tool: &str) -> color_eyre::Result<Option<ToolVersion>> {
        Ok(self
            .installed_versions(tool)?
            .find(|version| version.is_current()))
    }

    /// Resolve the active version's install path for `tool`, in priority order:
    /// the `current` symlink, then the `default` symlink, then the tool's
    /// `*_HOME` environment variable. The last two cover installs and shells
    /// that predate the `current` symlink. Returns `None` if nothing resolves
    /// to an existing directory under the tool dir.
    pub fn resolve_current(&self, tool: &str) -> Option<PathBuf> {
        let tool_dir = self.tool_dir(tool);
        let candidates = [
            resolve_symlink(&self.current_symlink_path(tool)),
            resolve_symlink(&self.default_symlink_path(tool)),
            std::env::var_os(crate::tool_version::home_env(tool)).map(PathBuf::from),
        ];
        candidates
            .into_iter()
            .flatten()
            .find(|path| path.is_dir() && path.starts_with(&tool_dir))
    }

    pub fn installed_versions<'a>(&'a self, tool: &'a str) -> color_eyre::Result<impl Iterator<Item=ToolVersion> + 'a> {
        Ok(self
            .all_installed()?
            .filter(|version| version.tool.eq(tool)))
    }

    /// Used at init time
    pub fn all_defaults(&self) -> color_eyre::Result<impl Iterator<Item=ToolVersion> + '_> {
        Ok(self
            .all_installed()?
            .filter(|version| version.is_default()))
    }

    pub fn all_installed(&self) -> color_eyre::Result<impl Iterator<Item=ToolVersion> + '_> {
        let tools_dir = self.tools();

        let tool_iter = fs::read_dir(tools_dir)?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().ok().is_some_and(|ft| ft.is_dir()))
            .flat_map(move |tool_entry| {
                let tool_name = tool_entry.file_name().into_string().ok()?;
                let tool_dir = self.tool_dir(&tool_name);
                Some(
                    fs::read_dir(tool_dir)
                        .ok()?
                        .filter_map(Result::ok)
                        .filter(|entry| entry.file_type().ok().is_some_and(|ft| ft.is_dir()))
                        .filter_map(move |version_entry| {
                            let version_name = version_entry.file_name().into_string().ok()?;
                            Some(ToolVersion::new(self, &tool_name, &version_name))
                        }),
                )
            })
            .flatten();

        Ok(tool_iter)
    }

    pub fn tools(&self) -> PathBuf {
        self.root.join("tools")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
    pub fn temp(&self) -> PathBuf {
        self.root.join("temp")
    }

    pub fn tool_dir(&self, tool: &str) -> PathBuf {
        self.tools().join(tool)
    }
}