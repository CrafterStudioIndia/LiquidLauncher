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

//! The launcher's own background picture. The file is copied into the launcher's config
//! directory and read back as a data URL, so the webview never needs file-system access.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Serialize;
use tokio::fs;

use crate::LAUNCHER_DIRECTORY;

const PREFIX: &str = "background";
const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Background {
    /// `data:image/png;base64,...`, or `None` when no picture is set.
    pub data_url: Option<String>,
    pub file_name: Option<String>,
}

/// The extension and MIME type of a supported image, checked by its magic bytes.
fn kind(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Some(("png", "image/png"));
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(("jpg", "image/jpeg"));
    }
    if bytes.starts_with(b"GIF8") {
        return Some(("gif", "image/gif"));
    }
    if bytes.starts_with(b"BM") {
        return Some(("bmp", "image/bmp"));
    }
    if bytes.len() > 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(("webp", "image/webp"));
    }
    None
}

async fn stored() -> Result<Option<PathBuf>> {
    let directory = LAUNCHER_DIRECTORY.config_dir();
    let mut entries = match fs::read_dir(directory).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("Failed to read the launcher config directory"),
    };

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(PREFIX)
            && Path::new(name.as_ref())
                .extension()
                .is_some_and(|extension| !extension.eq_ignore_ascii_case("json"))
        {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

async fn remove_stored() {
    if let Ok(Some(path)) = stored().await {
        let _ = fs::remove_file(path).await;
    }
}

pub async fn read() -> Result<Background> {
    let Some(path) = stored().await? else {
        return Ok(Background {
            data_url: None,
            file_name: None,
        });
    };

    let bytes = fs::read(&path).await?;
    if bytes.len() as u64 > MAX_BYTES {
        bail!("The background image is larger than 32 MB.");
    }

    let Some((_, mime)) = kind(&bytes) else {
        return Ok(Background {
            data_url: None,
            file_name: None,
        });
    };

    Ok(Background {
        data_url: Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes))),
        file_name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
    })
}

pub async fn choose(source: &str) -> Result<Background> {
    let source = PathBuf::from(source);
    if !source.is_file() {
        bail!("That file does not exist.");
    }

    let metadata = fs::metadata(&source).await?;
    if metadata.len() > MAX_BYTES {
        bail!("The background image is larger than 32 MB.");
    }

    let bytes = fs::read(&source).await?;
    let (extension, _) = kind(&bytes)
        .with_context(|| "Choose a PNG, JPEG, GIF, BMP or WebP image.")?;

    remove_stored().await;

    let directory = LAUNCHER_DIRECTORY.config_dir();
    fs::create_dir_all(directory).await?;
    let destination = directory.join(format!("{PREFIX}.{extension}"));
    fs::write(&destination, &bytes).await?;

    read().await
}

pub async fn clear() -> Result<()> {
    remove_stored().await;
    Ok(())
}
