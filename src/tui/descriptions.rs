//! Parser for the SDKMAN "tools list" text into per-tool description lines.

use std::collections::HashMap;

pub(crate) fn parse_tool_descriptions(text: &str) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for entry in text.split("\n---") {
        let lines: Vec<&str> = entry.lines().collect();
        let mut install_line = None;
        let mut tool = None;
        for l in &lines {
            let trimmed = l.trim();
            if let Some(rest) = trimmed.strip_prefix("$ sdk install ") {
                install_line = Some(l);
                tool = Some(rest.trim().to_string());
                break;
            }
        }
        let Some(tool) = tool else { continue };
        let Some(install_idx) = install_line.and_then(|il| lines.iter().position(|&x| x == *il))
        else {
            continue;
        };

        let header_idx = lines
            .iter()
            .position(|l| !l.trim().is_empty() && !l.trim_start().starts_with("---"))
            .unwrap_or(0);
        let header = lines.get(header_idx).copied().unwrap_or("");

        let body: Vec<String> = lines[header_idx + 1..install_idx]
            .iter()
            .map(|l| l.trim_end().to_string())
            .collect();
        let body: Vec<String> = body
            .iter()
            .rev()
            .skip_while(|l| l.trim().is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let mut full = vec![header.trim().to_string()];
        full.extend(body);
        out.insert(tool, full);
    }
    out
}
