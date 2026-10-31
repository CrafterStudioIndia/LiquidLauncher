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
    "https://liquid-backend.vercel.app".to_string()
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
        let file_content = fs::read(app_data.join("options.json")).await?;
        let options = serde_json::from_slice::<Self>(&file_content)?;
        info!("Successfully loaded options from file");
        Ok(options)
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
