use crate::config::get_app_config_path;
use crate::error::AppError;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Export `~/.cc-switch/config.json` to the given file path.
///
/// This mirrors the upstream Tauri command signature style (`Result<Value, String>`)
/// while keeping the CLI project JSON SSOT model.
pub async fn export_config_to_file(file_path: String) -> Result<Value, String> {
    let source_path = get_app_config_path();
    let target_path = PathBuf::from(&file_path);

    let Some(parent) = target_path.parent() else {
        return Err(
            AppError::InvalidInput(format!("Invalid export path: {file_path}")).to_string(),
        );
    };
    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e).to_string())?;
    }

    let bytes = std::fs::read(&source_path).map_err(|e| AppError::io(&source_path, e).to_string())?;
    std::fs::write(&target_path, bytes).map_err(|e| AppError::io(&target_path, e).to_string())?;

    Ok(json!({
        "success": true,
        "message": "Config exported successfully",
        "filePath": file_path
    }))
}

