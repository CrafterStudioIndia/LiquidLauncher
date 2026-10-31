/*
 * This file is part of LiquidLauncher (https://github.com/CCBlueX/LiquidLauncher)
 *
 * Copyright (c) 2015 - 2024 CCBlueX
 *
 * LiquidLauncher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * LiquidLauncher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with LiquidLauncher. If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::app::modrinth::ModrinthMod;
use crate::minecraft::auth::MinecraftAccount;
use crate::minecraft::java::DistributionSelection;
use crate::LAUNCHER_DIRECTORY;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub(crate) struct Options {
    #[serde(rename = "start")]
    pub start_options: StartOptions,
    #[serde(rename = "version", default)]
    pub version_options: VersionOptions,
    #[serde(rename = "launcher")]
    pub launcher_options: LauncherOptions,
    #[serde(rename = "appearance", default)]
    pub appearance_options: AppearanceOptions,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StartOptions {
    #[serde(rename = "account")]
    pub minecraft_account: Option<MinecraftAccount>,
    #[serde(rename = "accounts", default)]
    pub accounts: Vec<MinecraftAccount>,
    #[serde(rename = "customDataPath", default)]
    pub custom_data_path: String,
    #[serde(rename = "javaDistribution", default)]
    pub java_distribution: DistributionSelection,
    #[serde(rename = "jvmArgs", default)]
    pub jvm_args: Option<Vec<String>>,
    #[serde(rename = "memory", default = "default_memory")]
    pub memory: u64,
    #[serde(rename = "installation", default)]
    pub installation: MinecraftInstallationOptions,
    /// A LiquidBounce jar selected by the user. Empty means the bundled one.
    #[serde(rename = "clientJar", default)]
    pub client_jar: String,
    #[serde(rename = "backendUrl", default = "default_backend_url")]
    pub backend_url: String,
}

fn default_backend_url() -> String {
    std::env::var("LIQUID_BACKEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MinecraftInstallationOptions {
    #[serde(rename = "customPath", default)]
    pub custom_path: String,
    #[serde(rename = "useVanillaSaves", default)]
    pub use_vanilla_saves: bool,
    #[serde(rename = "useVanillaResourcePacks", default)]
    pub use_vanilla_resource_packs: bool,
    #[serde(rename = "useVanillaShaderPacks", default)]
    pub use_vanilla_shader_packs: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct VersionOptions {
    #[serde(rename = "options", default)]
    pub options: HashMap<String, BranchOptions>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LauncherOptions {
    #[serde(rename = "firstRun", default)]
    pub first_run: bool,
    #[serde(rename = "concurrentDownloads", default = "default_downloads")]
    pub concurrent_downloads: u32,
    #[serde(rename = "keepLauncherOpen", default)]
    pub keep_launcher_open: bool,
    /// The id of the selected instance.
    #[serde(rename = "instance", default)]
    pub instance: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppearanceOptions {
    /// The tint drawn over the background picture, as `#rrggbb`.
    #[serde(default = "default_background_color")]
    pub background_color: String,
    /// How much of the tint covers the picture, from 0 to 1.
    #[serde(default = "default_background_opacity")]
    pub background_opacity: f32,
    /// Blur radius of the background picture in pixels.
    #[serde(default)]
    pub background_blur: f32,
}

impl Default for AppearanceOptions {
    fn default() -> Self {
        Self {
            background_color: default_background_color(),
            background_opacity: default_background_opacity(),
            background_blur: 0.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct BranchOptions {
    #[serde(rename = "modStates", default)]
    pub mod_states: HashMap<String, bool>,
    #[serde(rename = "customModStates", default)]
    pub custom_mod_states: HashMap<String, bool>,
    /// By Minecraft version.
    #[serde(rename = "modrinthMods", default)]
    pub modrinth_mods: HashMap<String, Vec<ModrinthMod>>,
}

impl Options {
    pub async fn load(app_data: &Path) -> Result<Self> {
        let options_path = app_data.join("options.json");
        let (mut options, needs_save) = match fs::read(&options_path).await {
            Ok(file_content) => match serde_json::from_slice::<Self>(&file_content) {
                Ok(mut opt) => {
                    info!("Successfully loaded options from file");
                    let migrated = opt.fetch_legacy_accounts(app_data).await;
                    (opt, migrated)
                }
                Err(err) => {
                    tracing::warn!("Failed to deserialize options.json ({err}), recovering from old update...");
                    let mut opt = Self::recover_from_legacy_paths(app_data).await;
                    let _ = opt.fetch_legacy_accounts(app_data).await;
                    (opt, true)
                }
            },
            Err(_) => {
                info!("No options.json found at {options_path:?}, fetching user accounts from old update...");
                let mut opt = Self::recover_from_legacy_paths(app_data).await;
                let _ = opt.fetch_legacy_accounts(app_data).await;
                (opt, true)
            }
        };

        options.ensure_account_consistency();

        if needs_save {
            if let Err(e) = options.store(app_data).await {
                tracing::warn!("Failed to store migrated options: {e:?}");
            }
        }

        options.sync_accounts_to_backend();

        Ok(options)
    }

    pub fn ensure_account_consistency(&mut self) {
        if let Some(ref current) = self.start_options.minecraft_account {
            let has_it = self.start_options.accounts.iter().any(|a| a.get_username().eq_ignore_ascii_case(current.get_username()));
            if !has_it {
                self.start_options.accounts.insert(0, current.clone());
            }
        } else if let Some(first) = self.start_options.accounts.first() {
            self.start_options.minecraft_account = Some(first.clone());
        }
    }

    /// Recovers entire options struct from legacy paths if options.json does not exist yet in app_data
    pub async fn recover_from_legacy_paths(current_config_dir: &Path) -> Self {
        let mut candidates = Vec::new();

        // 1. Check data directory (where older versions stored options.json)
        candidates.push(LAUNCHER_DIRECTORY.data_dir().join("options.json"));

        // 2. Standard platform directories
        if let Some(cfg) = dirs::config_dir() {
            candidates.push(cfg.join("LiquidLauncher").join("options.json"));
            candidates.push(cfg.join("liquidlauncher").join("options.json"));
            candidates.push(cfg.join("CCBlueX").join("LiquidLauncher").join("options.json"));
        }
        if let Some(data) = dirs::data_dir() {
            candidates.push(data.join("LiquidLauncher").join("options.json"));
            candidates.push(data.join("liquidlauncher").join("options.json"));
            candidates.push(data.join("CCBlueX").join("LiquidLauncher").join("options.json"));
        }
        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join(".liquidlauncher").join("options.json"));
        }

        for path in candidates {
            if path == current_config_dir.join("options.json") {
                continue;
            }
            if let Ok(bytes) = fs::read(&path).await {
                if let Ok(recovered) = serde_json::from_slice::<Self>(&bytes) {
                    info!("Successfully recovered previous launcher options from old update at {:?}", path);
                    return recovered;
                }
            }
        }

        Self::default()
    }

    /// Fetches and migrates user accounts from older updates, legacy directories, or launcher accounts
    pub async fn fetch_legacy_accounts(&mut self, current_config_dir: &Path) -> bool {
        let mut added_any = false;
        let mut candidates = Vec::new();

        // 1. Data directory
        candidates.push(LAUNCHER_DIRECTORY.data_dir().join("options.json"));

        // 2. Config & Data candidate dirs
        if let Some(cfg) = dirs::config_dir() {
            candidates.push(cfg.join("LiquidLauncher").join("options.json"));
            candidates.push(cfg.join("liquidlauncher").join("options.json"));
            candidates.push(cfg.join("CCBlueX").join("LiquidLauncher").join("options.json"));
            candidates.push(cfg.join("LiquidLauncher").join("accounts.json"));
        }
        if let Some(data) = dirs::data_dir() {
            candidates.push(data.join("LiquidLauncher").join("options.json"));
            candidates.push(data.join("liquidlauncher").join("options.json"));
            candidates.push(data.join("CCBlueX").join("LiquidLauncher").join("options.json"));
        }
        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join(".liquidlauncher").join("options.json"));
            candidates.push(home.join(".liquidlauncher").join("accounts.json"));
            candidates.push(home.join(".minecraft").join("launcher_accounts.json"));
            candidates.push(home.join(".minecraft").join("launcher_profiles.json"));
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let p = PathBuf::from(appdata);
                candidates.push(p.join(".minecraft").join("launcher_accounts.json"));
                candidates.push(p.join(".liquidlauncher").join("options.json"));
                candidates.push(p.join("LiquidLauncher").join("options.json"));
            }
        }

        for path in candidates {
            if path == current_config_dir.join("options.json") {
                continue;
            }
            if !path.exists() {
                continue;
            }

            if let Ok(bytes) = fs::read(&path).await {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    // Try parsing accounts from options.json schema
                    if let Some(start) = val.get("start") {
                        if let Some(accs) = start.get("accounts").and_then(|a| a.as_array()) {
                            for item in accs {
                                if let Ok(acc) = serde_json::from_value::<MinecraftAccount>(item.clone()) {
                                    if self.add_account_if_missing(acc) {
                                        added_any = true;
                                    }
                                } else if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                    if let Ok(acc) = MinecraftAccount::offline(name) {
                                        if self.add_account_if_missing(acc) {
                                            added_any = true;
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(item) = start.get("account") {
                            if let Ok(acc) = serde_json::from_value::<MinecraftAccount>(item.clone()) {
                                if self.add_account_if_missing(acc) {
                                    added_any = true;
                                }
                            } else if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                if let Ok(acc) = MinecraftAccount::offline(name) {
                                    if self.add_account_if_missing(acc) {
                                        added_any = true;
                                    }
                                }
                            }
                        }
                    }

                    // Try parsing top-level account in older version
                    if let Some(item) = val.get("account") {
                        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                            if let Ok(acc) = MinecraftAccount::offline(name) {
                                if self.add_account_if_missing(acc) {
                                    added_any = true;
                                }
                            }
                        }
                    }

                    // Try parsing vanilla launcher_accounts.json
                    if let Some(accounts_map) = val.get("accounts").and_then(|a| a.as_object()) {
                        for (_key, obj) in accounts_map {
                            if let Some(profile) = obj.get("minecraftProfile") {
                                if let Some(name) = profile.get("name").and_then(|n| n.as_str()) {
                                    if let Ok(acc) = MinecraftAccount::offline(name) {
                                        if self.add_account_if_missing(acc) {
                                            added_any = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if added_any {
            info!("Successfully fetched and restored user accounts from previous launcher updates/data");
        }
        added_any
    }

    fn add_account_if_missing(&mut self, acc: MinecraftAccount) -> bool {
        let name = acc.get_username();
        if self.start_options.accounts.iter().any(|a| a.get_username().eq_ignore_ascii_case(name)) {
            return false;
        }
        self.start_options.accounts.push(acc);
        true
    }

    pub fn sync_accounts_to_backend(&self) {
        let backend_url = self.start_options.backend_url.clone();
        let accounts = self.start_options.accounts.clone();
        if accounts.is_empty() {
            return;
        }

        tokio::spawn(async move {
            let base = backend_url.trim_end_matches('/');
            let reg_post_url = format!("{}/api/register", base);
            for acc in accounts {
                let name = acc.get_username().to_string();
                let uuid = match acc {
                    MinecraftAccount::OfflineAccount { id, .. } => id.to_string(),
                };
                let reg_query_url = format!("{}/query.php?uuid={}&name={}", base, uuid, name);
                let body = serde_json::json!({
                    "username": name,
                    "uuid": uuid,
                    "accountType": "offline"
                });
                let _ = crate::HTTP_CLIENT.post(&reg_post_url).json(&body).send().await;
                let _ = crate::HTTP_CLIENT.get(&reg_query_url).send().await;
            }
        });
    }

    pub async fn store(&self, app_data: &Path) -> Result<()> {
        // store the options in the file
        fs::write(app_data.join("options.json"), serde_json::to_string(&self)?).await?;
        Ok(())
    }
}

impl StartOptions {
    /// The custom data path if one is set, otherwise the launcher's data directory.
    pub fn data_directory(&self) -> PathBuf {
        match self.custom_data_path.as_str() {
            "" => LAUNCHER_DIRECTORY.data_dir().to_path_buf(),
            path => PathBuf::from(path),
        }
    }
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            minecraft_account: None,
            accounts: Vec::new(),
            java_distribution: DistributionSelection::default(),
            custom_data_path: String::new(),
            jvm_args: None,
            memory: default_memory(),
            installation: MinecraftInstallationOptions::default(),
            client_jar: String::new(),
            backend_url: default_backend_url(),
        }
    }
}

impl Default for LauncherOptions {
    fn default() -> Self {
        Self {
            first_run: true,
            keep_launcher_open: false,
            concurrent_downloads: default_downloads(),
            instance: String::new(),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            start_options: StartOptions::default(),
            version_options: VersionOptions::default(),
            launcher_options: LauncherOptions::default(),
            appearance_options: AppearanceOptions::default(),
        }
    }
}

fn default_memory() -> u64 {
    4096
}

fn default_downloads() -> u32 {
    10
}

fn default_background_color() -> String {
    "#0b1020".to_owned()
}

fn default_background_opacity() -> f32 {
    0.55
}
