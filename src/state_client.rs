//! Client for SDKMAN's next-generation State API (`state.sdkman.io`), the
//! JSON backend of the official Rust native components (sdkman-cli-native).
//!
//! Serves candidates (name/description/website) and per-candidate version
//! records (platform/distribution/URL) as plain JSON, replacing the legacy
//! `/2` endpoints whose pipe-and-tilde tables need bespoke decoding
//! (`sdkman_decode.rs`). Version records also carry direct vendor URLs, which
//! installs download from first; the legacy broker is only a fallback.

use color_eyre::Result;
use eyre::Context;
use serde_derive::Deserialize;
use std::path::Path;

use crate::http_client::CachedHttpClient;
use crate::sdkman_client::PLATFORM;

pub static DEFAULT_STATE_URL: &str = "https://state.sdkman.io";

/// A candidate entry from `GET /candidates`.
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub candidate: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub website_url: String,
    /// Default version of the candidate. Absent for multi-vendor candidates
    /// like `java` (no single default).
    #[serde(default)]
    pub default: Option<String>,
}

/// A version record from `GET /versions/{candidate}` — one per
/// (version, distribution, platform) combination.
#[derive(Debug, Clone, Deserialize)]
pub struct Version {
    pub candidate: String,
    pub version: String,
    #[serde(default)]
    pub platform: String,
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Vendor distribution for multi-vendor candidates (e.g. `SEMERU`);
    /// absent for single-vendor ones like maven.
    #[serde(default)]
    pub distribution: Option<String>,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub sha256sum: Option<String>,
    #[serde(default)]
    pub sha512sum: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Map rsdk's platform constants (`linuxx64`, …) to the State API's
/// canonical platform enum. Unknown platforms yield `None` (records are
/// then matched without a platform preference).
fn state_platform(rsdk_platform: &str) -> Option<&'static str> {
    match rsdk_platform {
        "linuxx64" => Some("LINUX_X64"),
        "linuxarm64" => Some("LINUX_ARM64"),
        "darwinx64" => Some("MAC_X64"),
        "darwinarm64" => Some("MAC_ARM64"),
        "windowsx64" => Some("WINDOWS_X64"),
        _ => None,
    }
}

/// (State API distribution enum, legacy vendor suffix) — the single source of
/// truth for both directions: composing identifiers from records and splitting
/// identifiers for install resolution.
const DISTRIBUTION_SUFFIXES: &[(&str, &str)] = &[
    ("BISHENG", "bisheng"),
    ("CORRETTO", "amzn"),
    ("ELIYA", "eliya"),
    ("GRAALCE", "graalce"),
    ("GRAALVM", "graal"),
    ("JETBRAINS", "jbr"),
    ("KONA", "kona"),
    ("LIBERICA", "librca"),
    ("LIBERICA_NIK", "nik"),
    ("MANDREL", "mandrel"),
    ("MICROSOFT", "ms"),
    ("OPENJDK", "open"),
    ("ORACLE", "oracle"),
    ("SAP_MACHINE", "sapmchn"),
    ("SEMERU", "sem"),
    ("TEMURIN", "tem"),
    ("ZULU", "zulu"),
];

/// Map a State API `distribution` to the vendor suffix used in legacy
/// identifiers: version `27.0.0` + distribution `SEMERU` composes to the
/// identifier `27.0.0-sem` that rsdk (and SDKMAN itself) use everywhere.
///
/// Suffixes verified against the live legacy identifier table
/// (`/candidates/java/linuxx64/versions/list`), including the odd ones:
/// `-tem`, `-amzn`, `-sapmchn`, `-librca`, `-nik`, `-jbr`, `-ms`, `-kona`.
/// Note `fx`/`crac` are NOT distributions — they are version variants that
/// live inside the version string itself (`27.0.0-fx+35` + ZULU →
/// `27.0.0-fx+35-zulu`), so they need no mapping here.
pub(crate) fn distribution_suffix(distribution: &str) -> Option<&'static str> {
    // Case-insensitive: accepts the State API's canonical enum (`SEMERU`).
    let upper = distribution.to_ascii_uppercase();
    DISTRIBUTION_SUFFIXES
        .iter()
        .find(|(d, _)| *d == upper)
        .map(|(_, s)| *s)
}

/// Legacy-style display identifier for a version record: bare version plus
/// vendor suffix, or the bare version when the record has no (known)
/// distribution — which is the case for most non-java candidates.
pub(crate) fn display_version(v: &Version) -> String {
    match v.distribution.as_deref().and_then(distribution_suffix) {
        Some(suffix) => format!("{}-{}", v.version, suffix),
        None => v.version.clone(),
    }
}

