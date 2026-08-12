//! Loading the tools and versions lists from the SDKMAN API and the local
//! install home.

use std::collections::HashSet;

use color_eyre::Result;

use rsdk::tool_version::ToolVersion;

use crate::tui::descriptions::parse_tool_descriptions;
use crate::tui::{filter_items, sort_items, Item};

use super::App;

impl App {
    pub(super) fn load_tools(&mut self) -> Result<()> {
        let names = self.sdkman.get_tools()?;
        let installed: HashSet<String> =
            self.rsdk_home.all_installed()?.map(|tv| tv.tool).collect();

        if let Ok(text) = self.sdkman.get_tools_list_text() {
            self.tool_descriptions = parse_tool_descriptions(&text);
        }

        self.tools = names
            .into_iter()
            .map(|n| {
                let starred = installed.contains(&n);
                Item::new(n, starred)
            })
            .collect();
        sort_items(&mut self.tools);
        self.tools_state.select(Some(0));
        self.update_tool_info();
        Ok(())
    }

    pub(super) fn update_tool_info(&mut self) {
        let Some(i) = self.tools_state.selected() else {
            self.tool_info.clear();
            return;
        };
        let items = filter_items(&self.tools, &self.search);
        let Some(tool) = items.get(i) else {
            self.tool_info.clear();
            return;
        };

        let installed: Vec<String> = self
            .rsdk_home
            .installed_versions(&tool.name)
            .map(|iter| iter.map(|tv| tv.version).collect())
            .unwrap_or_default();

        let mut lines: Vec<String> = Vec::new();
        if let Some(desc) = self.tool_descriptions.get(&tool.name) {
            lines.extend(desc.iter().cloned());
            lines.push(String::new());
            lines.push("─".repeat(60));
            lines.push(String::new());
        }
        if installed.is_empty() {
            lines.push("No versions installed".to_string());
        } else {
            lines.push("Installed versions:".to_string());
            for v in &installed {
                lines.push(format!("  • {v}"));
            }
        }
        self.tool_info = lines;
    }

    pub(super) fn load_versions(&mut self, tool: &str) -> Result<()> {
        let all = self.sdkman.get_tool_versions(tool)?;
        let installed: Vec<ToolVersion> =
            self.rsdk_home.installed_versions(tool)?.collect::<Vec<_>>();

        // Start from the API list, then append any installed versions that
        // are no longer advertised (e.g. older releases dropped upstream).
        let mut seen: HashSet<String> = all.iter().cloned().collect();
        let mut versions: Vec<String> = all;
        for tv in &installed {
            if !seen.contains(&tv.version) {
                seen.insert(tv.version.clone());
                versions.push(tv.version.clone());
            }
        }

        self.versions = versions
            .into_iter()
            .map(|v| {
                let installed_tv = installed.iter().find(|tv| tv.version == v);
                let starred = installed_tv.is_some();
                let mut item = Item::new(v, starred);
                if let Some(tv) = installed_tv {
                    item.is_current = tv.is_current();
                    item.is_default = tv.is_default();
                }
                item
            })
            .collect();
        sort_items(&mut self.versions);
        self.versions_state.select(Some(0));
        Ok(())
    }
}
