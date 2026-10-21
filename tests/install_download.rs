//! Integration tests for the install download path against a local fake of the
//! SDKMAN APIs.
//!
//! These lock in the two failure modes that made `rsdk install` unusable:
//!
//! * the legacy broker answering 503 for a version while the State API vendor
//!   URL works, or the other way round — one side failing must not abort the
//!   install while the other can still deliver the archive;
//! * archives whose top directory has no `bin/` (jmc ships its launcher under
//!   "Azul Mission Control/bin"), which used to abort after the tool had
//!   already been moved into place.

use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use rsdk::rsdk_home::RsdkHome;
use rsdk::tool_version::ToolVersion;

/// A route the fake server answers with a fixed status and body.
#[derive(Clone)]
struct Route {
    status: u16,
    body: Vec<u8>,
}

/// Minimal HTTP/1.1 server: serves `routes` by path, closes after each
/// response, and records every requested path so tests can assert on the
/// download order.
struct FakeApi {
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl FakeApi {
    /// Start a server whose routes are built from the bound base URL (routes
    /// often need to point back at the server itself).
    fn start(routes: impl FnOnce(&str) -> HashMap<String, Route>) -> FakeApi {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake api");
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        let routes = Arc::new(routes(&base));

        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let Some(path) = read_request_path(&stream) else {
                    continue;
                };
                seen.lock().unwrap().push(path.clone());
                respond(&mut stream, path.trim_start_matches('/'), &routes);
            }
        });

        FakeApi { base, requests }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    fn requested(&self, path: &str) -> bool {
        self.requests.lock().unwrap().iter().any(|p| p == path)
    }
}

fn read_request_path(stream: &TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    // "GET /path HTTP/1.1"
    line.split_whitespace().nth(1).map(|p| p.to_string())
}

fn respond(stream: &mut TcpStream, path: &str, routes: &HashMap<String, Route>) {
    let route = routes.get(path).cloned().unwrap_or(Route {
        status: 404,
        body: b"not found".to_vec(),
    });
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
        route.status,
        if route.status == 200 { "OK" } else { "Error" },
        route.body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&route.body);
    let _ = stream.flush();
}

fn ok(body: Vec<u8>) -> Route {
    Route { status: 200, body }
}

fn status(code: u16) -> Route {
    Route {
        status: code,
        body: format!("<html>{code}</html>").into_bytes(),
    }
}

/// Build a zip archive with a single top-level directory `top` holding `files`.
fn zip_archive(top: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let opts: zip::write::SimpleFileOptions = Default::default();
        zip.add_directory(format!("{top}/"), opts).unwrap();
        for (path, content) in files {
            zip.start_file(format!("{top}/{path}"), opts).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
    }
    buf.into_inner()
}

/// Version record JSON the State API client understands (single record,
/// native platform, no distribution — the common non-java shape).
fn version_record(version: &str, url: &str) -> Vec<u8> {
    format!(
        r#"[{{"candidate":"tool","version":"{version}","platform":"LINUX_X64","url":"{url}","visible":true}}]"#
    )
    .into_bytes()
}

fn test_home() -> RsdkHome {
    let dir = env::temp_dir().join(format!("rsdk-download-test-{}", uuid::Uuid::new_v4()));
    RsdkHome::at(dir).expect("failed to create test RsdkHome")
}

/// Serializes tests that point the client at a fake API: the base URLs are
/// process-global environment variables.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` with both SDKMAN clients pointed at `api`.
fn with_fake_api<T>(api: &FakeApi, f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    env::set_var("RSDK_STATE_API_URL", api.url("/state"));
    env::set_var("RSDK_API_BASE_URL", api.url("/api"));
    let out = f();
    env::remove_var("RSDK_STATE_API_URL");
    env::remove_var("RSDK_API_BASE_URL");
    out
}

#[test]
fn vendor_url_failure_falls_back_to_the_broker() {
    let home = test_home();
    let archive = zip_archive("tool-1.0", &[("bin/tool", b"#!/bin/sh\n")]);

    let api = FakeApi::start(|base| {
        HashMap::from([
            (
                "state/versions/tool".to_string(),
                ok(version_record(
                    "1.0",
                    &format!("{base}/vendor/tool-1.0.zip"),
                )),
            ),
            ("vendor/tool-1.0.zip".to_string(), status(503)),
            (
                "api/broker/download/tool/1.0/linuxx64".to_string(),
                ok(archive),
            ),
        ])
    });

    let (installed, downloaded) = with_fake_api(&api, || {
        ToolVersion::install(&home, &"tool".to_string(), &Some("1.0".to_string()))
    })
    .expect("install should succeed via the broker after the vendor URL 503s");

    assert!(downloaded);
    assert!(installed.is_installed());
    assert!(installed.bin().join("tool").exists());

    assert!(
        api.requested("/vendor/tool-1.0.zip"),
        "vendor URL was not tried"
    );
    assert!(
        api.requested("/api/broker/download/tool/1.0/linuxx64"),
        "broker fallback was not tried"
    );
}

#[test]
fn install_succeeds_when_archive_has_no_bin_directory() {
    // jmc-style layout: the launcher lives in a subdirectory, so the
    // post-extract chmod of `<version>/bin` has nothing to work on.
    let home = test_home();
    let archive = zip_archive(
        "tool-1.0",
        &[("Azul Mission Control/bin/jmc", b"#!/bin/sh\n")],
    );

    let api = FakeApi::start(|base| {
        HashMap::from([
            (
                "state/versions/tool".to_string(),
                ok(version_record(
                    "1.0",
                    &format!("{base}/vendor/tool-1.0.zip"),
                )),
            ),
            ("vendor/tool-1.0.zip".to_string(), ok(archive)),
        ])
    });

    let (installed, _) = with_fake_api(&api, || {
        ToolVersion::install(&home, &"tool".to_string(), &Some("1.0".to_string()))
    })
    .expect("install must not fail just because the archive lacks a top-level bin/");

    assert!(installed.is_installed());
    assert!(installed
        .path()
        .join("Azul Mission Control/bin/jmc")
        .exists());
}
