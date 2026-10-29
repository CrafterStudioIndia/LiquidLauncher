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

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An offline ("cracked") account. This build intentionally supports no other kind of account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MinecraftAccount {
    #[serde(rename = "Offline")]
    OfflineAccount {
        name: String,
        #[serde(alias = "uuid")]
        id: Uuid,
    },
}

impl MinecraftAccount {
    /// Authenticate using an offline account.
    /// Generates UUID from following format: OfflinePlayer:<username>
    /// Java/Kotlin equivalent: UUID.nameUUIDFromBytes("OfflinePlayer:$name".toByteArray())
    ///
    /// `nameUUIDFromBytes` uses MD5 to generate a UUID from the input bytes. The input bytes
    /// are the UTF-8 bytes of the string "OfflinePlayer:$name", which results in a version 3
    /// UUID.
    pub fn offline(username: &str) -> Result<Self> {
        let username = username.trim();
        if username.is_empty() || username.chars().count() > 16 {
            bail!("Usernames must be between 1 and 16 characters long.");
        }
        if !username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            bail!("Usernames can only contain letters, numbers and underscores.");
        }

        let name_str = format!("OfflinePlayer:{}", username);
        let mut md5: [u8; 16] = md5::compute(name_str.as_bytes()).into();

        md5[6] &= 0x0f; // clear version
        md5[6] |= 0x30; // version 3
        md5[8] &= 0x3f; // clear variant
        md5[8] |= 0x80; // IETF variant

        Ok(MinecraftAccount::OfflineAccount {
            name: username.to_owned(),
            id: Uuid::from_bytes(md5),
        })
    }

    pub fn get_username(&self) -> &str {
        match self {
            MinecraftAccount::OfflineAccount { name, .. } => name,
        }
    }
}
