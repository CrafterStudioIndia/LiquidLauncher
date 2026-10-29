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

//! Offline ("cracked") skin management: local skin library, active skin switcher,
//! 3D/2D skin preview integration, and automatic game instance skin propagation.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;
use tracing::info;

use crate::app::instances;
use crate::{HTTP_CLIENT, LAUNCHER_DIRECTORY};

const SKINS_FOLDER: &str = "skins";
const SKINS_FILE: &str = "skins.json";
const MAX_SKIN_BYTES: usize = 2 * 1024 * 1024; // 2 MB

const STEVE_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAABkElEQVR4nO3aMUvDQBQH8P/9uUlEHP0EDQ7iLBYE5wqCUzft4uIsWV2kH8DFSbfiIAjxCwgtTk4ixS6uoiAinVyUVgQbSK4hiUnz3g9CQu7ykhzc5SU5A4e6t/QVV+7vbMQe3zi+MCgxQjhCOEI4QjhCOEI4Qjjjes7nrfv4XGieQAhnp6kUle21L29QqQbwI260fniK+vI52nu3P/XO1tDt78LH/sw3jPk7Brjy+veP4Xi9uDAfWy9JA8zUGNC764+XKuG0FU+CHp6Gn+NltC1qEBw52FpHFZnwjlrNn8gLPO9hojwIgvg+2+mkyyuazUTxr++vUn1/IIQjhCOEM+E+n9TgaDWy7Ld/Nla2URjHmGLzPPfL6xvKzuYZvLXZQtkRwlmUXPg5n1TDUU4IRwhHCEcIZ8MvO0kNEJ0IzQJCOBZ9AUoppZRSSimllFL/zWQdMOv5BWn//7sQwhHCEcKZtAHynF+QyZwiB0I4QjhCOFv1//8uhHCEcIRwhHA2bYC08wuAORSJEI5FX0DRvgH7zGA2+OLEAgAAAABJRU5ErkJggg==";
const ALEX_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAABa0lEQVR4nO3aMUoDQRQG4H9+tre3FQ0ImxOk08ZSQVC08gS5gCB4AS8hKSwsRUi5lWUsxFjFwt4DiBIhYhays5PdzST53wcLYWbz2IGdN4+dcfC4P97+LupvHZwV/n/34sphiRHiCHGEOEIcIY4QR4hzvnW+aYd3b1HrBEJcUuamWdXe68MtVh3LDH7zqIuTweNf2/j3uM1XBq9kDmhVHFToW7FSOWCYPf9ecjlgYqeTYt0Q4ly+4XwvncoJw4/Pqf6nl/fCOXva7VSqK3o3Wen4l+39yt8dCHGEOEKcy8/5UF/pxsy+yRy9HvSxaL5cMtcyGCrGwEMR4ghxCZbc/7U+RA9ZqfsIcYQ4QlySr/VDbRXUARJJcNQfoVHtZsMT4hj7AYwxxhhjjDHGGGMWzdUdsM7zBXXs//sQ4ghxhDhXNUCT5wvq2P/3IcQR4ghxybrv//sQ4ghxhLikaoCq5wtiI8QR4hj7AWL7AQHJVSGlO1V2AAAAAElFTkSuQmCC";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoredSkin {
    pub id: String,
    pub name: String,
    /// "classic" (4px arms) or "slim" (3px arms)
    pub model: String,
    pub active: bool,
    pub file_name: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_url: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
struct SkinLibraryFile {
    #[serde(default)]
    skins: Vec<StoredSkin>,
}

fn skins_dir() -> PathBuf {
    LAUNCHER_DIRECTORY.config_dir().join(SKINS_FOLDER)
}

fn index_file() -> PathBuf {
    skins_dir().join(SKINS_FILE)
}

pub fn check_skin_png(bytes: &[u8]) -> Result<(u32, u32)> {
    if bytes.len() < 24 || !bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        bail!("The file is not a valid PNG image.");
    }
    if &bytes[12..16] != b"IHDR" {
        bail!("The file does not contain a valid PNG IHDR header.");
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

    if (width != 64 || (height != 64 && height != 32))
        && (width != 128 || (height != 128 && height != 64))
    {
        bail!(
            "Invalid Minecraft skin dimensions: {}x{}. Skins must be 64x64 or 64x32.",
            width,
            height
        );
    }
    Ok((width, height))
}

