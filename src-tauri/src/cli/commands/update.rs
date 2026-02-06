use clap::Args;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(not(windows))]
use std::process::Command;
use tempfile::TempDir;

use crate::cli::ui::{highlight, info, success};
use crate::error::AppError;

const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const BINARY_NAME: &str = "cc-switch";
const CHECKSUMS_FILE_NAME: &str = "checksums.txt";

#[derive(Args, Debug, Clone)]
pub struct UpdateCommand {
    /// Target version (example: v4.6.2). Defaults to latest release.
    #[arg(long)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseInfo {
    tag_name: String,
}

struct DownloadedAsset {
    _temp_dir: TempDir,
    archive_path: PathBuf,
}

pub fn execute(cmd: UpdateCommand) -> Result<(), AppError> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target_tag = resolve_target_tag(cmd.version.as_deref())?;
    let target_version = target_tag.trim_start_matches('v');

    if target_version == current_version {
        println!(
            "{}",
            info(&format!("Already on latest version: v{current_version}"))
        );
        return Ok(());
    }

    let asset_name = release_asset_name()?;
    let download_url = format!("{REPO_URL}/releases/download/{target_tag}/{asset_name}");
    let checksum_url = format!("{REPO_URL}/releases/download/{target_tag}/{CHECKSUMS_FILE_NAME}");

    println!(
        "{}",
        highlight(&format!("Current version: v{current_version}"))
    );
    println!("{}", highlight(&format!("Updating to: {target_tag}")));
    println!("{}", info(&format!("Downloading: {download_url}")));
    println!("{}", info(&format!("Verifying checksum: {checksum_url}")));

    let downloaded_asset = download_release_asset(&download_url, &asset_name)?;
    verify_asset_checksum(&downloaded_asset.archive_path, &checksum_url, &asset_name)?;
    let extracted_binary = extract_binary(&downloaded_asset.archive_path)?;
    replace_current_binary(&extracted_binary)?;

    println!(
        "{}",
        success(&format!("Updated successfully to {target_tag}"))
    );
    println!(
        "{}",
        info("Run `cc-switch --version` to verify the installed version.")
    );
    Ok(())
}

fn run_async<T>(
    fut: impl std::future::Future<Output = Result<T, AppError>>,
) -> Result<T, AppError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| AppError::Message(format!("Failed to create runtime: {e}")))?
        .block_on(fut)
}

fn resolve_target_tag(version: Option<&str>) -> Result<String, AppError> {
    let tag = match version.map(str::trim).filter(|v| !v.is_empty()) {
        Some(version) => normalize_tag(version),
        None => run_async(fetch_latest_release_tag())?,
    };
    validate_target_tag(&tag)?;
    Ok(tag)
}

fn validate_target_tag(tag: &str) -> Result<(), AppError> {
    if !tag.starts_with('v') {
        return Err(AppError::Message(format!(
            "Invalid version tag '{tag}': must start with 'v'."
        )));
    }
    if tag.len() > 64 {
        return Err(AppError::Message(format!(
            "Invalid version tag '{tag}': too long."
        )));
    }
    if tag.contains('/') || tag.contains('\\') || tag.contains("..") {
        return Err(AppError::Message(format!(
            "Invalid version tag '{tag}': contains forbidden path characters."
        )));
    }
    if !tag
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_')
    {
        return Err(AppError::Message(format!(
            "Invalid version tag '{tag}': only [A-Za-z0-9._-] allowed."
        )));
    }
    Ok(())
}

