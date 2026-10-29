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

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tokio::fs;
use tracing::*;

use crate::app::gui::ShareableWindow;
use crate::app::launch::{LaunchManifest, LoaderMod, LoaderSubsystem, ModSource};
use crate::error::LauncherError;
use crate::minecraft::launcher;
use crate::minecraft::launcher::{LauncherData, StartParameter};
use crate::minecraft::progress::{
    get_max, get_progress, ProgressReceiver, ProgressUpdate, ProgressUpdateSteps,
};
use crate::minecraft::version::{VersionManifest, VersionProfile};
use crate::utils::{download_file, get_maven_artifact_path};
use crate::LAUNCHER_DIRECTORY;

use backon::{ExponentialBuilder, Retryable};

///
/// Prelaunching client
///
pub(crate) async fn launch(
    launch_manifest: LaunchManifest,
    launching_parameter: StartParameter,
    additional_mods: Vec<LoaderMod>,
    launcher_data: LauncherData<ShareableWindow>,
) -> Result<()> {
    // Starts the speed meter, kills it when the launch is completed.
    let _speed_meter = launcher_data.start_speed_meter();

    launcher_data.progress_update(ProgressUpdate::set_max());
    launcher_data.progress_update(ProgressUpdate::SetProgress(0));
    launcher_data.progress_update(ProgressUpdate::set_label("Loading version manifest..."));

    let mc_version_manifest = VersionManifest::fetch
        .retry(ExponentialBuilder::default())
        .notify(|err, dur| {
            launcher_data.log(&format!(
                "Failed to load version manifest. Retrying in {:?}. Error: {}",
                dur, err
            ));
        })
        .await?;

    let build = &launch_manifest.build;
    let subsystem = &launch_manifest.subsystem;

    let data_directory = launching_parameter
        .custom_data_path
        .clone()
        .map(|x| x.into())
        .unwrap_or_else(|| LAUNCHER_DIRECTORY.data_dir().to_path_buf());

    // Copy retrieve and copy mods from manifest
    clear_mods(&data_directory, &launch_manifest).await?;
    retrieve_and_copy_mods(
        &data_directory,
        &launch_manifest,
        &launch_manifest.mods,
        &launcher_data,
    )
    .await?;
    retrieve_and_copy_mods(
        &data_directory,
        &launch_manifest,
        &additional_mods,
        &launcher_data,
    )
    .await?;

    launcher_data.progress_update(ProgressUpdate::set_label("Loading version profile..."));
    let manifest_url = match subsystem {
        LoaderSubsystem::Fabric { manifest, .. } => manifest
            .replace("{MINECRAFT_VERSION}", &build.mc_version)
            .replace(
                "{FABRIC_LOADER_VERSION}",
                &build.subsystem_specific_data.fabric_loader_version,
            ),
        LoaderSubsystem::Forge { manifest, .. } => manifest.clone(),
    };
    let mut version = (|| async { VersionProfile::load(&manifest_url).await })
        .retry(ExponentialBuilder::default())
        .notify(|err, dur| {
            launcher_data.log(&format!(
                "Failed to load version profile: {}. Retrying in {:?}. Error: {}",
                manifest_url, dur, err
            ));
        })
        .await?;

    if let Some(inherited_version) = &version.inherits_from {
        let url = mc_version_manifest
            .versions
            .iter()
            .find(|x| &x.id == inherited_version)
            .map(|x| &x.url)
            .ok_or_else(|| {
                LauncherError::InvalidVersionProfile(format!(
                    "unable to find inherited version manifest {}",
                    inherited_version
                ))
            })?;

        debug!(
            "Determined {}'s download url to be {}",
            inherited_version, url
        );
        launcher_data.log(&format!(
            "Downloading inherited version {}...",
            inherited_version
        ));

        let parent_version = (|| async { VersionProfile::load(url).await })
            .retry(ExponentialBuilder::default())
            .notify(|err, dur| {
                launcher_data.log(&format!(
                    "Failed to load inherited version profile: {}. Retrying in {:?}. Error: {}",
                    inherited_version, dur, err
                ));
            })
            .await?;
        version.merge(parent_version)?;
    }
    version.apply_library_patches(&launch_manifest.library_patches);

    launcher_data.progress_update(ProgressUpdate::set_label(format!(
        "Launching {}...",
        launch_manifest.build.commit_id
    )));
    launcher::launch(
        &data_directory,
        launch_manifest,
        version,
        launching_parameter,
        launcher_data,
    )
    .await?;
    Ok(())
}

