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

//! Instances: one game directory each, sharing the downloaded game and runtime.

use anyhow::Result;

use super::failed;
use crate::app::instances::{self, Instance};
use crate::app::options::Options;

#[tauri::command]
pub(crate) async fn list_instances(options: Options) -> Result<Vec<Instance>, String> {
    let data = options.start_options.data_directory();
    instances::list(&data)
        .await
        .map_err(failed("list the instances"))
}

#[tauri::command]
pub(crate) async fn create_instance(options: Options, name: String) -> Result<Instance, String> {
    let data = options.start_options.data_directory();
    instances::create(&data, &name)
        .await
        .map_err(failed("create the instance"))
}

#[tauri::command]
pub(crate) async fn rename_instance(
    options: Options,
    id: String,
    name: String,
) -> Result<Instance, String> {
    let data = options.start_options.data_directory();
    instances::rename(&data, &id, &name)
        .await
        .map_err(failed("rename the instance"))
}

#[tauri::command]
pub(crate) async fn delete_instance(options: Options, id: String) -> Result<Vec<Instance>, String> {
    let data = options.start_options.data_directory();
    async {
        instances::delete(&data, &id).await?;
        instances::list(&data).await
    }
    .await
    .map_err(failed("delete the instance"))
}