fn normalize_tag(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

async fn fetch_latest_release_tag() -> Result<String, AppError> {
    let api_url =
        format!("{REPO_URL}/releases/latest").replace("github.com", "api.github.com/repos");
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| AppError::Message(format!("Failed to initialize HTTP client: {e}")))?;
    let release = client
        .get(api_url)
        .header(reqwest::header::USER_AGENT, "cc-switch-cli-updater")
        .send()
        .await
        .map_err(|e| AppError::Message(format!("Failed to query latest release: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Message(format!("Release API returned error: {e}")))?
        .json::<ReleaseInfo>()
        .await
        .map_err(|e| AppError::Message(format!("Failed to parse latest release response: {e}")))?;
    Ok(release.tag_name)
}

fn release_asset_name() -> Result<String, AppError> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let name = match (os, arch) {
        ("macos", "x86_64") | ("macos", "aarch64") => "cc-switch-cli-darwin-universal.tar.gz",
        ("linux", "x86_64") => "cc-switch-cli-linux-x64-musl.tar.gz",
        ("linux", "aarch64") => "cc-switch-cli-linux-arm64-musl.tar.gz",
        ("windows", "x86_64") => "cc-switch-cli-windows-x64.zip",
        _ => {
            return Err(AppError::Message(format!(
                "Self-update is not supported for platform {os}/{arch}."
            )));
        }
    };

    Ok(name.to_string())
}

fn download_release_asset(url: &str, asset_name: &str) -> Result<DownloadedAsset, AppError> {
    run_async(async move {
        let bytes = reqwest::Client::builder()
            .build()
            .map_err(|e| AppError::Message(format!("Failed to initialize HTTP client: {e}")))?
            .get(url)
            .header(reqwest::header::USER_AGENT, "cc-switch-cli-updater")
            .send()
            .await
            .map_err(|e| AppError::Message(format!("Failed to download release asset: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Message(format!("Release asset request failed: {e}")))?
            .bytes()
            .await
            .map_err(|e| AppError::Message(format!("Failed to read release asset body: {e}")))?;

        let temp_dir = tempfile::tempdir()
            .map_err(|e| AppError::Message(format!("Failed to create temp directory: {e}")))?;
        let archive_path = temp_dir.path().join(asset_name);
        fs::write(&archive_path, &bytes).map_err(|e| AppError::io(&archive_path, e))?;
        Ok(DownloadedAsset {
            _temp_dir: temp_dir,
            archive_path,
        })
    })
}

fn verify_asset_checksum(
    archive_path: &Path,
    checksum_url: &str,
    asset_name: &str,
) -> Result<(), AppError> {
    let checksum_content = run_async(download_text(checksum_url.to_string()))?;
    let expected = parse_checksum_for_asset(&checksum_content, asset_name)?;

    let actual = compute_sha256_hex(archive_path)?;

    if actual != expected {
        return Err(AppError::Message(format!(
            "Checksum mismatch for {asset_name}: expected {expected}, got {actual}."
        )));
    }

    Ok(())
}

fn compute_sha256_hex(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path).map_err(|e| AppError::io(path, e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer).map_err(|e| AppError::io(path, e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

async fn download_text(url: String) -> Result<String, AppError> {
    reqwest::Client::builder()
        .build()
        .map_err(|e| AppError::Message(format!("Failed to initialize HTTP client: {e}")))?
        .get(url)
        .header(reqwest::header::USER_AGENT, "cc-switch-cli-updater")
        .send()
        .await
        .map_err(|e| AppError::Message(format!("Failed to download checksum file: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Message(format!("Checksum file request failed: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::Message(format!("Failed to read checksum file body: {e}")))
}

fn parse_checksum_for_asset(checksum_content: &str, asset_name: &str) -> Result<String, AppError> {
    let expected = checksum_content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let mut file = parts.next()?;
            if parts.next().is_some() {
                return None;
            }
            // sha256sum -b format can include a leading '*'
            if let Some(stripped) = file.strip_prefix('*') {
                file = stripped;
            }

            if file == asset_name {
                Some(hash.to_string())
            } else {
                None
            }
        })
        .next();

    expected.ok_or_else(|| {
        AppError::Message(format!(
            "Unable to find SHA256 for {asset_name} in {CHECKSUMS_FILE_NAME}."
        ))
    })
}

fn extract_binary(archive_path: &Path) -> Result<PathBuf, AppError> {
    let extract_dir = archive_path
        .parent()
        .ok_or_else(|| AppError::Message("Invalid archive path".to_string()))?
        .join("extracted");
    fs::create_dir_all(&extract_dir).map_err(|e| AppError::io(&extract_dir, e))?;

    if cfg!(windows) {
        extract_zip_binary(archive_path, &extract_dir)
    } else {
        extract_tar_binary(archive_path, &extract_dir)
    }
}

#[cfg(not(windows))]
fn extract_tar_binary(archive_path: &Path, extract_dir: &Path) -> Result<PathBuf, AppError> {
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(archive_path)
        .arg("-C")
        .arg(extract_dir)
        .status()
        .map_err(|e| AppError::Message(format!("Failed to run tar for extraction: {e}")))?;

    if !status.success() {
        return Err(AppError::Message(
            "Failed to extract release archive with tar.".to_string(),
        ));
    }

    let binary_path = extract_dir.join(BINARY_NAME);
    if !binary_path.exists() {
        return Err(AppError::Message(format!(
            "Extracted archive does not contain expected binary: {BINARY_NAME}"
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&binary_path, perms).map_err(|e| AppError::io(&binary_path, e))?;
    }

    Ok(binary_path)
}

#[cfg(not(windows))]
fn extract_zip_binary(_archive_path: &Path, _extract_dir: &Path) -> Result<PathBuf, AppError> {
    Err(AppError::Message(
        "ZIP extraction is only supported on Windows.".to_string(),
    ))
}

#[cfg(windows)]
fn extract_zip_binary(archive_path: &Path, extract_dir: &Path) -> Result<PathBuf, AppError> {
    let file = fs::File::open(archive_path).map_err(|e| AppError::io(archive_path, e))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Message(format!("Failed to open ZIP archive: {e}")))?;
    let binary_filename = format!("{BINARY_NAME}.exe");

    let mut entry = zip.by_name(&binary_filename).map_err(|_| {
        AppError::Message(format!("ZIP archive does not contain {binary_filename}"))
    })?;

    let binary_path = extract_dir.join(binary_filename);
    let mut output = fs::File::create(&binary_path).map_err(|e| AppError::io(&binary_path, e))?;
    std::io::copy(&mut entry, &mut output)
        .map_err(|e| AppError::Message(format!("Failed to extract binary from ZIP: {e}")))?;

    Ok(binary_path)
}

#[cfg(windows)]
fn extract_tar_binary(_archive_path: &Path, _extract_dir: &Path) -> Result<PathBuf, AppError> {
    Err(AppError::Message(
        "TAR extraction is not supported on Windows.".to_string(),
    ))
}

fn replace_current_binary(new_binary_path: &Path) -> Result<(), AppError> {
    let current_binary = std::env::current_exe().map_err(|e| {
        AppError::Message(format!("Failed to resolve current executable path: {e}"))
    })?;
    let parent = current_binary.parent().ok_or_else(|| {
        AppError::Message("Current executable path has no parent directory.".to_string())
    })?;

    let staged_binary = parent.join(format!("{BINARY_NAME}.new"));
    let backup_binary = parent.join(format!("{BINARY_NAME}.old"));

    if backup_binary.exists() {
        fs::remove_file(&backup_binary).map_err(|e| AppError::io(&backup_binary, e))?;
    }
    if staged_binary.exists() {
        fs::remove_file(&staged_binary).map_err(|e| AppError::io(&staged_binary, e))?;
    }

    fs::copy(new_binary_path, &staged_binary)
        .map_err(|e| map_update_permission_error(&current_binary, e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&staged_binary, perms)
            .map_err(|e| map_update_permission_error(&current_binary, e))?;
    }

    fs::rename(&current_binary, &backup_binary)
        .map_err(|e| map_update_permission_error(&current_binary, e))?;

    if let Err(err) = fs::rename(&staged_binary, &current_binary) {
        let restore_err = fs::rename(&backup_binary, &current_binary).err();
        if let Some(restore_err) = restore_err {
            return Err(AppError::Message(format!(
                "Update failed while replacing binary: {err}. Rollback also failed: {restore_err}. Manual recovery needed from {}.",
                backup_binary.display()
            )));
        }
        return Err(map_update_permission_error(&current_binary, err));
    }

    let _ = fs::remove_file(&backup_binary);
    Ok(())
}

fn map_update_permission_error(target: &Path, err: std::io::Error) -> AppError {
    if err.kind() == std::io::ErrorKind::PermissionDenied {
        return AppError::Message(format!(
            "Permission denied while updating {}. Re-run with elevated privileges (for example: sudo cc-switch update), or use your package manager update command.",
            target.display()
        ));
    }
    AppError::io(target, err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tag_adds_prefix_when_missing() {
        assert_eq!(normalize_tag("4.6.2"), "v4.6.2");
    }

    #[test]
    fn normalize_tag_keeps_existing_prefix() {
        assert_eq!(normalize_tag("v4.6.2"), "v4.6.2");
    }

    #[test]
    fn parse_checksum_for_asset_finds_plain_filename() {
        let checksums = "abc123  cc-switch-cli-linux-x64-musl.tar.gz\n";
        let got = parse_checksum_for_asset(checksums, "cc-switch-cli-linux-x64-musl.tar.gz")
            .expect("checksum should exist");
        assert_eq!(got, "abc123");
    }

    #[test]
    fn parse_checksum_for_asset_supports_star_prefix() {
        let checksums = "def456 *cc-switch-cli-linux-x64-musl.tar.gz\n";
        let got = parse_checksum_for_asset(checksums, "cc-switch-cli-linux-x64-musl.tar.gz")
            .expect("checksum should exist");
        assert_eq!(got, "def456");
    }

    #[test]
    fn validate_target_tag_accepts_normal_value() {
        validate_target_tag("v4.6.3-rc1").expect("valid tag should pass");
    }

    #[test]
    fn validate_target_tag_rejects_path_content() {
        let err = validate_target_tag("v4.6.3/../../evil").expect_err("must reject traversal");
        assert!(err.to_string().contains("forbidden"));
    }
}