async fn read_library() -> Result<SkinLibraryFile> {
    let dir = skins_dir();
    fs::create_dir_all(&dir).await?;
    let path = index_file();

    let mut file: SkinLibraryFile = match fs::read_to_string(&path).await {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => SkinLibraryFile::default(),
    };

    if file.skins.is_empty() {
        // Initialize default Steve and Alex
        let steve_bytes = STANDARD.decode(STEVE_PNG_BASE64)?;
        let alex_bytes = STANDARD.decode(ALEX_PNG_BASE64)?;

        let steve_path = dir.join("steve.png");
        let alex_path = dir.join("alex.png");

        let _ = fs::write(&steve_path, &steve_bytes).await;
        let _ = fs::write(&alex_path, &alex_bytes).await;

        file.skins = vec![
            StoredSkin {
                id: "steve".to_string(),
                name: "Steve".to_string(),
                model: "classic".to_string(),
                active: true,
                file_name: "steve.png".to_string(),
                created_at: Utc::now(),
                data_url: None,
            },
            StoredSkin {
                id: "alex".to_string(),
                name: "Alex".to_string(),
                model: "slim".to_string(),
                active: false,
                file_name: "alex.png".to_string(),
                created_at: Utc::now(),
                data_url: None,
            },
        ];
        write_library(&file).await?;
    }

    Ok(file)
}

async fn write_library(file: &SkinLibraryFile) -> Result<()> {
    let dir = skins_dir();
    fs::create_dir_all(&dir).await?;
    let path = index_file();
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(file)?).await?;
    fs::rename(&temp, &path).await?;
    Ok(())
}

/// Returns all skins with their base64 `data:` URL populated for preview.
pub async fn list() -> Result<Vec<StoredSkin>> {
    let library = read_library().await?;
    let dir = skins_dir();

    let mut result = Vec::with_capacity(library.skins.len());
    for mut skin in library.skins {
        let skin_path = dir.join(&skin.file_name);
        if let Ok(bytes) = fs::read(&skin_path).await {
            skin.data_url = Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)));
        }
        result.push(skin);
    }
    Ok(result)
}

/// Returns the currently active skin, or None.
pub async fn get_active() -> Result<Option<StoredSkin>> {
    let skins = list().await?;
    Ok(skins.into_iter().find(|s| s.active))
}

