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

//! Game instances. Each one has its own game directory, mods, worlds and settings, while the
//! downloaded Minecraft, Fabric and Java runtime are shared by the launcher.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::app::launch::SUPPORTED_MC_VERSION;

const FILE_NAME: &str = "instances.json";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    /// Folder name, unique, and safe to use as a directory.
    pub id: String,
    pub name: String,
    pub minecraft: String,
    pub created_at: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(Default, Serialize, Deserialize)]
struct InstanceFile {
    #[serde(default)]
    instances: Vec<Instance>,
}

fn file(data: &Path) -> PathBuf {
    data.join(FILE_NAME)
}

/// Validates that an instance ID is safe to use as a directory component.
pub fn validate_instance_id(id: &str) -> Result<()> {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        bail!("Invalid instance ID length.");
    }
    if !trimmed.chars().all(|character| character.is_ascii_alphanumeric() || character == '-') {
        bail!("Instance ID must contain only alphanumeric characters and hyphens.");
    }
    Ok(())
}

/// The `mods` and `gameDir` folder of an instance.
pub fn game_dir(data: &Path, id: &str) -> PathBuf {
    let sanitized_id: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    data.join("gameDir").join(sanitized_id)
}

/// Where files added to an instance are kept before they are copied into its `mods` folder.
pub fn custom_mods_dir(data: &Path, id: &str) -> PathBuf {
    let sanitized_id: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    data.join("custom_mods")
        .join(format!("{sanitized_id}-{SUPPORTED_MC_VERSION}"))
}

async fn read(data: &Path) -> Result<InstanceFile> {
    let mut file: InstanceFile = match fs::read_to_string(file(data)).await {
        Ok(raw) => serde_json::from_str(&raw)
            .with_context(|| format!("Unexpected layout of {}", file(data).display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(InstanceFile::default()),
        Err(error) => return Err(error)
            .with_context(|| format!("Failed to read {}", file(data).display())),
    };

    let mut modified = false;
    for instance in &mut file.instances {
        if instance.minecraft != SUPPORTED_MC_VERSION {
            let old_version = instance.minecraft.clone();
            tracing::info!(
                "Auto-migrating instance '{}' from old version {} to {}",
                instance.id,
                old_version,
                SUPPORTED_MC_VERSION
            );
            instance.minecraft = SUPPORTED_MC_VERSION.to_owned();

            // Migrate custom_mods folder if present
            let old_mods = data.join("custom_mods").join(format!("{}-{old_version}", instance.id));
            let new_mods = custom_mods_dir(data, &instance.id);
            if old_mods.is_dir() && !new_mods.exists() {
                let _ = fs::rename(&old_mods, &new_mods).await;
            }
            modified = true;
        }
    }

    if modified {
        let _ = write(data, &file).await;
    }

    Ok(file)
}

async fn write(data: &Path, file: &InstanceFile) -> Result<()> {
    fs::create_dir_all(data).await?;
    let path = self::file(data);
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(file)?).await?;
    fs::rename(&temporary, &path).await?;
    Ok(())
}

/// Every instance, oldest first.
pub async fn list(data: &Path) -> Result<Vec<Instance>> {
    Ok(read(data).await?.instances)
}

pub async fn find(data: &Path, id: &str) -> Result<Option<Instance>> {
    Ok(list(data).await?.into_iter().find(|instance| instance.id == id))
}

/// The instance the launcher launches, creating the first one when there is none.
pub async fn ensure(data: &Path) -> Result<Instance> {
    if let Some(instance) = list(data).await?.into_iter().next() {
        return Ok(instance);
    }

    create(data, &format!("Minecraft {SUPPORTED_MC_VERSION}")).await
}

pub async fn create(data: &Path, name: &str) -> Result<Instance> {
    let name = name.trim();
    if name.is_empty() {
        bail!("Give the instance a name.");
    }
    if name.chars().count() > 48 {
        bail!("Instance names are limited to 48 characters.");
    }

    let mut file = read(data).await?;
    let id = unique_id(name, &file.instances);
    let instance = Instance {
        id,
        name: name.to_owned(),
        minecraft: SUPPORTED_MC_VERSION.to_owned(),
        created_at: Utc::now(),
        last_played: None,
    };
    file.instances.push(instance.clone());
    write(data, &file).await?;
    Ok(instance)
}

pub async fn delete(data: &Path, id: &str) -> Result<()> {
    validate_instance_id(id)?;
    let mut file = read(data).await?;
    if file.instances.len() <= 1 {
        bail!("The last instance cannot be removed.");
    }

    let before = file.instances.len();
    file.instances.retain(|instance| instance.id != id);
    if file.instances.len() == before {
        bail!("The instance does not exist.");
    }
    write(data, &file).await?;

    let expected_game_parent = data.join("gameDir");
    let game_dir = game_dir(data, id);
    if game_dir.exists() && game_dir.parent() == Some(expected_game_parent.as_path()) {
        fs::remove_dir_all(&game_dir)
            .await
            .with_context(|| format!("Failed to remove {}", game_dir.display()))?;
    }

    let expected_mods_parent = data.join("custom_mods");
    let custom_mods = custom_mods_dir(data, id);
    if custom_mods.exists() && custom_mods.parent() == Some(expected_mods_parent.as_path()) {
        let _ = fs::remove_dir_all(&custom_mods).await;
    }
    Ok(())
}

pub async fn rename(data: &Path, id: &str, name: &str) -> Result<Instance> {
    validate_instance_id(id)?;
    let name = name.trim();
    if name.is_empty() {
        bail!("Give the instance a name.");
    }
    if name.chars().count() > 48 {
        bail!("Instance names are limited to 48 characters.");
    }

    let mut file = read(data).await?;
    let instance = file
        .instances
        .iter_mut()
        .find(|instance| instance.id == id)
        .with_context(|| format!("The instance {id} does not exist."))?;
    instance.name = name.to_owned();
    let instance = instance.clone();
    write(data, &file).await?;
    Ok(instance)
}

pub async fn touch(data: &Path, id: &str) -> Result<()> {
    let mut file = read(data).await?;
    if let Some(instance) = file.instances.iter_mut().find(|instance| instance.id == id) {
        instance.last_played = Some(Utc::now());
        write(data, &file).await?;
    }
    Ok(())
}

fn unique_id(name: &str, instances: &[Instance]) -> String {
    let base: String = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    let base = if base.is_empty() {
        "instance".to_owned()
    } else {
        base
    };

    let taken = |candidate: &str| instances.iter().any(|instance| instance.id == candidate);

    if !taken(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !taken(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

/// The instance the options point at, or the first one when that instance is gone.
pub async fn selected(data: &Path, preferred: &str) -> Result<Instance> {
    if !preferred.is_empty() {
        if let Some(instance) = find(data, preferred).await? {
            return Ok(instance);
        }
    }
    ensure(data).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_instance_id() {
        assert!(validate_instance_id("default").is_ok());
        assert!(validate_instance_id("my-instance-1").is_ok());
        assert!(validate_instance_id("").is_err());
        assert!(validate_instance_id("../escape").is_err());
        assert!(validate_instance_id("slash/name").is_err());
        assert!(validate_instance_id("backslash\\name").is_err());
        assert!(validate_instance_id("has space").is_err());
        assert!(validate_instance_id(&"a".repeat(100)).is_err());
    }
}
