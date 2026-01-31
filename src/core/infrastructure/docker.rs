// Copyright 2026 eraflo
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(any(target_os = "windows", target_os = "macos"))]
use anyhow::Context;
use anyhow::{Result, bail};
use std::process::Command;

/// Checks if Docker is available and running.
/// If installed but not running, attempts to start Docker Desktop (Windows).
pub fn ensure_docker_available() -> Result<()> {
    // 1. Check if Docker is responsive (daemon running)
    let status = Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if status.is_ok_and(|s| s.success()) {
        return Ok(());
    }

    // 2. Check if Docker is installed but not running
    let version_check = Command::new("docker")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match version_check {
        Ok(_) => {
            // Docker is installed, try to start it
            println!("🐳 Docker daemon is not running. Attempting to start Docker Desktop...");

            #[cfg(target_os = "windows")]
            {
                let docker_path = std::path::PathBuf::from(
                    "C:\\Program Files\\Docker\\Docker\\Docker Desktop.exe",
                );
                if docker_path.exists() {
                    Command::new(docker_path)
                        .spawn()
                        .context("Failed to launch Docker Desktop")?;
                } else {
                    bail!(
                        "Docker is installed but I couldn't find 'Docker Desktop.exe' to auto-start it. Please start it manually."
                    );
                }
            }
            #[cfg(target_os = "macos")]
            {
                Command::new("open")
                    .arg("-a")
                    .arg("Docker")
                    .spawn()
                    .context("Failed to launch Docker on macOS")?;
            }

            #[cfg(target_os = "linux")]
            {
                println!("Attempting to start Docker service...");
                // Try systemctl, hoping for polkit or user privileges
                let _ = Command::new("systemctl")
                    .arg("start")
                    .arg("docker")
                    .status();
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            {
                bail!("Docker is not running. Please start the Docker daemon manually.");
            }

            // 3. Wait loop (up to 45 seconds)
            println!("⏳ Waiting for Docker to be ready...");
            let start = std::time::Instant::now();
            while start.elapsed().as_secs() < 45 {
                print!(".");
                use std::io::Write;
                let _ = std::io::stdout().flush();

                std::thread::sleep(std::time::Duration::from_secs(2));
                let check = Command::new("docker")
                    .arg("info")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();

                if check.is_ok_and(|s| s.success()) {
                    println!("\n✅ Docker started successfully!");
                    return Ok(());
                }
            }
            println!();
            bail!("Timed out waiting for Docker to start. Please checks its status.");
        }
        Err(_) => {
            // Docker is not installed
            bail!(
                "Docker is not installed or not in PATH.\n\
                Sandbox mode requires Docker to isolate execution.\n\
                👉 Please download it here: https://www.docker.com/products/docker-desktop"
            );
        }
    }
}
