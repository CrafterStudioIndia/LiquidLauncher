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

//! Tauri commands for offline skin management.

use std::path::{Path, PathBuf};

use anyhow::Context;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use super::failed;
use crate::app::options::Options;
use crate::app::skins::{self, StoredSkin};

#[tauri::command]
pub(crate) async fn list_skins(options: Options) -> Result<Vec<StoredSkin>, String> {
    let data = options.start_options.data_directory();
    // Ensure active skin is synced
    if let Ok(Some(active)) = skins::get_active().await {
        let skin_path = skins::sync_skin_to_instances(
            &active,
            &crate::LAUNCHER_DIRECTORY.config_dir().join("skins").join(&active.file_name),
            &data,
        )
        .await;
        let _ = skin_path;
    }
    skins::list().await.map_err(failed("list skins"))
}

#[tauri::command]
pub(crate) async fn get_active_skin() -> Result<Option<StoredSkin>, String> {
    skins::get_active().await.map_err(failed("get active skin"))
}

#[tauri::command]
pub(crate) async fn add_skin(
    options: Options,
    name: String,
    model: String,
    source_type: String,
    source: String,
    set_active: bool,
) -> Result<StoredSkin, String> {
    let data = options.start_options.data_directory();
    async {
        match source_type.as_str() {
            "file" => {
                let path = PathBuf::from(&source);
                skins::add_from_file(&name, &model, &path, set_active, &data).await
            }
            "mojang" => skins::add_from_mojang(&name, Some(&model), &source, set_active, &data).await,
            "base64" => {
                let clean = source.trim();
                let clean = if let Some(idx) = clean.find(',') {
                    &clean[idx + 1..]
                } else {
                    clean
                };
                let bytes = STANDARD.decode(clean).context("Invalid base64 skin data")?;
                skins::add_skin_bytes(&name, &model, bytes, set_active, &data).await
            }
            _ => anyhow::bail!("Invalid skin source type: {}", source_type),
        }
    }
    .await
    .map_err(failed("add skin"))
}

#[tauri::command]
pub(crate) async fn update_skin(
    options: Options,
    id: String,
    name: Option<String>,
    model: Option<String>,
) -> Result<StoredSkin, String> {
    let data = options.start_options.data_directory();
    skins::update_skin(&id, name.as_deref(), model.as_deref(), &data)
        .await
        .map_err(failed("update skin"))
}

#[tauri::command]
pub(crate) async fn set_active_skin(options: Options, id: String) -> Result<StoredSkin, String> {
    let data = options.start_options.data_directory();
    skins::set_active(&id, &data)
        .await
        .map_err(failed("set active skin"))
}

#[tauri::command]
pub(crate) async fn delete_skin(options: Options, id: String) -> Result<Vec<StoredSkin>, String> {
    let data = options.start_options.data_directory();
    skins::delete(&id, &data)
        .await
        .map_err(failed("delete skin"))
}

#[tauri::command]
pub(crate) async fn export_skin(id: String, path: String) -> Result<(), String> {
    let destination = Path::new(&path);
    skins::export(&id, destination)
        .await
        .map_err(failed("export skin"))
}