fn sanitize_id(name: &str, skins: &[StoredSkin]) -> String {
    let base: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    let base = if base.is_empty() {
        "skin".to_string()
    } else {
        base
    };

    let taken = |candidate: &str| skins.iter().any(|s| s.id == candidate);
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

/// Adds a new skin from a local file path.
pub async fn add_from_file(
    name: &str,
    model: &str,
    source_path: &Path,
    set_active: bool,
    data_dir: &Path,
) -> Result<StoredSkin> {
    let name = name.trim();
    if name.is_empty() {
        bail!("Skin name cannot be empty.");
    }
    if !source_path.is_file() {
        bail!("Selected skin file does not exist.");
    }

    let bytes = fs::read(source_path).await?;
    if bytes.len() > MAX_SKIN_BYTES {
        bail!("Skin image file exceeds 2 MB limit.");
    }
    check_skin_png(&bytes)?;

    add_skin_bytes(name, model, bytes, set_active, data_dir).await
}

/// Adds a skin from raw PNG bytes.
pub async fn add_skin_bytes(
    name: &str,
    model: &str,
    bytes: Vec<u8>,
    set_active: bool,
    data_dir: &Path,
) -> Result<StoredSkin> {
    let name = name.trim();
    if name.is_empty() {
        bail!("Skin name cannot be empty.");
    }
    check_skin_png(&bytes)?;

    let model = if model.eq_ignore_ascii_case("slim") {
        "slim".to_string()
    } else {
        "classic".to_string()
    };

    let mut library = read_library().await?;
    let id = sanitize_id(name, &library.skins);
    let file_name = format!("{id}.png");
    let target_path = skins_dir().join(&file_name);

    fs::write(&target_path, &bytes).await?;

    if set_active {
        for s in &mut library.skins {
            s.active = false;
        }
    }

    let new_skin = StoredSkin {
        id: id.clone(),
        name: name.to_string(),
        model,
        active: set_active || library.skins.is_empty(),
        file_name,
        created_at: Utc::now(),
        data_url: Some(format!("data:image/png;base64,{}", STANDARD.encode(&bytes))),
    };

    library.skins.push(new_skin.clone());
    write_library(&library).await?;

    if new_skin.active {
        let _ = sync_skin_to_instances(&new_skin, &target_path, data_dir).await;
    }

    Ok(new_skin)
}

/// Fetches skin from a Mojang player profile.
pub async fn add_from_mojang(
    name: &str,
    model_override: Option<&str>,
    username: &str,
    set_active: bool,
    data_dir: &Path,
) -> Result<StoredSkin> {
    let username = username.trim();
    if username.is_empty()
        || username.len() > 16
        || !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        bail!("Invalid Minecraft username. Must be 1-16 alphanumeric characters or underscores.");
    }

    // Step 1: Query Mojang profile UUID
    let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{username}");
    let profile_res = HTTP_CLIENT
        .get(&profile_url)
        .send()
        .await
        .context("Failed to connect to Mojang API")?;

    if profile_res.status() == reqwest::StatusCode::NOT_FOUND
        || profile_res.status() == reqwest::StatusCode::NO_CONTENT
    {
        bail!("Minecraft player '{}' does not exist.", username);
    }
    let profile_json: Value = profile_res
        .error_for_status()?
        .json()
        .await
        .context("Failed to parse Mojang profile response")?;

    let uuid = profile_json
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Mojang profile missing player UUID"))?;

    // Step 2: Query Mojang Session Server
    let session_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}");
    let session_json: Value = HTTP_CLIENT
        .get(&session_url)
        .send()
        .await
        .context("Failed to connect to Mojang session server")?
        .error_for_status()?
        .json()
        .await
        .context("Failed to parse Mojang session profile")?;

    let properties = session_json
        .get("properties")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("Player profile contains no properties"))?;

    let texture_prop = properties
        .iter()
        .find(|p| p.get("name").and_then(Value::as_str) == Some("textures"))
        .ok_or_else(|| anyhow::anyhow!("Player profile has no textures"))?;

    let texture_b64 = texture_prop
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Textures property value is missing"))?;

    let texture_bytes = STANDARD
        .decode(texture_b64)
        .context("Failed to decode texture base64")?;
    let texture_json: Value =
        serde_json::from_slice(&texture_bytes).context("Failed to parse texture JSON")?;

    let skin_obj = texture_json
        .pointer("/textures/SKIN")
        .ok_or_else(|| anyhow::anyhow!("Player '{}' has no custom skin set.", username))?;

    let skin_url = skin_obj
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Skin texture URL is missing"))?;

    // Detect model: "slim" or "classic"
    let detected_model = if skin_obj
        .pointer("/metadata/model")
        .and_then(Value::as_str)
        == Some("slim")
    {
        "slim"
    } else {
        "classic"
    };

    let final_model = model_override.unwrap_or(detected_model);

    // Download skin image
    let skin_bytes = HTTP_CLIENT
        .get(skin_url)
        .send()
        .await
        .context("Failed to download skin texture image")?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let display_name = if name.trim().is_empty() {
        username
    } else {
        name.trim()
    };

    add_skin_bytes(display_name, final_model, skin_bytes, set_active, data_dir).await
}

/// Sets a skin as active, updates the library, and propagates to instances.
pub async fn set_active(id: &str, data_dir: &Path) -> Result<StoredSkin> {
    let mut library = read_library().await?;
    let mut found = None;

    for s in &mut library.skins {
        if s.id == id {
            s.active = true;
            found = Some(s.clone());
        } else {
            s.active = false;
        }
    }

    let skin = found.ok_or_else(|| anyhow::anyhow!("Skin with ID '{}' not found", id))?;
    write_library(&library).await?;

    let skin_path = skins_dir().join(&skin.file_name);
    let _ = sync_skin_to_instances(&skin, &skin_path, data_dir).await;

    let mut result = skin;
    if let Ok(bytes) = fs::read(&skin_path).await {
        result.data_url = Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)));
    }
    Ok(result)
}

