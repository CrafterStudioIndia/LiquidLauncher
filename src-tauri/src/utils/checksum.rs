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

use anyhow::Result;
use sha1::Sha1;
use sha2::Sha256;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

pub fn sha1sum(path: &PathBuf) -> Result<String> {
    use sha1::Digest;
    let hash = Sha1::digest(std::fs::read(path)?);
    Ok(base16ct::lower::encode_string(&hash))
}

pub fn sha256sum(path: &Path) -> Result<String> {
    use sha2::Digest;
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(128 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let hash = hasher.finalize();
    Ok(base16ct::lower::encode_string(&hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sha256sum_empty() {
        let temp = std::env::temp_dir().join("test_sha256_empty.tmp");
        let mut file = File::create(&temp).unwrap();
        file.flush().unwrap();
        drop(file);

        let hash = sha256sum(&temp).unwrap();
        let _ = std::fs::remove_file(&temp);
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_sha256sum_content() {
        let temp = std::env::temp_dir().join("test_sha256_content.tmp");
        let mut file = File::create(&temp).unwrap();
        file.write_all(b"liquidbounce").unwrap();
        file.flush().unwrap();
        drop(file);

        let hash = sha256sum(&temp).unwrap();
        let _ = std::fs::remove_file(&temp);
        // printf "liquidbounce" | sha256sum -> b065e2e076eb2d4be89c571f37d65d390418b2011b7e06a71433e5779f122835
        assert_eq!(hash, "b065e2e076eb2d4be89c571f37d65d390418b2011b7e06a71433e5779f122835");
    }
}
