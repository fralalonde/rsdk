use std::process::Command;

fn main() {
    // Derive a human-friendly version string from the Git tag.
    //
    //   Tagged release       → "0.5.5"
    //   Dirty on tag         → "0.5.5+dirty"
    //   Between tags         → "0.5.5-dev.3+gabcdef"
    //   Dirty between tags   → "0.5.5-dev.3+gabcdef.dirty"
    //   No tags / bare       → Cargo.toml fallback
    let fallback = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();

    let desc = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string());

    let version = match desc.as_deref() {
        None => fallback,
        Some(raw) => {
            let raw = raw.strip_prefix('v').unwrap_or(raw);
            let suffix = if raw.ends_with("dirty") {
                // Save 'dirty' marker and remove it before parsing.
                raw.trim_end_matches("-dirty")
            } else {
                raw
            };

            let base = if suffix.contains('-') {
                // v0.5.5-3-gabcdef  →  0.5.5-dev.3+gabcdef
                let parts: Vec<&str> = suffix.splitn(3, '-').collect();
                if parts.len() == 3 {
                    format!("{}-dev.{}+{}", parts[0], parts[1], parts[2])
                } else {
                    suffix.to_string()
                }
            } else {
                suffix.to_string()
            };

            if suffix != raw {
                format!("{}+dirty", base)
            } else {
                base
            }
        }
    };

    println!("cargo:rustc-env=RSDK_VERSION={version}");
    println!("cargo:rerun-if-changed=.git/HEAD");
}
