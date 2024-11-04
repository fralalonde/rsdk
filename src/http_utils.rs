use std::env;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;

pub fn extract_filename_from_disposition(content_disposition: &str) -> Option<&str> {
    content_disposition
        .split(';')
        .find_map(|part| part.trim().strip_prefix("filename="))
        .map(|filename| filename.trim_matches('"'))
}

pub fn initialize_progress_bar(total_size: u64, file_name: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.green}} {file_name} {{bar:40.cyan/blue}} {{bytes}}/{{total_bytes}} ({{eta}})"
            ))
            .expect("Failed to set progress bar template")
            .progress_chars("#>-"),
    );
    pb
}

pub fn read_proxy_from_env() -> Option<reqwest::Proxy> {
    if let Ok(http_proxy) = env::var("http_proxy") {
        debug!("Using HTTP proxy {http_proxy}");
        return reqwest::Proxy::http(&http_proxy).ok();
    }
    if let Ok(https_proxy) = env::var("https_proxy") {
        debug!("Using HTTPS proxy {https_proxy}");
        return reqwest::Proxy::https(&https_proxy).ok();
    }
    None
}