/// Updates an existing skin's name or model type.
pub async fn update_skin(
    id: &str,
    new_name: Option<&str>,
    new_model: Option<&str>,
    data_dir: &Path,
) -> Result<StoredSkin> {
    let mut library = read_library().await?;
    let mut updated = None;

    for s in &mut library.skins {
        if s.id == id {
            if let Some(name) = new_name {
                let name = name.trim();
                if !name.is_empty() {
                    s.name = name.to_string();
                }
            }
            if let Some(model) = new_model {
                s.model = if model.eq_ignore_ascii_case("slim") {
                    "slim".to_string()
                } else {
                    "classic".to_string()
                };
            }
            updated = Some(s.clone());
            break;
        }
    }

    let skin = updated.ok_or_else(|| anyhow::anyhow!("Skin with ID '{}' not found", id))?;
    write_library(&library).await?;

    if skin.active {
        let skin_path = skins_dir().join(&skin.file_name);
        let _ = sync_skin_to_instances(&skin, &skin_path, data_dir).await;
    }

    let skin_path = skins_dir().join(&skin.file_name);
    let mut result = skin;
    if let Ok(bytes) = fs::read(&skin_path).await {
        result.data_url = Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)));
    }
    Ok(result)
}

/// Deletes a skin by ID.
pub async fn delete(id: &str, data_dir: &Path) -> Result<Vec<StoredSkin>> {
    let mut library = read_library().await?;
    if library.skins.len() <= 1 {
        bail!("Cannot delete the only skin in your library.");
    }

    let before_len = library.skins.len();
    let was_active = library.skins.iter().any(|s| s.id == id && s.active);

    let mut file_to_remove = None;
    if let Some(pos) = library.skins.iter().position(|s| s.id == id) {
        file_to_remove = Some(library.skins[pos].file_name.clone());
        library.skins.remove(pos);
    }

    if library.skins.len() == before_len {
        bail!("Skin '{}' does not exist.", id);
    }

    if let Some(fname) = file_to_remove {
        let path = skins_dir().join(fname);
        let _ = fs::remove_file(path).await;
    }

    if was_active && !library.skins.is_empty() {
        library.skins[0].active = true;
        let skin = library.skins[0].clone();
        let skin_path = skins_dir().join(&skin.file_name);
        let _ = sync_skin_to_instances(&skin, &skin_path, data_dir).await;
    }

    write_library(&library).await?;
    list().await
}

