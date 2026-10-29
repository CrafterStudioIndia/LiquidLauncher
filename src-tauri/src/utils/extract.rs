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

use anyhow::{bail, Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use async_zip::base::read::seek::ZipFileReader;
use futures::StreamExt;
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;

/// Extracts everything from the ZIP archive to the output directory
pub async fn zip_extract<R>(archive: R, out_dir: &Path) -> Result<()>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let out_dir_abs = out_dir.absolutize()?;
    let mut reader = ZipFileReader::with_tokio(BufReader::new(archive)).await?;
    for index in 0..reader.file().entries().len() {
        let entry = reader.file().entries().get(index).unwrap();
        let file_name = entry.filename().as_str()?.to_owned();
        let sanitized = sanitize_file_path(&file_name);
        if sanitized.as_os_str().is_empty() {
            continue;
        }

        let path = out_dir.join(&sanitized);
        let path_abs = path.absolutize_from(out_dir);
        if !path_abs.starts_with(&out_dir_abs) {
            bail!("Zip slip detected: '{}' escapes output directory", file_name);
        }

        let entry_is_dir = file_name.ends_with('/');
        let mut entry_reader = reader.reader_without_entry(index).await?.compat();

        if entry_is_dir {
            if !path.exists() {
                create_dir_all(&path).await?;
            }
        } else {
            let parent = path.parent().unwrap();
            if !parent.is_dir() {
                create_dir_all(parent).await?;
            }

            let mut writer = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await
                .context("Failed to create extracted file")?;
            io::copy(&mut entry_reader, &mut writer).await?;
        }
    }
    Ok(())
}

pub async fn tar_gz_extract<R>(archive: R, out_dir: &Path) -> Result<()>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let mut decoder = GzipDecoder::new(BufReader::new(archive));
    let mut decoded_data: Vec<u8> = vec![];
    decoder.read_to_end(&mut decoded_data).await?;

    let mut ar = tokio_tar::Archive::new(&decoded_data[..]);
    let mut entries = ar.entries()?;
    while let Some(entry) = entries.next().await {
        let mut entry = entry?;
        let entry_path = entry.path()?.into_owned();
        if entry_path.is_absolute()
            || entry_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_)))
        {
            bail!("Tar archive entry escapes target directory: {}", entry_path.display());
        }
        entry.unpack_in(out_dir).await?;
    }
    Ok(())
}

/// Returns a relative path without reserved names, redundant separators, ".", or "..".
fn sanitize_file_path(path: &str) -> PathBuf {
    // Replaces backwards slashes
    path.replace('\\', "/")
        // Sanitizes each component
        .split('/')
        .map(sanitize_filename::sanitize)
        .collect()
}
