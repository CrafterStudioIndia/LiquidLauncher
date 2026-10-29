/*
 * This file is part of LiquidLauncher (https://github.com/CCBlueX/LiquidLauncher)
 *
 * Copyright (c) 2015 - 2026 CCBlueX
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

//! The local launch manifest.
//!
//! LiquidBounce itself ships with the launcher. Minecraft and the Fabric loader come from
//! Mojang's and Fabric's own metadata servers; Fabric API comes from Modrinth. No CCBlueX
//! service is contacted at runtime.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use async_zip::tokio::read::fs::ZipFileReader;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tracing::{info, warn};

use crate::app::instances::Instance;
use crate::app::modrinth::{self, ModrinthMod};
use crate::app::options::Options;
use crate::minecraft::java::JavaDistribution;
use crate::minecraft::version::LibraryPatch;
use crate::{HTTP_CLIENT, LAUNCHER_DIRECTORY};

/// The only Minecraft version this build of LiquidBounce supports.
pub const SUPPORTED_MC_VERSION: &str = "1.21.11";
pub const LOADER: &str = "fabric";
pub const MODRINTH_MAVEN: &str = "https://api.modrinth.com/maven/";

const FABRIC_LOADER_ENDPOINT: &str = "https://meta.fabricmc.net/v2/versions/loader/1.21.11";
const FABRIC_PROFILE: &str =
    "https://meta.fabricmc.net/v2/versions/loader/{MINECRAFT_VERSION}/{FABRIC_LOADER_VERSION}/profile/json";
const LOADER_CACHE_MAX_AGE: u64 = 6 * 60 * 60;
const CLIENT_JAR_NAME: &str = "liquidbounce.jar";

#[derive(Serialize, Deserialize, Clone)]
pub struct Build {
    #[serde(rename(serialize = "buildId"))]
    pub build_id: u32,
    #[serde(rename(serialize = "commitId"))]
    pub commit_id: String,
    pub branch: String,
    pub subsystem: String,
    #[serde(rename(serialize = "lbVersion"))]
    pub lb_version: String,
    #[serde(rename(serialize = "mcVersion"))]
    pub mc_version: String,
    pub release: bool,
    pub date: DateTime<Utc>,
    pub message: String,
    pub url: String,
    #[serde(rename(serialize = "jreDistribution"), default)]
    pub jre_distribution: JavaDistribution,
    #[serde(rename(serialize = "jreVersion"))]
    pub jre_version: u32,
    #[serde(flatten)]
    pub subsystem_specific_data: SubsystemSpecificData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubsystemSpecificData {
    #[serde(rename(serialize = "fabricApiVersion"))]
    pub fabric_api_version: String,
    #[serde(rename(serialize = "fabricLoaderVersion"))]
    pub fabric_loader_version: String,
    #[serde(rename(serialize = "kotlinVersion"))]
    pub kotlin_version: String,
    #[serde(rename(serialize = "kotlinModVersion"))]
    pub kotlin_mod_version: String,
}

#[derive(Deserialize)]
pub struct LaunchManifest {
    pub build: Build,
    pub subsystem: LoaderSubsystem,
    pub mods: Vec<LoaderMod>,
    pub repositories: BTreeMap<String, String>,
    #[serde(default)]
    pub library_patches: Vec<LibraryPatch>,
}

#[derive(Deserialize)]
#[serde(tag = "name")]
pub enum LoaderSubsystem {
    #[serde(rename = "fabric")]
    Fabric {
        manifest: String,
        mod_directory: String,
    },
    #[serde(rename = "forge")]
    Forge {
        manifest: String,
        mod_directory: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderMod {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub enabled: bool,
    pub name: String,
    pub source: ModSource,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum ModSource {
    /// A file in `custom_mods/<instance>-<minecraft version>`.
    #[serde(rename = "local")]
    #[serde(rename_all = "camelCase")]
    Local { file_name: String },

    /// A file that the launcher keeps itself, referenced by its absolute path.
    #[serde(rename = "path")]
    #[serde(rename_all = "camelCase")]
    Path { path: String, file_name: String },

    /// A file in the Maven repository named by the launch manifest.
    #[serde(rename = "repository")]
    #[serde(rename_all = "camelCase")]
    Repository { repository: String, artifact: String },
}

impl ModSource {
    pub fn get_path(&self) -> Result<String> {
        Ok(match self {
            ModSource::Local { file_name } => file_name.clone(),
            ModSource::Path { file_name, .. } => file_name.clone(),
            ModSource::Repository { artifact, .. } => {
                crate::utils::get_maven_artifact_path(artifact)?
            }
        })
    }
}

/// What the launcher shows and resolves before a launch.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub minecraft: String,
    pub loader: String,
    pub fabric_api: String,
    pub liquidbounce: String,
    pub java: u32,
}

#[derive(Serialize, Deserialize)]
struct CachedLoader {
    version: String,
    fetched_at: u64,
}

#[derive(Deserialize)]
struct FabricLoaderEntry {
    loader: FabricLoader,
}

#[derive(Deserialize)]
struct FabricLoader {
    version: String,
    stable: bool,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn cache_file() -> PathBuf {
    LAUNCHER_DIRECTORY
        .config_dir()
        .join("cache")
        .join(format!("fabric-loader-{SUPPORTED_MC_VERSION}.json"))
}

fn api_cache_file() -> PathBuf {
    LAUNCHER_DIRECTORY
        .config_dir()
        .join("cache")
        .join(format!("fabric-api-{SUPPORTED_MC_VERSION}.json"))
}

/// A resolved Fabric API build, kept so the launcher still works while Modrinth is unreachable.
#[derive(Serialize, Deserialize)]
struct CachedApi {
    #[serde(flatten)]
    module: ModrinthMod,
    fetched_at: u64,
}

/// The newest stable Fabric loader for the supported Minecraft version, cached for a few hours.
pub async fn fabric_loader_version() -> Result<String> {
    let cache = cache_file();

    if let Ok(raw) = fs::read_to_string(&cache).await {
        if let Ok(cached) = serde_json::from_str::<CachedLoader>(&raw) {
            if now().saturating_sub(cached.fetched_at) < LOADER_CACHE_MAX_AGE {
                return Ok(cached.version);
            }
        }
    }

    let fetched = async {
        let entries: Vec<FabricLoaderEntry> = HTTP_CLIENT
            .get(FABRIC_LOADER_ENDPOINT)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        entries
            .iter()
            .find(|entry| entry.loader.stable)
            .or_else(|| entries.first())
            .map(|entry| entry.loader.version.clone())
            .context("Fabric offers no loader for Minecraft 1.21.11")
    }
    .await;

    match fetched {
        Ok(version) => {
            if let Some(parent) = cache.parent() {
                let _ = fs::create_dir_all(parent).await;
            }
            let cached = CachedLoader {
                version: version.clone(),
                fetched_at: now(),
            };
            if let Ok(raw) = serde_json::to_string(&cached) {
                let _ = fs::write(&cache, raw).await;
            }
            info!("Resolved Fabric loader {}", version);
            Ok(version)
        }
        Err(error) => {
            // A stale cache beats refusing to launch while Fabric's meta server is down.
            if let Ok(raw) = fs::read_to_string(&cache).await {
                if let Ok(cached) = serde_json::from_str::<CachedLoader>(&raw) {
                    warn!(
                        "Fabric meta is unreachable ({}), using cached loader {}",
                        error, cached.version
                    );
                    return Ok(cached.version);
                }
            }
            Err(error)
        }
    }
}

/// The newest Fabric API build for the supported Minecraft version, cached for a few hours.
pub async fn fabric_api() -> Result<ModrinthMod> {
    let cache = api_cache_file();

    if let Ok(raw) = fs::read_to_string(&cache).await {
        if let Ok(cached) = serde_json::from_str::<CachedApi>(&raw) {
            if now().saturating_sub(cached.fetched_at) < LOADER_CACHE_MAX_AGE {
                return Ok(cached.module);
            }
        }
    }

    match modrinth::newest("fabric-api", SUPPORTED_MC_VERSION, LOADER).await {
        Ok(api) => {
            if let Some(parent) = cache.parent() {
                let _ = fs::create_dir_all(parent).await;
            }
            let cached = CachedApi {
                module: api.clone(),
                fetched_at: now(),
            };
            if let Ok(raw) = serde_json::to_string(&cached) {
                let _ = fs::write(&cache, raw).await;
            }
            info!("Resolved Fabric API {}", api.version);
            Ok(api)
        }
        Err(error) => {
            // A stale cache beats refusing to launch while Modrinth is down.
            if let Ok(raw) = fs::read_to_string(&cache).await {
                if let Ok(cached) = serde_json::from_str::<CachedApi>(&raw) {
                    warn!(
                        "Modrinth is unreachable ({}), using cached Fabric API {}",
                        error, cached.module.version
                    );
                    return Ok(cached.module);
                }
            }
            Err(error).context("Unable to resolve Fabric API for Minecraft 1.21.11")
        }
    }
}

/// The LiquidBounce jar to launch: the one selected in the settings, or the bundled one.
pub(crate) async fn client_jar(app: &AppHandle, options: &Options) -> Result<PathBuf> {
    let custom = options.start_options.client_jar.trim();
    if !custom.is_empty() {
        let path = PathBuf::from(custom);
        if path.is_file() {
            return Ok(path);
        }
        bail!("The selected LiquidBounce jar does not exist: {}", path.display());
    }

    let data = options.start_options.data_directory();
    let directory = data.join("client");
    let destination = directory.join(CLIENT_JAR_NAME);
    let source = bundled_client_jar(app).ok();

    let up_to_date = match (&source, fs::metadata(&destination).await) {
        (Some(source), Ok(installed)) => match fs::metadata(source).await {
            Ok(bundled) => {
                let size_eq = bundled.len() == installed.len();
                let time_ok = match (bundled.modified(), installed.modified()) {
                    (Ok(b), Ok(i)) => i >= b,
                    _ => true,
                };
                size_eq && time_ok
            }
            Err(_) => true,
        },
        _ => true,
    };
    if destination.is_file() && up_to_date {
        return Ok(destination);
    }

    fs::create_dir_all(&directory)
        .await
        .with_context(|| format!("Failed to create {}", directory.display()))?;

    if let Some(source) = source {
        info!(
            "Installing bundled LiquidBounce jar from {}",
            source.display()
        );
        copy_replacing(&source, &destination)
            .await
            .with_context(|| format!("Failed to copy {}", source.display()))?;
        return Ok(destination);
    }

    // If no local bundled jar was found, attempt downloading from GitHub releases
    if !destination.is_file() {
        info!("No local LiquidBounce jar found. Attempting download from GitHub release...");
        match download_client_from_github(&destination).await {
            Ok(()) => return Ok(destination),
            Err(e) => {
                warn!("GitHub release mod download failed: {e}");
            }
        }
    } else {
        return Ok(destination);
    }

    bail!(
        "The LiquidBounce mod jar is missing. Select one in Settings → General, \
         or publish a release to GitHub with liquidbounce-0.37.0.jar."
    );
}

const DEFAULT_CLIENT_JAR_URL: &str =
    "https://github.com/CrafterStudioIndia/LiquidBounce/releases/download/v1.0.0/liquidbounce-0.37.0.jar";

pub async fn download_client_from_github(destination: &Path) -> Result<()> {
    let url = std::env::var("LIQUID_CLIENT_JAR_URL")
        .unwrap_or_else(|_| DEFAULT_CLIENT_JAR_URL.to_string());
    info!("Downloading LiquidBounce jar from {url} to {}", destination.display());
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).await?;
    }
    let temp_dest = destination.with_extension("jar.download.tmp");
    crate::utils::download_file_untracked(&url, &temp_dest).await?;
    fs::rename(&temp_dest, destination).await?;
    Ok(())
}

fn bundled_client_jar(app: &AppHandle) -> Result<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let candidates = [
        app.path()
            .resolve(CLIENT_JAR_NAME, BaseDirectory::Resource)
            .ok(),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join(CLIENT_JAR_NAME)),
        // Running the built mod straight out of the LiquidBounce repository build.
        Some(repo_root.join("LiquidBounce").join("build").join("libs").join("liquidbounce-0.37.0.jar")),
        // Bundled release jar at repository root.
        Some(repo_root.join("liquidbounce-0.37.0-ryoku.jar")),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|path| path.is_file())
        .context(
            "The bundled LiquidBounce jar is missing. Select one in Settings → Instances \
             or place liquidbounce.jar next to the launcher.",
        )
}

async fn copy_replacing(source: &Path, destination: &Path) -> Result<()> {
    let temporary = destination.with_extension("jar.tmp");
    fs::copy(source, &temporary).await?;
    fs::rename(&temporary, destination).await?;
    Ok(())
}

/// The version declared by a LiquidBounce jar's `fabric.mod.json`.
pub async fn liquidbounce_version(jar: &Path) -> Option<String> {
    let zip = ZipFileReader::new(jar).await.ok()?;
    let index = zip.file().entries().iter().position(|entry| {
        entry
            .filename()
            .as_str()
            .is_ok_and(|name| name == "fabric.mod.json")
    })?;
    let mut json = String::new();
    zip.reader_with_entry(index)
        .await
        .ok()?
        .read_to_string_checked(&mut json)
        .await
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(&json).ok()?;
    value.get("version")?.as_str().map(ToOwned::to_owned)
}

pub(crate) async fn version_info(app: &AppHandle, options: &Options) -> Result<VersionInfo> {
    let jar = client_jar(app, options).await?;
    let (loader, api, liquidbounce) = tokio::join!(
        fabric_loader_version(),
        fabric_api(),
        liquidbounce_version(&jar)
    );

    Ok(VersionInfo {
        minecraft: SUPPORTED_MC_VERSION.to_owned(),
        loader: loader?,
        fabric_api: api?.version,
        liquidbounce: liquidbounce.unwrap_or_else(|| "unknown".to_owned()),
        java: 21,
    })
}

/// Builds the launch manifest for an instance from the local jar and the public version metadata.
pub(crate) async fn manifest(
    app: &AppHandle,
    options: &Options,
    instance: &Instance,
) -> Result<LaunchManifest> {
    let jar = client_jar(app, options).await?;
    let (loader, api) = tokio::try_join!(fabric_loader_version(), fabric_api())?;
    let lb_version = liquidbounce_version(&jar)
        .await
        .unwrap_or_else(|| "unknown".to_owned());

    Ok(LaunchManifest {
        build: Build {
            build_id: 0,
            commit_id: instance.id.clone(),
            branch: instance.id.clone(),
            subsystem: LOADER.to_owned(),
            lb_version,
            mc_version: SUPPORTED_MC_VERSION.to_owned(),
            release: true,
            date: Utc::now(),
            message: format!("Local build for {}", instance.name),
            url: String::new(),
            jre_distribution: JavaDistribution::Temurin,
            jre_version: 21,
            subsystem_specific_data: SubsystemSpecificData {
                fabric_api_version: api.version.clone(),
                fabric_loader_version: loader,
                kotlin_version: String::new(),
                kotlin_mod_version: String::new(),
            },
        },
        subsystem: LoaderSubsystem::Fabric {
            manifest: FABRIC_PROFILE.to_owned(),
            mod_directory: "mods".to_owned(),
        },
        mods: vec![
            LoaderMod {
                required: true,
                enabled: true,
                name: "LiquidBounce".to_owned(),
                source: ModSource::Path {
                    path: jar.to_string_lossy().into_owned(),
                    file_name: CLIENT_JAR_NAME.to_owned(),
                },
            },
            LoaderMod {
                required: true,
                enabled: true,
                name: "Fabric API".to_owned(),
                source: ModSource::Repository {
                    repository: "modrinth".to_owned(),
                    artifact: format!("maven.modrinth:fabric-api:{}", api.version_id),
                },
            },
        ],
        repositories: BTreeMap::from([("modrinth".to_owned(), MODRINTH_MAVEN.to_owned())]),
        library_patches: vec![],
    })
}
