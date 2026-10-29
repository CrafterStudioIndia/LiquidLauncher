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

use crate::minecraft::auth::MinecraftAccount;

/// This build only signs in with offline ("cracked") accounts.
#[tauri::command]
pub(crate) async fn login_offline(username: String) -> Result<MinecraftAccount, String> {
    MinecraftAccount::offline(&username).map_err(|error| format!("{error}"))
}

/// An offline account has no session to refresh; the name and id never change.
#[tauri::command]
pub(crate) async fn refresh(account_data: MinecraftAccount) -> Result<MinecraftAccount, String> {
    Ok(account_data)
}

/// There is no remote session to end, the account is only stored locally.
#[tauri::command]
pub(crate) async fn logout() -> Result<(), String> {
    Ok(())
}
