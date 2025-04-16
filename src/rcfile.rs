use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use anyhow::bail;
use crate::rsdk_home_dir::RsdkHomeDir;
use crate::tool_version::{ToolVersion};

pub const SDKMAN_RC: &str = ".sdkmanrc";

type Sdkmanrc = HashMap<String, String>;

pub fn env_apply(home: &RsdkHomeDir) -> anyhow::Result<()> {
    if let Some(sdkmanrc) = load()? {
        for tv in &sdkmanrc {
            if !ToolVersion::new(home, &tv.0, &tv.1).is_installed() {
                bail!("Tool {} version {} is not installed, run 'rsdk env install' first.", tv.0, tv.1)
            }
        }
        for tv in sdkmanrc {
            ToolVersion::new(home, &tv.0, &tv.1).make_current()?;
        }
    } else {
        bail!("no .sdkmanrc file found in current directory.")
    }
    Ok(())
}

pub fn env_init(home: &RsdkHomeDir) -> anyhow::Result<()> {
    let mut kv = HashMap::new();
    for tv in home.all_installed()? {
        if tv.is_current() {
            kv.insert(tv.tool, tv.version);
        }
    }
    save(kv)
}

pub fn env_install(home: &RsdkHomeDir) -> anyhow::Result<()> {
    if let Some(sdkmanrc) = load()? {
        for tv in sdkmanrc {
            ToolVersion::install(home, &tv.0, &Some(tv.1))?;
        }
    }
    Ok(())
}

pub fn env_clear(home: &RsdkHomeDir) -> anyhow::Result<()> {
    for tv in home.all_defaults()? { tv.make_current()? }
    Ok(())
}

fn load() -> anyhow::Result<Option<Sdkmanrc>> {
    let path = Path::new(SDKMAN_RC);
    if path.exists() {
        let file = File::open(path)?;
        Ok(Some(serde_ini::from_read(&file)?))
    } else {
        Ok(None)
    }
}

fn save(sdkmanrc: Sdkmanrc) -> anyhow::Result<()> {
    let path = SDKMAN_RC;
    let file = File::create(path)?;
    Ok(serde_ini::to_writer(&file, &sdkmanrc)?)
}