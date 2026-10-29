use std::env;

use anyhow::{bail, Context, Result};
use tokio::fs;

const HOSTS_PATH: &str = "Windows\\System32\\drivers\\etc\\hosts";
const HOSTS: [&str; 2] = ["mojang.com", "minecraft.net"];

/// Many users have modified their hosts file to block Mojang, usually through a third-party program.
/// The launcher needs Mojang's servers to download the game, so tell them what blocks it.
pub async fn check_hosts_file() -> Result<()> {
    // Get location of Windows hosts file dynamically
    // "SystemDrive" env, if not assigned default to C:
    let system_drive = env::var("SystemDrive").unwrap_or("C:".to_string());
    let hosts_path = format!("{}\\{}", system_drive, HOSTS_PATH);

    // Check if hosts file exists, if not cancel this check with OK
    if let Ok(exists) = fs::try_exists(&hosts_path).await {
        if !exists {
            return Ok(());
        }
    }

    // Check if the hosts file has been modified
    let hosts_file = fs::read_to_string(&hosts_path)
        .await
        .context(format!("Failed to read hosts file at {}", hosts_path))?;

    let flagged_entries = hosts_file
        .lines()
        .filter(|line| {
            if line.starts_with('#') {
                return false;
            }

            let mut parts = line.split_whitespace();
            let _ = match parts.next() {
                Some(ip) => ip,
                None => return false,
            };
            let domain = match parts.next() {
                Some(domain) => domain,
                None => return false,
            };

            HOSTS.iter().any(|&entry| domain.contains(entry))
        })
        .collect::<Vec<_>>();

    if !flagged_entries.is_empty() {
        bail!(
            "The hosts file has been modified to block Mojang's servers.\n\
            \n\
            The launcher cannot download Minecraft while these entries exist:\n\
            {}\n\n\
            The file is located at:\n\
            {}",
            flagged_entries.join("\n"),
            hosts_path
        );
    }

    Ok(())
}