/// Collapse version records (one per platform/distribution) into legacy
/// display identifiers. Records sharing an identifier (the same version on
/// several platforms) are deduplicated, preferring the record matching
/// `rsdk_platform`, then `UNIVERSAL`, then any. Returned names are sorted
/// descending (latest first) for determinism.
pub(crate) fn display_names(records: Vec<Version>, rsdk_platform: &str) -> Vec<String> {
    let native = state_platform(rsdk_platform);
    let rank = |v: &Version| match native {
        Some(p) if v.platform.eq_ignore_ascii_case(p) => 0,
        _ if v.platform.eq_ignore_ascii_case("UNIVERSAL") => 1,
        _ => 2,
    };

    let mut best: std::collections::HashMap<String, (u8, Version)> =
        std::collections::HashMap::new();
    for v in records {
        let name = display_version(&v);
        let r = rank(&v);
        match best.get_mut(&name) {
            Some((best_rank, _)) if *best_rank <= r => {}
            Some((best_rank, slot)) => {
                *best_rank = r;
                *slot = v;
            }
            None => {
                best.insert(name, (r, v));
            }
        }
    }

    let mut names: Vec<String> = best.into_keys().collect();
    names.sort_by(|a, b| b.cmp(a));
    names
}

pub struct StateClient {
    http: CachedHttpClient,
    base_url: String,
}

impl StateClient {
    pub fn new(cache_dir: &Path) -> Self {
        // Allow overriding the API base (offline tests); defaults to the
        // production State API.
        let base_url =
            std::env::var("RSDK_STATE_API_URL").unwrap_or_else(|_| DEFAULT_STATE_URL.to_string());
        Self {
            http: CachedHttpClient::new(cache_dir),
            base_url,
        }
    }

    /// All candidates with their display name, description, website and
    /// (where it exists) default version.
    pub fn get_candidates(&self) -> Result<Vec<Candidate>> {
        let url = format!("{}/candidates", self.base_url);
        let text = self
            .http
            .get_text(&url)
            .with_context(|| format!("failed to fetch {url}"))?;
        serde_json::from_str(&text).with_context(|| format!("failed to decode {url}"))
    }

    /// All (visible) version records for a candidate, across platforms.
    /// Use [`display_names`] to collapse them into legacy identifiers.
    pub fn get_versions(&self, tool: &str) -> Result<Vec<Version>> {
        let url = format!("{}/versions/{tool}", self.base_url);
        let text = self
            .http
            .get_text(&url)
            .with_context(|| format!("failed to fetch {url}"))?;
        serde_json::from_str(&text).with_context(|| format!("failed to decode {url}"))
    }

    /// Resolve a legacy-style identifier (`27.0.0-sem`) to the version record
    /// matching the candidate's platform. `version_part` is the identifier
    /// with any `-suffix` vendor tag stripped; `distribution` is the vendor
    /// tag (lowercase) or `None` for single-vendor candidates.
    ///
    /// Matching prefers the native platform, then `UNIVERSAL`, then any
    /// platform — mirroring [`display_names`] so installs land on the same
    /// record the list view showed.
    pub fn resolve_version(
        &self,
        tool: &str,
        version_part: &str,
        distribution: Option<&str>,
    ) -> Result<Version> {
        let records = self.get_versions(tool)?;
        let native = state_platform(PLATFORM);
        let rank = |v: &Version| match native {
            Some(p) if v.platform.eq_ignore_ascii_case(p) => 0,
            _ if v.platform.eq_ignore_ascii_case("UNIVERSAL") => 1,
            _ => 2,
        };

        let dist_matches = |v: &Version| {
            v.version == version_part
                && v.distribution
                    .as_deref()
                    .and_then(distribution_suffix)
                    .is_some_and(|s| Some(s) == distribution)
        };

        let mut candidates: Vec<&Version> = records.iter().filter(|v| dist_matches(v)).collect();
        // Tolerate a missing platform tag on UNIVERSAL-only candidates (some
        // records leave `platform` empty instead of `UNIVERSAL`).
        if candidates.is_empty() {
            candidates = records
                .iter()
                .filter(|v| v.version == version_part && v.distribution.is_none())
                .collect();
        }
        candidates.sort_by_key(|v| rank(v));

        candidates.into_iter().next().cloned().ok_or_else(|| {
            eyre::eyre!(
                "no {tool} {version_part}{} record for {}",
                distribution.map(|d| format!("-{d}")).unwrap_or_default(),
                native.unwrap_or("any platform")
            )
        })
    }
}

