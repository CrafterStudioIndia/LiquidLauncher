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

use tracing::warn;

use super::{failed, search_query};
use crate::app::instances;
use crate::app::launch::{self, LoaderMod, ModSource};
use crate::app::modrinth::{self, Held, ModrinthMod, Search};
use crate::app::options::Options;
use crate::minecraft::prelauncher;

/// Mods Modrinth has for Minecraft 1.21.11 and Fabric, and whether each is installed in the
/// selected instance.
#[tauri::command]
pub(crate) async fn modrinth_search(
    options: Options,
    query: Option<String>,
) -> Result<Search, String> {
    async {
        let data = options.start_options.data_directory();
        let instance = instances::selected(&data, &options.launcher_options.instance).await?;
        let directory = prelauncher::custom_mods_directory(
            &data,
            &instance.id,
            launch::SUPPORTED_MC_VERSION,
        );

        let query = search_query(&query);
        // A file the user added counts as installed once Modrinth knows it.
        let files = modrinth::file_projects(&directory)
            .await
            .inspect_err(|error| warn!("Failed to look up mod files: {:?}", error))
            .unwrap_or_default();

        let mut installed: Vec<String> = files.into_iter().collect();
        let branch = options.version_options.options.get(&instance.id);
        installed.extend(
            branch
                .and_then(|branch| branch.modrinth_mods.get(launch::SUPPORTED_MC_VERSION))
                .into_iter()
                .flatten()
                .map(|installed| installed.project_id.clone()),
        );

        // Fabric API is installed by the launcher, so Modrinth must not offer it again.
        let fabric_api = launch::fabric_api().await.ok().map(|api| LoaderMod {
            required: true,
            enabled: true,
            name: "Fabric API".to_owned(),
            source: ModSource::Repository {
                repository: "modrinth".to_owned(),
                artifact: format!("maven.modrinth:fabric-api:{}", api.version_id),
            },
        });
        let built_in: Vec<LoaderMod> = fabric_api.into_iter().collect();
        let installed: std::collections::HashSet<String> = installed.into_iter().collect();

        let hits = modrinth::search(query, launch::SUPPORTED_MC_VERSION, launch::LOADER).await?;
        let hits = modrinth::tell(
            hits,
            &Held {
                launched: &built_in,
                recommended: &built_in,
                installed: &installed,
            },
        );

        anyhow::Ok(Search::new(
            hits,
            query,
            launch::SUPPORTED_MC_VERSION,
            launch::LOADER,
        ))
    }
    .await
    .map_err(failed("search Modrinth"))
}

/// The newest version of a project for Minecraft 1.21.11 and Fabric, to install or update to.
#[tauri::command]
pub(crate) async fn modrinth_install(project_id: String) -> Result<ModrinthMod, String> {
    modrinth::newest(&project_id, launch::SUPPORTED_MC_VERSION, launch::LOADER)
        .await
        .map_err(failed("install from Modrinth"))
}
