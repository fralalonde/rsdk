//! Integration tests for `rsdk init`: the emitted PATH must only contain
//! `<tool>/current/bin` entries that actually resolve, otherwise a newly
//! opened shell has a dead PATH entry and the tool looks "installed but not on
//! PATH".
//!
//! The bug this covers: a tool with a `default` symlink but no `current`
//! (installs predating the symlink model, or `rsdk default` run on such a
//! tool) got `current/bin` on PATH with nothing behind it.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RSDK: &str = env!("CARGO_BIN_EXE_rsdk");

// Paths are assembled component by component: joining a single
// "a/b/c" string keeps the forward slashes in the displayed path, which no
// longer matches what the binary emits on Windows.

fn root(home: &Path) -> PathBuf {
    home.join(".rsdk")
}

fn tool_dir(home: &Path, tool: &str) -> PathBuf {
    root(home).join("tools").join(tool)
}

fn version_dir(home: &Path, tool: &str, version: &str) -> PathBuf {
    tool_dir(home, tool).join(version)
}

fn current_link(home: &Path, tool: &str) -> PathBuf {
    tool_dir(home, tool).join("current")
}

fn current_bin(home: &Path, tool: &str) -> PathBuf {
    current_link(home, tool).join("bin")
}

/// A throwaway rsdk root with the given tools/versions fabricated on disk.
fn fake_home(tag: &str) -> PathBuf {
    let home = env::temp_dir().join(format!("rsdk-init-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(root(&home).join("tools")).unwrap();
    home
}

/// Fabricate an installed version: `<tool>/<version>/bin` on disk.
fn fake_version(home: &Path, tool: &str, version: &str) {
    fs::create_dir_all(version_dir(home, tool, version).join("bin")).unwrap();
}

fn symlink(target: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(target, link).unwrap();
}

/// Run `rsdk --shell fish --envout <file> init` against `home` and return the
/// emitted shell snippet.
fn run_init(home: &Path) -> String {
    let envout = home.join("envout.fish");
    let output = Command::new(RSDK)
        // RSDK_HOME (not HOME) is what the binary honors, so the child gets a
        // private root on every platform — `directories` ignores HOME on
        // Windows, where the runner's real profile would be used instead.
        .env("RSDK_HOME", root(home))
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .args([
            "--shell",
            "fish",
            "--envout",
            envout.to_str().unwrap(),
            "init",
        ])
        .output()
        .expect("failed to run rsdk init");
    assert!(
        output.status.success(),
        "rsdk init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(&envout).unwrap_or_default()
}

/// PATH list separator of the platform the client was built for.
fn path_separator() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

/// PATH entries emitted for rsdk-managed tools, as absolute paths.
fn tool_path_entries(snippet: &str) -> Vec<String> {
    let sep = path_separator();
    snippet
        .lines()
        .find(|line| line.contains(" PATH "))
        .and_then(|line| line.split('\'').nth(1))
        .map(|path| {
            path.split(sep)
                .filter(|p| p.contains("rsdk") && p.contains("tools"))
                .map(|p| p.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// The `<TOOL>_HOME` value emitted by init, if any.
fn emitted_home(snippet: &str, tool: &str) -> Option<String> {
    snippet
        .lines()
        .find(|line| line.contains(&format!("{}_HOME '", tool.to_uppercase())))
        .and_then(|line| line.split('\'').nth(1))
        .map(|v| v.to_string())
}

#[test]
fn init_materializes_current_for_a_default_only_tool() {
    let home = fake_home("default-only");
    fake_version(&home, "java", "21.0.6-tem");
    fake_version(&home, "java", "25.0.4-tem");
    // Legacy state: a default but no `current`.
    symlink(
        &version_dir(&home, "java", "25.0.4-tem"),
        &tool_dir(&home, "java").join("default"),
    );

    let snippet = run_init(&home);

    // `init` must have converged the symlink instead of advertising a dead
    // PATH entry.
    let current = current_link(&home, "java");
    assert!(
        current.is_dir(),
        "init did not create the java current symlink"
    );
    assert_eq!(
        fs::read_link(&current).unwrap(),
        version_dir(&home, "java", "25.0.4-tem")
    );
    assert!(
        tool_path_entries(&snippet).contains(&current_bin(&home, "java").display().to_string()),
        "java current/bin missing from PATH: {snippet}"
    );
    // The *_HOME var follows the resolved current version.
    assert_eq!(
        emitted_home(&snippet, "java"),
        Some(
            version_dir(&home, "java", "25.0.4-tem")
                .display()
                .to_string()
        ),
        "JAVA_HOME not emitted: {snippet}"
    );
}

#[test]
fn init_includes_current_only_tool_and_skips_unresolvable_one() {
    let home = fake_home("current-only");
    // current but no default: previously left out of PATH entirely.
    fake_version(&home, "maven", "3.9.9");
    symlink(
        &version_dir(&home, "maven", "3.9.9"),
        &current_link(&home, "maven"),
    );
    // Installed but neither default nor current: nothing to activate.
    fake_version(&home, "scala", "3.7.0");

    let snippet = run_init(&home);
    let entries = tool_path_entries(&snippet);

    assert!(
        entries.contains(&current_bin(&home, "maven").display().to_string()),
        "maven current/bin missing from PATH: {snippet}"
    );
    assert!(
        entries.iter().all(|p| !p.contains("scala")),
        "scala has no current version and must stay off PATH: {snippet}"
    );
}

#[test]
fn init_keeps_an_existing_current() {
    // `init` runs on every shell start, so it must never reset `current` back
    // to `default` — that would undo `rsdk use`.
    let home = fake_home("keeps-current");
    fake_version(&home, "java", "21.0.6-tem");
    fake_version(&home, "java", "25.0.4-tem");
    symlink(
        &version_dir(&home, "java", "25.0.4-tem"),
        &tool_dir(&home, "java").join("default"),
    );
    symlink(
        &version_dir(&home, "java", "21.0.6-tem"),
        &current_link(&home, "java"),
    );

    let snippet = run_init(&home);

    assert_eq!(
        fs::read_link(current_link(&home, "java")).unwrap(),
        version_dir(&home, "java", "21.0.6-tem"),
        "init overrode the active version with the default"
    );
    assert_eq!(
        emitted_home(&snippet, "java"),
        Some(
            version_dir(&home, "java", "21.0.6-tem")
                .display()
                .to_string()
        ),
        "JAVA_HOME should follow current, not default: {snippet}"
    );
}

#[test]
fn init_is_idempotent() {
    let home = fake_home("idempotent");
    fake_version(&home, "java", "25.0.4-tem");
    symlink(
        &version_dir(&home, "java", "25.0.4-tem"),
        &tool_dir(&home, "java").join("default"),
    );

    let first = run_init(&home);
    let first_entries = tool_path_entries(&first);
    let second = run_init(&home);
    let second_entries = tool_path_entries(&second);

    // Running init again must not stack duplicate tool entries (each run sees
    // a PATH that already carries them and de-duplicates).
    assert_eq!(second_entries.len(), first_entries.len());
    let mut sorted = second_entries.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), second_entries.len(), "duplicate PATH entries");
}