/// Split a legacy display identifier into its version part and vendor tag:
/// `27.0.0-sem` → `("27.0.0", Some("sem"))`, `3.9.16` → `("3.9.16", None)`.
///
/// Known suffixes only — unknown `-tags` stay part of the version string
/// (they are genuine variants, not vendors).
pub(crate) fn split_identifier(identifier: &str) -> (&str, Option<&str>) {
    if let Some((v, tag)) = identifier.rsplit_once('-') {
        // Vendor tags are exactly the lowercase suffixes of known
        // distributions; anything else (`-ea`, `-fx`) is a version variant.
        if DISTRIBUTION_SUFFIXES.iter().any(|(_, s)| *s == tag) {
            return (v, Some(tag));
        }
    }
    (identifier, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_legacy_identifiers() {
        let v = Version {
            candidate: "java".into(),
            version: "27.0.0".into(),
            platform: "LINUX_X64".into(),
            url: "".into(),
            visible: true,
            distribution: Some("SEMERU".into()),
            md5sum: None,
            sha256sum: None,
            sha512sum: None,
        };
        assert_eq!(display_version(&v), "27.0.0-sem");

        // Version variants ride inside the version string (fx/crac are not
        // distributions).
        let v = Version {
            version: "27.0.0-fx+35".into(),
            distribution: Some("ZULU".into()),
            ..v.clone()
        };
        assert_eq!(display_version(&v), "27.0.0-fx+35-zulu");

        // Unknown/absent distribution → bare version (non-java candidates).
        let v = Version {
            candidate: "maven".into(),
            version: "3.9.16".into(),
            distribution: None,
            ..v
        };
        assert_eq!(display_version(&v), "3.9.16");
    }

    #[test]
    fn suffix_table_covers_openapi_enum() {
        // Every distribution the OpenAPI spec defines (minus ELIYA, which is
        // absent from the legacy identifier table too) must map to a suffix.
        for d in [
            "BISHENG",
            "CORRETTO",
            "GRAALCE",
            "GRAALVM",
            "JETBRAINS",
            "KONA",
            "LIBERICA",
            "LIBERICA_NIK",
            "MANDREL",
            "MICROSOFT",
            "OPENJDK",
            "ORACLE",
            "SAP_MACHINE",
            "SEMERU",
            "TEMURIN",
            "ZULU",
        ] {
            assert!(distribution_suffix(d).is_some(), "{d} missing a suffix");
        }
    }

    #[test]
    fn platform_mapping() {
        assert_eq!(state_platform("linuxx64"), Some("LINUX_X64"));
        assert_eq!(state_platform("darwinarm64"), Some("MAC_ARM64"));
        assert_eq!(state_platform("atari"), None);
    }

    fn rec(version: &str, platform: &str) -> Version {
        Version {
            candidate: "java".into(),
            version: version.into(),
            platform: platform.into(),
            url: "".into(),
            visible: true,
            distribution: Some("SEMERU".into()),
            md5sum: None,
            sha256sum: None,
            sha512sum: None,
        }
    }

    #[test]
    fn dedups_platforms_preferring_native_then_universal() {
        let records = vec![
            rec("27.0.0", "WINDOWS_X64"),
            rec("27.0.0", "LINUX_X64"),
            rec("26.0.1", "UNIVERSAL"),
            rec("26.0.1", "MAC_ARM64"),
        ];
        let names = display_names(records, "linuxx64");
        // One entry per identifier, descending order.
        assert_eq!(names, vec!["27.0.0-sem", "26.0.1-sem"]);
    }

    #[test]
    fn splits_known_suffixes_only() {
        // Known vendor suffixes split into (version, distribution).
        assert_eq!(split_identifier("27.0.0-sem"), ("27.0.0", Some("sem")));
        assert_eq!(
            split_identifier("26.0.2+1-amzn"),
            ("26.0.2+1", Some("amzn"))
        );
        assert_eq!(
            split_identifier("27.0.0-fx+35-zulu"),
            ("27.0.0-fx+35", Some("zulu"))
        );
        // Unknown suffixes are version variants, not vendors.
        assert_eq!(split_identifier("25.0.4-ea"), ("25.0.4-ea", None));
        assert_eq!(split_identifier("3.9.16"), ("3.9.16", None));
    }

    #[test]
    fn unknown_rsdk_platform_still_dedups() {
        let records = vec![rec("27.0.0", "LINUX_X64"), rec("27.0.0", "MAC_ARM64")];
        assert_eq!(display_names(records, "atari"), vec!["27.0.0-sem"]);
    }
}
