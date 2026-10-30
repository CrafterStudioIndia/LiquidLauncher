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

use anyhow::{anyhow, bail};
use serde::Serialize;
use std::path::PathBuf;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tauri::{AppHandle, Emitter, Window};
use tokio::fs;
use tracing::{error, info};
use uuid::Uuid;

use super::failed;
use crate::app::instances::{self, Instance};
use crate::app::launch::{self, LoaderMod, VersionInfo};
use crate::app::modrinth::{self, CustomMod};
use crate::app::options::Options;
use crate::app::skins;
use crate::minecraft::auth::MinecraftAccount;
use crate::minecraft::{
    launcher::{LauncherData, StartParameter},
    prelauncher,
    progress::ProgressUpdate,
};
use crate::app::gui::{AppState, RunnerInstance, ShareableWindow};

/// The offline client id Minecraft uses for every launcher that does not sign in.
const CLIENT_ID: &str = "00000000402b5328";

/// Everything the main screen shows before a launch, all resolved locally or from Mojang and
/// Fabric's public metadata.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LaunchInfo {
    pub version: VersionInfo,
    pub selected: Instance,
    pub instances: Vec<Instance>,
    pub client_jar: String,
}

#[tauri::command]
pub(crate) async fn get_launch_info(
    app: AppHandle,
    options: Options,
) -> Result<LaunchInfo, String> {
    async {
        let data = options.start_options.data_directory();
        let selected = instances::selected(&data, &options.launcher_options.instance).await?;
        let version = launch::version_info(&app, &options).await?;
        let listed = instances::list(&data).await?;
        let client_jar = launch::client_jar(&app, &options).await?;

        anyhow::Ok(LaunchInfo {
            version,
            selected,
            instances: listed,
            client_jar: client_jar.to_string_lossy().into_owned(),
        })
    }
    .await
    .map_err(failed("read the local version"))
}

/// The mods in an instance's folder, and the Modrinth updates for them when `check` is set.
#[tauri::command]
pub(crate) async fn get_custom_mods(
    options: Options,
    check: bool,
) -> Result<Vec<CustomMod>, String> {
    async {
        let data = options.start_options.data_directory();
        let instance = instances::selected(&data, &options.launcher_options.instance).await?;
        let branch_options = options
            .version_options
            .options
            .get(&instance.id)
            .cloned()
            .unwrap_or_default();
        let installed = branch_options
            .modrinth_mods
            .get(launch::SUPPORTED_MC_VERSION)
            .map(Vec::as_slice)
            .unwrap_or_default();

        modrinth::custom_mods(
            &prelauncher::custom_mods_directory(
                &data,
                &instance.id,
                launch::SUPPORTED_MC_VERSION,
            ),
            installed,
            &branch_options.custom_mod_states,
            launch::SUPPORTED_MC_VERSION,
            launch::LOADER,
            check,
        )
        .await
    }
    .await
    .map_err(failed("read the mods"))
}

#[tauri::command]
pub(crate) async fn install_custom_mod(options: Options, path: PathBuf) -> Result<(), String> {
    async {
        if !path.is_file() {
            bail!("Selected file does not exist.");
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("That file has no valid name."))?;

        if !file_name.to_lowercase().ends_with(".jar") {
            bail!("Only .jar mod files can be installed.");
        }
        if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
            bail!("Invalid file name.");
        }

        let data = options.start_options.data_directory();
        let instance = instances::selected(&data, &options.launcher_options.instance).await?;
        let directory = prelauncher::custom_mods_directory(
            &data,
            &instance.id,
            launch::SUPPORTED_MC_VERSION,
        );

        fs::create_dir_all(&directory).await?;
        fs::copy(&path, directory.join(file_name)).await?;
        anyhow::Ok(())
    }
    .await
    .map_err(failed("install the mod"))
}

#[tauri::command]
pub(crate) async fn delete_custom_mod(options: Options, mod_name: String) -> Result<(), String> {
    async {
        if mod_name.contains('/') || mod_name.contains('\\') || mod_name.contains("..") {
            bail!("Invalid mod name.");
        }
        if !mod_name.to_lowercase().ends_with(".jar") {
            bail!("Mod file must end with .jar.");
        }

        let data = options.start_options.data_directory();
        let instance = instances::selected(&data, &options.launcher_options.instance).await?;
        let directory = prelauncher::custom_mods_directory(
            &data,
            &instance.id,
            launch::SUPPORTED_MC_VERSION,
        );
        let path = directory.join(&mod_name);

        // Only delete a file that really sits inside this instance's folder.
        if path.is_file() && path.parent() == Some(directory.as_path()) {
            fs::remove_file(path).await?;
        }
        anyhow::Ok(())
    }
    .await
    .map_err(failed("delete the mod"))
}

fn handle_stdout(window: &ShareableWindow, data: &[u8]) -> anyhow::Result<()> {
    let data = String::from_utf8(data.to_vec())?;
    if data.is_empty() {
        return Ok(()); // ignore empty lines
    }

    info!("{}", data.strip_suffix("\n").unwrap_or(&data));
    window
        .lock()
        .map_err(|_| anyhow!("Window lock is poisoned"))?
        .emit("process-output", data)?;
    Ok(())
}

fn handle_stderr(window: &ShareableWindow, data: &[u8]) -> anyhow::Result<()> {
    let data = String::from_utf8(data.to_vec())?;
    if data.is_empty() {
        return Ok(()); // ignore empty lines
    }

    error!("{}", data.strip_suffix("\n").unwrap_or(&data));
    window
        .lock()
        .map_err(|_| anyhow!("Window lock is poisoned"))?
        .emit("process-output", data)?;
    Ok(())
}

