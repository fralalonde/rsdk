//! Integration tests for the SDKMAN-style environment handling: the `current`
//! symlink is the source of truth for "active version" and must behave
//! correctly across the install/use/uninstall/env lifecycle.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rsdk::rcfile;
use rsdk::rsdk_home::RsdkHome;
use rsdk::tool_version::ToolVersion;

/// Serializes tests that mutate the process-wide current directory, which is
/// shared global state and would otherwise race under `cargo test`'s default
/// parallelism.
static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Create an `RsdkHome` rooted in a fresh temp dir.
fn test_home() -> RsdkHome {
    let dir = env::temp_dir().join(format!("rsdk-test-{}", uuid::Uuid::new_v4()));
    RsdkHome::at(dir).expect("failed to create test RsdkHome")
}

/// Fabricate an "installed" tool version on disk (a version dir with a bin/).
fn fake_install(home: &RsdkHome, tool: &str, version: &str) -> ToolVersion {
    let tv = ToolVersion::new(home, tool, version);
    fs::create_dir_all(tv.bin()).expect("failed to fabricate install");
    tv
}

fn read_link(path: &Path) -> PathBuf {
    fs::read_link(path).expect("expected a symlink")
}

// --- current resolution fallback (legacy installs) -------------------------

#[test]
fn current_falls_back_to_default_symlink() {
    // Legacy install: only `default` set, no `current`. Should resolve.
    let home = test_home();
    let tv = fake_install(&home, "java", "25-tem");
    tv.make_default().unwrap();

    assert!(tv.is_current());
    assert_eq!(home.current_version("java").unwrap().unwrap(), tv);
}

#[test]
fn current_symlink_wins_over_default() {
    let home = test_home();
    let default = fake_install(&home, "java", "21-tem");
    let current = fake_install(&home, "java", "17-tem");
    default.make_default().unwrap();
    current.make_current().unwrap();

    assert!(current.is_current());
    assert!(!default.is_current());
}

#[test]
fn current_falls_back_to_home_env_var() {
    // Legacy install: no symlinks at all, only the `*_HOME` env var (the old
    // model). Guard env mutation since it is process-global.
    let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = test_home();
    let tv = fake_install(&home, "java", "25-tem");
    env::set_var("JAVA_HOME", tv.path());
    let result = home.current_version("java").unwrap();
    env::remove_var("JAVA_HOME");

    assert_eq!(result.unwrap(), tv);
}

#[test]
fn env_var_outside_tool_dir_is_ignored() {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = test_home();
    fake_install(&home, "java", "25-tem");
    env::set_var("JAVA_HOME", "/usr/lib/jvm/system-java");
    let resolved = home.resolve_current("java");
    env::remove_var("JAVA_HOME");

    assert!(resolved.is_none());
}

// --- current symlink model -------------------------------------------------

#[test]
fn current_is_not_set_initially() {
    let home = test_home();
    let tv = fake_install(&home, "java", "21-tem");
    assert!(!tv.is_current());
    assert!(home.current_version("java").unwrap().is_none());
}

#[test]
fn make_current_points_current_symlink_at_version() {
    let home = test_home();
    let tv = fake_install(&home, "java", "21-tem");
    tv.make_current().unwrap();

    assert!(tv.is_current());
    assert_eq!(read_link(&home.current_symlink_path("java")), tv.path());
    assert_eq!(home.current_version("java").unwrap().unwrap(), tv);
}

#[test]
fn make_current_is_persistent_across_home_instances() {
    // Regression test for the old env-var-based is_current, which forgot the
    // current version as soon as a new process (fresh env) checked it.
    let home = test_home();
    let tv = fake_install(&home, "java", "21-tem");
    tv.make_current().unwrap();

    // Simulate a brand-new shell / process by rebuilding RsdkHome from the
    // same root. No *_HOME env var is set, yet current must still resolve.
    let fresh = RsdkHome::at(home.root.clone()).unwrap();
    assert!(ToolVersion::new(&fresh, "java", "21-tem").is_current());
    assert_eq!(fresh.current_version("java").unwrap().unwrap(), tv);
}

#[test]
fn use_switches_current_between_versions() {
    let home = test_home();
    let v21 = fake_install(&home, "java", "21-tem");
    let v17 = fake_install(&home, "java", "17-tem");

    v21.make_current().unwrap();
    assert!(v21.is_current());
    assert!(!v17.is_current());

    v17.make_current().unwrap();
    assert!(v17.is_current());
    assert!(!v21.is_current());
    // Only one current symlink exists and it now points at 17.
    assert_eq!(read_link(&home.current_symlink_path("java")), v17.path());
}

// --- default symlink (unchanged behaviour, now sharing the helper) ---------