/// Exports a skin file to an external path.
pub async fn export(id: &str, destination: &Path) -> Result<()> {
    let library = read_library().await?;
    let skin = library
        .skins
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Skin '{}' not found.", id))?;

    let source = skins_dir().join(&skin.file_name);
    if !source.is_file() {
        bail!("Skin file does not exist on disk.");
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::copy(&source, destination).await?;
    Ok(())
}

/// Propagates the active skin into instance directories for LiquidBounce and offline mods.
pub async fn sync_skin_to_instances(skin: &StoredSkin, skin_file: &Path, data_dir: &Path) -> Result<()> {
    if !skin_file.is_file() {
        return Ok(());
    }

    let skin_bytes = fs::read(skin_file).await?;

    // 1. Write to central launcher data dir <data>/skins/active.png
    let central_skins = data_dir.join(SKINS_FOLDER);
    let _ = fs::create_dir_all(&central_skins).await;
    let _ = fs::write(central_skins.join("active.png"), &skin_bytes).await;
    let meta = json!({
        "id": skin.id,
        "name": skin.name,
        "model": skin.model,
    });
    let _ = fs::write(
        central_skins.join("active.json"),
        serde_json::to_vec_pretty(&meta)?,
    )
    .await;

    // 2. Propagate to every known game instance
    if let Ok(instance_list) = instances::list(data_dir).await {
        for instance in instance_list {
            sync_skin_to_single_instance(&skin.model, &skin_bytes, skin_file, data_dir, &instance.id).await;
        }
    }

    Ok(())
}

/// Syncs skin to a specific instance folder.
pub async fn sync_skin_to_single_instance(
    model: &str,
    skin_bytes: &[u8],
    skin_file: &Path,
    data_dir: &Path,
    instance_id: &str,
) {
    let game_dir = instances::game_dir(data_dir, instance_id);
    if !game_dir.exists() {
        let _ = fs::create_dir_all(&game_dir).await;
    }

    // Direct skin.png in instance folder
    let _ = fs::write(game_dir.join("skin.png"), skin_bytes).await;

    // CustomSkinLoader compatibility: <gameDir>/CustomSkinLoader/LocalSkin/skins/
    let csl_dir = game_dir.join("CustomSkinLoader").join("LocalSkin").join("skins");
    if fs::create_dir_all(&csl_dir).await.is_ok() {
        let _ = fs::write(csl_dir.join("default.png"), skin_bytes).await;
        let csl_meta = json!({
            "type": if model == "slim" { "slim" } else { "default" }
        });
        let _ = fs::write(
            csl_dir.join("default.json"),
            serde_json::to_vec_pretty(&csl_meta).unwrap_or_default(),
        )
        .await;
    }

    // LiquidBounce ModuleSkinChanger configuration: <gameDir>/LiquidBounce/modules.json
    let lb_dir = game_dir.join("LiquidBounce");
    let _ = fs::create_dir_all(&lb_dir).await;
    let modules_json_path = lb_dir.join("modules.json");

    let lb_model_tag = if model == "slim" { "Slim" } else { "Default" };
    let skin_abs_path = skin_file.to_string_lossy().into_owned();

    let mut root: Value = if let Ok(content) = fs::read_to_string(&modules_json_path).await {
        serde_json::from_str(&content).unwrap_or_else(|_| json!({ "name": "modules", "value": [] }))
    } else {
        json!({ "name": "modules", "value": [] })
    };

    if let Some(arr) = root.get_mut("value").and_then(Value::as_array_mut) {
        let skin_changer = arr.iter_mut().find(|item| {
            item.get("name").and_then(Value::as_str) == Some("SkinChanger")
        });

        let target_module = json!({
            "name": "SkinChanger",
            "state": true,
            "hidden": false,
            "value": [
                {
                    "name": "ForceOverride",
                    "value": true
                },
                {
                    "name": "UploadSkin",
                    "value": false
                },
                {
                    "name": "Mode",
                    "active": "File",
                    "value": [],
                    "choices": {
                        "File": {
                            "name": "File",
                            "value": [
                                {
                                    "name": "Image",
                                    "value": skin_abs_path
                                },
                                {
                                    "name": "Model",
                                    "value": lb_model_tag
                                }
                            ]
                        }
                    }
                }
            ]
        });

        if let Some(existing) = skin_changer {
            *existing = target_module;
        } else {
            arr.push(target_module);
        }

        let _ = fs::write(
            &modules_json_path,
            serde_json::to_vec_pretty(&root).unwrap_or_default(),
        )
        .await;
        info!(
            "Configured LiquidBounce SkinChanger with skin {} for instance {}",
            skin_abs_path, instance_id
        );
    }
}

/// Ensures the active skin is synced for the current player before launch.
pub async fn sync_for_launch(data_dir: &Path, instance_id: &str, player_name: &str) {
    if let Ok(Some(active)) = get_active().await {
        let skin_path = skins_dir().join(&active.file_name);
        if let Ok(bytes) = fs::read(&skin_path).await {
            sync_skin_to_single_instance(&active.model, &bytes, &skin_path, data_dir, instance_id).await;

            // Also write player-specific CustomSkinLoader files: <player_name>.png and <player_name>.json
            let csl_dir = instances::game_dir(data_dir, instance_id)
                .join("CustomSkinLoader")
                .join("LocalSkin")
                .join("skins");
            if fs::create_dir_all(&csl_dir).await.is_ok() {
                let _ = fs::write(csl_dir.join(format!("{player_name}.png")), &bytes).await;
                let csl_meta = json!({
                    "type": if active.model == "slim" { "slim" } else { "default" }
                });
                let _ = fs::write(
                    csl_dir.join(format!("{player_name}.json")),
                    serde_json::to_vec_pretty(&csl_meta).unwrap_or_default(),
                )
                .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_skin_png_valid_steve() {
        let bytes = STANDARD.decode(STEVE_PNG_BASE64).unwrap();
        let (w, h) = check_skin_png(&bytes).unwrap();
        assert_eq!(w, 64);
        assert_eq!(h, 64);
    }

    #[test]
    fn test_check_skin_png_invalid_data() {
        let not_png = b"NOT_A_PNG_FILE_HEADER_123456789";
        assert!(check_skin_png(not_png).is_err());
    }

    #[test]
    fn test_sanitize_id() {
        let skins = vec![StoredSkin {
            id: "steve".to_string(),
            name: "Steve".to_string(),
            model: "classic".to_string(),
            active: true,
            file_name: "steve.png".to_string(),
            created_at: Utc::now(),
            data_url: None,
        }];
        assert_eq!(sanitize_id("Steve", &skins), "steve-2");
        assert_eq!(sanitize_id("Cool Knight!", &skins), "coolknight");
        assert_eq!(sanitize_id("!!!", &skins), "skin");
    }
}