pub(crate) async fn clear_mods(data: &Path, manifest: &LaunchManifest) -> Result<()> {
    let mods_path = data
        .join("gameDir")
        .join(&manifest.build.branch)
        .join("mods");

    if !mods_path.exists() {
        return Ok(());
    }

    // Clear mods directory
    let mut mods_read = fs::read_dir(&mods_path).await?;
    while let Some(entry) = mods_read.next_entry().await? {
        if entry.file_type().await?.is_file() {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
    Ok(())
}

/// Where the custom mods of an instance and Minecraft version are kept.
pub fn custom_mods_directory(data: &Path, branch: &str, mc_version: &str) -> PathBuf {
    data.join("custom_mods").join(format!("{}-{}", branch, mc_version))
}

/// Copies a file into the instance's mods folder, preferring a hard link.
async fn place(source: &Path, destination: &Path, name: &str) -> Result<()> {
    if fs::hard_link(source, destination).await.is_ok() {
        return Ok(());
    }

    fs::copy(source, destination)
        .await
        .with_context(|| format!("Failed to copy mod {}", name))?;
    Ok(())
}

pub async fn retrieve_and_copy_mods(
    data: &Path,
    manifest: &LaunchManifest,
    mods: &Vec<LoaderMod>,
    launcher_data: &LauncherData<ShareableWindow>,
) -> Result<()> {
    let mod_cache_path = data.join("mod_cache");
    let mod_custom_path =
        custom_mods_directory(data, &manifest.build.branch, &manifest.build.mc_version);
    let mods_path = data
        .join("gameDir")
        .join(&manifest.build.branch)
        .join("mods");

    fs::create_dir_all(&mod_cache_path).await.with_context(|| {
        format!(
            "Failed to create mod cache directory {}",
            mod_cache_path.display()
        )
    })?;
    fs::create_dir_all(&mods_path)
        .await
        .with_context(|| format!("Failed to create mods directory {}", mods_path.display()))?;
    fs::create_dir_all(&mod_custom_path)
        .await
        .with_context(|| {
            format!(
                "Failed to create custom mods directory {}",
                mod_custom_path.display()
            )
        })?;

    // Download and copy mods
    let max = get_max(mods.len());

    for (mod_idx, current_mod) in mods.iter().enumerate() {
        // Skip mods that are not needed
        if !current_mod.required && !current_mod.enabled {
            continue;
        }

        let safe_mod_name = sanitize_filename::sanitize(&current_mod.name);
        let safe_mod_name = if safe_mod_name.is_empty() {
            format!("mod_{}", mod_idx)
        } else {
            safe_mod_name
        };

        launcher_data.progress_update(ProgressUpdate::set_label(format!(
            "Installing mod {}",
            current_mod.name
        )));

        match &current_mod.source {
            ModSource::Local { file_name } => {
                let path = mod_custom_path.join(file_name);

                // Check if local mod exists
                if !path.exists() {
                    error!("File of local mod {} does not exist", current_mod.name);
                    continue;
                }

                place(&path, &mods_path.join(format!("{}.jar", safe_mod_name)), &current_mod.name)
                    .await?;
                continue;
            }
            ModSource::Path { path, .. } => {
                let path = PathBuf::from(path);
                if !path.exists() {
                    bail!(
                        "File of mod {} does not exist: {}",
                        current_mod.name,
                        path.display()
                    );
                }

                place(&path, &mods_path.join(format!("{}.jar", safe_mod_name)), &current_mod.name)
                    .await?;
                continue;
            }
            ModSource::Repository { .. } => {}
        }

        let current_mod_path = mod_cache_path.join(current_mod.source.get_path()?);

        // Do we need to download the mod?
        if !current_mod_path.exists() {
            // Make sure that the parent directory exists
            fs::create_dir_all(&current_mod_path.parent().unwrap()).await?;

            let ModSource::Repository {
                repository,
                artifact,
            } = &current_mod.source
            else {
                unreachable!()
            };

            launcher_data.log(&format!("Downloading mod {} from {}", artifact, repository));
            let repository_url = manifest.repositories.get(repository).ok_or_else(|| {
                LauncherError::InvalidVersionProfile(format!(
                    "There is no repository specified with the name {}",
                    repository
                ))
            })?;

            let retrieved_bytes = download_file(
                &format!("{}{}", repository_url, get_maven_artifact_path(artifact)?),
                |a, b| {
                    launcher_data.progress_update(ProgressUpdate::set_for_step(
                        ProgressUpdateSteps::DownloadLiquidBounceMods,
                        get_progress(mod_idx, a, b),
                        max,
                    ));
                },
            )
            .await?;

            fs::write(&current_mod_path, retrieved_bytes)
                .await
                .with_context(|| format!("Failed to write mod {}", current_mod.name))?;
        }

        // Copy the mod.
        place(
            &current_mod_path,
            &mods_path.join(format!("{}.jar", safe_mod_name)),
            &current_mod.name,
        )
        .await?;
    }

    Ok(())
}
