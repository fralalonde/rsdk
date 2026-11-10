//! Publish stable tool links for processes started outside an rsdk shell.

use crate::rsdk_home::RsdkHome;
use eyre::{bail, eyre};
use log::warn;
use std::io;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

const ENVIRONMENT: &[u16] = &[
    b'E' as u16,
    b'n' as u16,
    b'v' as u16,
    b'i' as u16,
    b'r' as u16,
    b'o' as u16,
    b'n' as u16,
    b'm' as u16,
    b'e' as u16,
    b'n' as u16,
    b't' as u16,
    0,
];
const PATH: &[u16] = &[b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0];

/// Add `<rsdk>/tools/<tool>/current/bin` to HKCU\Environment\Path once.
/// The link target changes on `use`, so the registry value never needs a
/// version-specific rewrite. Existing PATH references retain their value type.
pub fn ensure_tool_path(home: &RsdkHome, tool: &str) -> color_eyre::Result<()> {
    let root = if home.root.is_absolute() {
        home.root.clone()
    } else {
        std::env::current_dir()?.join(&home.root)
    };
    let tool_bin = root.join("tools").join(tool).join("current").join("bin");
    let entry = tool_bin.to_string_lossy();
    if entry.contains(';') {
        bail!("rsdk path for {tool} contains ';' and cannot be added to Windows PATH");
    }

    let key = UserEnvironmentKey::open()?;
    let (old_path, value_type) = key.read_path()?;
    if let Some(new_path) = path_with_entry(&old_path, &entry) {
        key.write_path(&new_path, value_type)?;
        // A new Explorer child can inherit the change after this broadcast.
        // A hung window must not turn a successful registry update into failure.
        let result = unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                ENVIRONMENT.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                1000,
                std::ptr::null_mut(),
            )
        };
        if result == 0 {
            warn!("Windows did not confirm the environment-change broadcast");
        }
    }
    Ok(())
}

fn path_with_entry(path: &str, entry: &str) -> Option<String> {
    let normalized = |part: &str| {
        part.trim()
            .trim_matches('"')
            .trim_end_matches(['\\', '/'])
            .to_lowercase()
    };
    if path
        .split(';')
        .any(|part| normalized(part) == normalized(entry))
    {
        return None;
    }
    if path.is_empty() {
        Some(entry.to_owned())
    } else {
        Some(format!("{entry};{path}"))
    }
}

struct UserEnvironmentKey(HKEY);

impl UserEnvironmentKey {
    fn open() -> color_eyre::Result<Self> {
        let mut key = std::ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                ENVIRONMENT.as_ptr(),
                0,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                &mut key,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32).into());
        }
        Ok(Self(key))
    }

    fn read_path(&self) -> color_eyre::Result<(String, u32)> {
        let mut value_type = REG_EXPAND_SZ;
        let mut byte_len = 0;
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                PATH.as_ptr(),
                std::ptr::null(),
                &mut value_type,
                std::ptr::null_mut(),
                &mut byte_len,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok((String::new(), REG_EXPAND_SZ));
        }
        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32).into());
        }
        if value_type != REG_SZ && value_type != REG_EXPAND_SZ {
            bail!("HKCU\\Environment\\Path is not a string registry value");
        }

        loop {
            let mut utf16 = vec![0u16; (byte_len as usize).div_ceil(2).max(1)];
            let mut actual_len = (utf16.len() * 2) as u32;
            let status = unsafe {
                RegQueryValueExW(
                    self.0,
                    PATH.as_ptr(),
                    std::ptr::null(),
                    &mut value_type,
                    utf16.as_mut_ptr().cast(),
                    &mut actual_len,
                )
            };
            if status == ERROR_MORE_DATA {
                byte_len = actual_len;
                continue;
            }
            if status != ERROR_SUCCESS {
                return Err(io::Error::from_raw_os_error(status as i32).into());
            }
            if value_type != REG_SZ && value_type != REG_EXPAND_SZ {
                bail!("HKCU\\Environment\\Path changed to a non-string registry value");
            }
            if actual_len % 2 != 0 {
                bail!("HKCU\\Environment\\Path contains invalid UTF-16 data");
            }
            utf16.truncate(actual_len as usize / 2);
            let end = utf16
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(utf16.len());
            return Ok((String::from_utf16(&utf16[..end])?, value_type));
        }
    }

    fn write_path(&self, path: &str, value_type: u32) -> color_eyre::Result<()> {
        let utf16: Vec<u16> = std::ffi::OsStr::new(path)
            .encode_wide()
            .chain(Some(0))
            .collect();
        if utf16.len() > 32767 {
            bail!("Windows user PATH would exceed 32,767 characters");
        }
        let byte_len =
            u32::try_from(utf16.len() * 2).map_err(|_| eyre!("Windows PATH is too long"))?;
        let status = unsafe {
            RegSetValueExW(
                self.0,
                PATH.as_ptr(),
                0,
                value_type,
                utf16.as_ptr().cast(),
                byte_len,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32).into());
        }
        Ok(())
    }
}

impl Drop for UserEnvironmentKey {
    fn drop(&mut self) {
        unsafe { RegCloseKey(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::path_with_entry;

    #[test]
    fn prepends_once_without_expanding_existing_path() {
        let entry = r"C:\Users\Ada\.rsdk\tools\java\current\bin";
        let old = r"%LOCALAPPDATA%\Programs;C:\Other\bin";
        let updated = path_with_entry(old, entry).unwrap();
        assert_eq!(updated, format!("{entry};{old}"));
        assert_eq!(path_with_entry(&updated, entry), None);
        assert_eq!(path_with_entry(&updated, &entry.to_uppercase()), None);
    }

    #[test]
    fn recognizes_existing_entry_with_trailing_separator() {
        let entry = r"C:\Users\Ada\.rsdk\tools\java\current\bin";
        assert_eq!(
            path_with_entry(&format!("{entry}\\;C:\\Other"), entry),
            None
        );
    }

    #[test]
    fn keeps_multiple_tools_on_path() {
        let java = r"C:\Users\Ada\.rsdk\tools\java\current\bin";
        let maven = r"C:\Users\Ada\.rsdk\tools\maven\current\bin";
        let path = path_with_entry("C:\\Windows", java).unwrap();
        let path = path_with_entry(&path, maven).unwrap();
        assert_eq!(path_with_entry(&path, java), None);
        assert_eq!(path_with_entry(&path, maven), None);
        assert_eq!(path, format!("{maven};{java};C:\\Windows"));
    }
}
