use fs::create_dir_all;
use std::{fs, io};
use std::path::{PathBuf};
use directories::UserDirs;
use crate::tool_version::ToolVersion;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RsdkHomeDir {
    pub root: PathBuf,
}

impl RsdkHomeDir {
    pub fn new() -> io::Result<RsdkHomeDir> {
        let user_dirs = UserDirs::new().expect("Failed to get user directories");
        let home_dir = user_dirs.home_dir();

        let rsdk_dir = home_dir.join(".rsdk");

        let rsdk = RsdkHomeDir { root: rsdk_dir.to_path_buf() };
        create_dir_all(rsdk.tools())?;
        create_dir_all(rsdk.cache())?;
        create_dir_all(rsdk.temp())?;
        fs::remove_dir_all(rsdk.temp())?;
        Ok(rsdk)
    }

    pub fn default_symlink_path(&self, tool: &str) -> PathBuf {
        self.tool_dir(tool).join("default")
    }

    #[allow(unused)]
    pub fn default_version(&self, tool: &str) -> anyhow::Result<Option<ToolVersion>> {
        Ok(self
            .installed_versions(tool)?
            .find(|version| version.is_default()))
    }

    pub fn installed_versions<'a>(&'a self, tool: &'a str) -> anyhow::Result<impl Iterator<Item=ToolVersion> + 'a> {
        Ok(self
            .all_installed()?
            .filter(|version| version.tool.eq(tool)))
    }

    /// Used at init time
    pub fn all_defaults(&self) -> anyhow::Result<impl Iterator<Item=ToolVersion> + '_> {
        Ok(self
            .all_installed()?
            .filter(|version| version.is_default()))
    }

    pub fn all_installed(&self) -> anyhow::Result<impl Iterator<Item=ToolVersion> + '_> {
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