use fs::create_dir_all;
use std::{fs, io};
use std::path::{PathBuf};
use directories::UserDirs;
use crate::version::ToolVersion;

#[derive(Clone)]
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

    pub fn current_default(&self, tool: &str) -> anyhow::Result<Option<ToolVersion>> {
        let def = self.default_symlink_path(tool);
        if !def.exists() {
            return Ok(None);
        }
        let linked = fs::read_link(def)?;
        Ok(linked.as_path().iter().last()
            .and_then(|version| version.to_str().map(|version| version.to_owned()))
            .map(|version| ToolVersion::new(self, tool, &version)))
    }

    pub fn all_defaults(&self) -> anyhow::Result<Vec<ToolVersion>> {
        let mut defaults = Vec::new();
        let tool_dir = self.tools();

        // Iterate over directories in the `tools` path
        for entry in fs::read_dir(tool_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(tool) = entry.file_name().to_str() {
                    // Check if there is a default version for this tool
                    if let Some(default_version) = self.current_default(tool)? {
                        defaults.push(default_version);
                    }
                }
            }
        }
        Ok(defaults)
    }

    pub fn all_versions(&self) -> anyhow::Result<Vec<ToolVersion>> {
        let mut versions = Vec::new();
        let tools_dir = self.tools();

        // Iterate over directories in the `tools` path
        for entry in fs::read_dir(tools_dir)? {
            let c_entry = entry?;
            if c_entry.file_type()?.is_dir() {
                if let Some(tool) = c_entry.file_name().to_str() {
                    for v_entry in fs::read_dir(self.tool_dir(tool))? {
                        let vv = v_entry?;
                        if vv.file_type()?.is_dir() {
                            if let Some(version) = vv.file_name().to_str() {
                                let cv = ToolVersion::new(self, tool, version);
                                versions.push(cv);
                            }
                        }
                    }
                }
            }
        }
        Ok(versions)
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