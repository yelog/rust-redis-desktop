use crate::updater::{InstallResult, Result, UpdateError};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use zip::ZipArchive;

pub struct WindowsInstaller;

impl WindowsInstaller {
    pub fn install(zip_path: &PathBuf) -> Result<InstallResult> {
        let temp_dir = dirs::cache_dir()
            .ok_or_else(|| {
                UpdateError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "无法找到缓存目录",
                ))
            })?
            .join("rust-redis-desktop")
            .join("update_temp");

        fs::create_dir_all(&temp_dir).map_err(UpdateError::IoError)?;

        Self::extract_zip(zip_path, &temp_dir)?;

        let current_exe = std::env::current_exe().map_err(UpdateError::IoError)?;

        let new_exe = temp_dir.join("rust-redis-desktop.exe");

        if !new_exe.exists() {
            return Err(UpdateError::InstallError(
                "ZIP包中未找到可执行文件".to_string(),
            ));
        }

        Self::create_update_script(&current_exe, &new_exe)?;

        Ok(InstallResult::RestartInProgress)
    }

    fn extract_zip(zip_path: &PathBuf, dest: &PathBuf) -> Result<()> {
        let file = fs::File::open(zip_path).map_err(UpdateError::IoError)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| UpdateError::InstallError(e.to_string()))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| UpdateError::InstallError(e.to_string()))?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(UpdateError::IoError)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(UpdateError::IoError)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(UpdateError::IoError)?;
                std::io::copy(&mut file, &mut outfile).map_err(UpdateError::IoError)?;
            }
        }

        Ok(())
    }

    fn create_update_script(current_exe: &PathBuf, new_exe: &PathBuf) -> Result<()> {
        let script_path = dirs::cache_dir()
            .ok_or_else(|| {
                UpdateError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "无法找到缓存目录",
                ))
            })?
            .join("rust-redis-desktop")
            .join("update.bat");

        let current_exe_str = current_exe.to_string_lossy();
        let new_exe_str = new_exe.to_string_lossy();

        let script = format!(
            r"@echo off
timeout /t 2 /nobreak > nul
copy /y \"{}\" \"{}\"
del \"{}\"
start \"\" \"{}\"
del \"%~f0\"
exit
",
            new_exe_str, current_exe_str, new_exe_str, current_exe_str
        );

        fs::write(&script_path, script).map_err(UpdateError::IoError)?;

        Command::new("cmd")
            .args(["/C", "start", &script_path.to_string_lossy()])
            .spawn()
            .map_err(|e| UpdateError::InstallError(e.to_string()))?;

        Ok(())
    }
}