#[test]
fn make_default_points_default_symlink_at_version() {
    let home = test_home();
    let tv = fake_install(&home, "maven", "3.9.9");
    tv.make_default().unwrap();

    assert!(tv.is_default());
    assert_eq!(read_link(&home.default_symlink_path("maven")), tv.path());
}

#[test]
fn default_and_current_are_independent() {
    let home = test_home();
    let v1 = fake_install(&home, "maven", "3.8.8");
    let v2 = fake_install(&home, "maven", "3.9.9");

    v1.make_default().unwrap();
    v2.make_current().unwrap();

    assert!(v1.is_default());
    assert!(!v1.is_current());
    assert!(v2.is_current());
    assert!(!v2.is_default());
}

// --- uninstall -------------------------------------------------------------

#[test]
fn uninstalling_current_removes_current_symlink() {
    let home = test_home();
    let tv = fake_install(&home, "gradle", "8.7");
    tv.make_current().unwrap();
    assert!(home.current_symlink_path("gradle").exists());

    tv.uninstall().unwrap();

    // current symlink must not be left dangling.
    assert!(!home.current_symlink_path("gradle").exists());
    assert!(!tv.is_installed());
}

#[test]
fn uninstalling_non_current_keeps_current_symlink() {
    let home = test_home();
    let keep = fake_install(&home, "gradle", "8.7");
    let drop = fake_install(&home, "gradle", "8.6");
    keep.make_current().unwrap();

    drop.uninstall().unwrap();

    assert!(keep.is_current());
    assert_eq!(read_link(&home.current_symlink_path("gradle")), keep.path());
}

// --- .sdkmanrc env handling -------------------------------------------------

/// Run a closure in a temp working dir, restoring the original cwd after.
/// `.sdkmanrc` is read/written relative to the current directory.
fn in_temp_dir<F: FnOnce(&Path)>(f: F) {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = env::current_dir().unwrap();
    let dir = env::temp_dir().join(format!("rsdk-cwd-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    env::set_current_dir(&dir).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&dir)));
    env::set_current_dir(&original).unwrap();
    fs::remove_dir_all(&dir).ok();
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn env_init_writes_current_versions_to_sdkmanrc() {
    let home = test_home();
    let java = fake_install(&home, "java", "21-tem");
    let maven = fake_install(&home, "maven", "3.9.9");
    java.make_current().unwrap();
    maven.make_current().unwrap();

    in_temp_dir(|dir| {
        rcfile::env_init(&home).unwrap();
        let written = fs::read_to_string(dir.join(".sdkmanrc")).unwrap();
        assert!(written.contains("java=21-tem"), "got: {written}");
        assert!(written.contains("maven=3.9.9"), "got: {written}");
    });
}

#[test]
fn env_init_skips_tools_with_no_current() {
    let home = test_home();
    fake_install(&home, "java", "21-tem"); // installed but never made current

    in_temp_dir(|dir| {
        rcfile::env_init(&home).unwrap();
        let written = fs::read_to_string(dir.join(".sdkmanrc")).unwrap();
        assert!(!written.contains("java"), "got: {written}");
    });
}

#[test]
fn env_apply_switches_current_from_sdkmanrc() {
    let home = test_home();
    let v17 = fake_install(&home, "java", "17-tem");
    let v21 = fake_install(&home, "java", "21-tem");
    v21.make_current().unwrap();

    in_temp_dir(|dir| {
        fs::write(dir.join(".sdkmanrc"), "java=17-tem\n").unwrap();
        rcfile::env_apply(&home).unwrap();
    });

    assert!(v17.is_current());
    assert!(!v21.is_current());
}

#[test]
fn env_apply_errors_when_tool_not_installed() {
    let home = test_home();
    in_temp_dir(|dir| {
        fs::write(dir.join(".sdkmanrc"), "java=11-tem\n").unwrap();
        let err = rcfile::env_apply(&home).unwrap_err();
        assert!(err.to_string().contains("not installed"), "got: {err}");
    });
}

#[test]
fn env_apply_errors_without_sdkmanrc() {
    let home = test_home();
    in_temp_dir(|_| {
        let err = rcfile::env_apply(&home).unwrap_err();
        assert!(err.to_string().contains(".sdkmanrc"), "got: {err}");
    });
}

#[test]
fn env_clear_restores_default_as_current() {
    let home = test_home();
    let default = fake_install(&home, "java", "21-tem");
    let other = fake_install(&home, "java", "17-tem");
    default.make_default().unwrap();
    other.make_current().unwrap();
    assert!(other.is_current());

    rcfile::env_clear(&home).unwrap();

    assert!(default.is_current());
    assert!(!other.is_current());
}