fn handle_progress(
    window: &ShareableWindow,
    progress_update: ProgressUpdate,
) -> anyhow::Result<()> {
    window
        .lock()
        .map_err(|_| anyhow!("Window lock is poisoned"))?
        .emit("progress-update", &progress_update)?;

    // Check if progress update is label update
    if let ProgressUpdate::SetLabel(label) = progress_update {
        handle_log(window, &label)?;
    }
    Ok(())
}

fn handle_log(window: &ShareableWindow, msg: &str) -> anyhow::Result<()> {
    info!("{}", msg);

    if let Ok(k) = window.lock() {
        let _ = k.emit("process-output", msg);
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn run_client(
    app: AppHandle,
    options: Options,
    mods: Vec<LoaderMod>,
    window: Window,
    app_state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // A shared mutex for the window object.
    let shareable_window: ShareableWindow = Arc::new(Mutex::new(window));
    let data = options.start_options.data_directory();

    let minecraft_account = options
        .start_options
        .minecraft_account
        .clone()
        .ok_or("no account selected")?;
    let (account_name, uuid, token, user_type) = match &minecraft_account {
        MinecraftAccount::OfflineAccount { name, id } => {
            let trimmed = name.trim();
            if trimmed.is_empty()
                || trimmed.len() > 16
                || !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return Err("Invalid Minecraft offline username.".to_string());
            }
            (
                trimmed.to_string(),
                id.to_string(),
                "-".to_string(),
                "legacy".to_string(),
            )
        }
    };

    // Random XUID
    let xuid = Uuid::new_v4().to_string();

    let runner_instance = &app_state.runner_instance;

    if runner_instance
        .lock()
        .map_err(|e| format!("unable to lock runner instance: {:?}", e))?
        .is_some()
    {
        return Err("client is already running".to_string());
    }

    let instance = instances::selected(&data, &options.launcher_options.instance)
        .await
        .map_err(|e| format!("unable to select the instance: {e:#}"))?;

    info!("Preparing LiquidBounce for instance {}...", instance.name);
    skins::sync_for_launch(&data, &instance.id, &account_name).await;
    let launch_manifest = launch::manifest(&app, &options, &instance)
        .await
        .map_err(|e| format!("failed to prepare the local build: {e:#}"))?;

    if let Err(error) = instances::touch(&data, &instance.id).await {
        error!("Failed to record the last launch: {:?}", error);
    }

    let (terminator_tx, terminator_rx) = tokio::sync::oneshot::channel();

    *runner_instance
        .lock()
        .map_err(|e| format!("unable to lock runner instance: {:?}", e))? = Some(RunnerInstance {
        terminator: terminator_tx,
    });

    let copy_of_runner_instance = runner_instance.clone();

    let parameters = StartParameter {
        java_distribution: options.start_options.java_distribution.clone(),
        jvm_args: options.start_options.jvm_args.clone().unwrap_or_default(),
        memory: options.start_options.memory,
        custom_data_path: if !options.start_options.custom_data_path.is_empty() {
            Some(options.start_options.custom_data_path.clone())
        } else {
            None
        },
        auth_player_name: account_name,
        auth_uuid: uuid,
        auth_access_token: token,
        auth_xuid: xuid,
        clientid: CLIENT_ID.to_string(),
        user_type,
        keep_launcher_open: options.launcher_options.keep_launcher_open,
        concurrent_downloads: options.launcher_options.concurrent_downloads,
        vanilla_integration: options.start_options.installation.clone(),
    };

    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let keep_launcher_open = parameters.keep_launcher_open;

                let launcher_data = LauncherData {
                    on_stdout: handle_stdout,
                    on_stderr: handle_stderr,
                    on_progress: handle_progress,
                    on_log: handle_log,
                    hide_window: |w| w.lock().unwrap().hide().unwrap(),
                    data: Box::new(shareable_window.clone()),
                    terminator: terminator_rx,
                };

                if let Err(e) =
                    prelauncher::launch(launch_manifest, parameters, mods, launcher_data).await
                {
                    if !keep_launcher_open {
                        shareable_window.lock().unwrap().show().unwrap();
                    }

                    let message = format!("An error occured:\n\n{:?}", e);
                    shareable_window
                        .lock()
                        .unwrap()
                        .emit("client-error", ())
                        .unwrap();
                    handle_stderr(&shareable_window, message.as_bytes()).unwrap();
                };

                *copy_of_runner_instance
                    .lock()
                    .map_err(|e| format!("unable to lock runner instance: {:?}", e))
                    .unwrap() = None;
                shareable_window
                    .lock()
                    .unwrap()
                    .emit("client-exited", ())
                    .unwrap()
            });
    });

    Ok(())
}

#[tauri::command]
pub(crate) async fn terminate(app_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut lck = app_state
        .runner_instance
        .lock()
        .map_err(|e| format!("unable to lock runner instance: {:?}", e))?;

    if let Some(inst) = lck.take() {
        info!("Sending sigterm");
        inst.terminator.send(()).unwrap();
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn check_jar_release_update(
    app: AppHandle,
    options: Options,
) -> Result<Option<crate::app::launch::ReleasePromptInfo>, String> {
    let data = options.start_options.data_directory();
    let destination = data.join("client").join("liquidbounce.jar");
    crate::app::launch::check_and_fetch_release_jar(&app, &destination)
        .await
        .map_err(|e| e.to_string())
}
