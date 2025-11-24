use clap::Subcommand;
use crate::app_config::AppType;
use crate::error::AppError;
use crate::cli::ui::{create_table, success, error, highlight, info};
use crate::services::env_checker;
use crate::services::env_manager;

#[derive(Subcommand)]
pub enum EnvCommand {
    /// Check for environment variable conflicts
    Check,
    /// List all relevant environment variables
    List,
    /// Set an environment variable
    Set {
        /// Variable name
        key: String,
        /// Variable value
        value: String,
    },
    /// Unset an environment variable
    Unset {
        /// Variable name
        key: String,
    },
}

pub fn execute(cmd: EnvCommand, app: Option<AppType>) -> Result<(), AppError> {
    let app_type = app.unwrap_or(AppType::Claude);

    match cmd {
        EnvCommand::Check => check_conflicts(app_type),
        EnvCommand::List => list_env_vars(app_type),
        EnvCommand::Set { key, value } => set_env_var(app_type, &key, &value),
        EnvCommand::Unset { key } => unset_env_var(app_type, &key),
    }
}

fn check_conflicts(app_type: AppType) -> Result<(), AppError> {
    let app_str = app_type.as_str();

    println!("\n{}", highlight(&format!("Checking Environment Variables for {}", app_str)));
    println!("{}", "═".repeat(60));

    // 检测冲突
    let conflicts = env_checker::check_env_conflicts(app_str)
        .map_err(|e| AppError::Message(format!("Failed to check environment variables: {}", e)))?;

    if conflicts.is_empty() {
        println!("\n{}", success("✓ No environment variable conflicts detected"));
        println!("{}", info(&format!("Your {} configuration should work correctly.", app_str)));
        return Ok(());
    }

    // 显示冲突
    println!("\n{}", error(&format!("⚠ Found {} environment variable(s) that may conflict:", conflicts.len())));
    println!();

    let mut table = create_table();
    table.set_header(vec!["Variable", "Value", "Source Type", "Source Location"]);

    for conflict in &conflicts {
        // 截断过长的值
        let value_display = if conflict.var_value.len() > 30 {
            format!("{}...", &conflict.var_value[..27])
        } else {
            conflict.var_value.clone()
        };

        table.add_row(vec![
            conflict.var_name.as_str(),
            &value_display,
            conflict.source_type.as_str(),
            conflict.source_path.as_str(),
        ]);
    }

    println!("{}", table);
    println!();
    println!("{}", info("These environment variables may override CC-Switch's configuration."));
    println!("{}", info("Use 'cc-switch env unset <VAR>' to remove them (automatic backup included)."));

    Ok(())
}

fn list_env_vars(app_type: AppType) -> Result<(), AppError> {
    let app_str = app_type.as_str();

    println!("\n{}", highlight(&format!("Environment Variables for {}", app_str)));
    println!("{}", "═".repeat(60));

    // 获取所有相关环境变量
    let conflicts = env_checker::check_env_conflicts(app_str)
        .map_err(|e| AppError::Message(format!("Failed to list environment variables: {}", e)))?;

    if conflicts.is_empty() {
        println!("\n{}", info("No related environment variables found."));
        return Ok(());
    }

    println!("\n{} environment variable(s) found:\n", conflicts.len());

    let mut table = create_table();
    table.set_header(vec!["Variable", "Value", "Source Type", "Source Location"]);

    for conflict in &conflicts {
        table.add_row(vec![
            conflict.var_name.as_str(),
            conflict.var_value.as_str(),
            conflict.source_type.as_str(),
            conflict.source_path.as_str(),
        ]);
    }

    println!("{}", table);

    Ok(())
}

fn set_env_var(app_type: AppType, key: &str, value: &str) -> Result<(), AppError> {
    let app_str = app_type.as_str();

    println!("\n{}", highlight(&format!("Setting Environment Variable for {}", app_str)));
    println!("{}", "═".repeat(60));

    #[cfg(target_os = "windows")]
    {
        println!("\n{}", info("Setting environment variables on Windows requires registry access."));
        println!("{}", error("This feature is not yet fully implemented."));
        println!();
        println!("{}", info("Please set the environment variable manually:"));
        println!("  1. Open System Properties → Environment Variables");
        println!("  2. Add new variable: {} = {}", key, value);
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("\n{}", info("To set an environment variable, add it to your shell configuration:"));
        println!();

        // 检测当前 shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let config_file = if shell.contains("zsh") {
            "~/.zshrc"
        } else if shell.contains("fish") {
            "~/.config/fish/config.fish"
        } else {
            "~/.bashrc"
        };

        println!("{}", highlight(&format!("Add this line to {}:", config_file)));
        println!();
        println!("  export {}=\"{}\"", key, value);
        println!();
        println!("{}", info("Then restart your terminal or run:"));
        println!("  source {}", config_file);

        return Ok(());
    }
}

fn unset_env_var(app_type: AppType, key: &str) -> Result<(), AppError> {
    let app_str = app_type.as_str();

    println!("\n{}", highlight(&format!("Removing Environment Variable for {}", app_str)));
    println!("{}", "═".repeat(60));

    // 首先检查变量是否存在
    let all_conflicts = env_checker::check_env_conflicts(app_str)
        .map_err(|e| AppError::Message(format!("Failed to check environment variables: {}", e)))?;

    let to_delete: Vec<_> = all_conflicts
        .into_iter()
        .filter(|c| c.var_name == key)
        .collect();

    if to_delete.is_empty() {
        println!("\n{}", error(&format!("Environment variable '{}' not found.", key)));
        println!("{}", info("Use 'cc-switch env list' to see all variables."));
        return Ok(());
    }

    // 显示将要删除的变量
    println!("\n{}", info("The following will be removed:"));
    println!();
    for conflict in &to_delete {
        println!("  • {} = {}", conflict.var_name, conflict.var_value);
        println!("    Source: {}", conflict.source_path);
    }
    println!();

    // 执行删除（会自动创建备份）
    let backup_info = env_manager::delete_env_vars(to_delete)
        .map_err(|e| AppError::Message(format!("Failed to delete environment variable: {}", e)))?;

    println!("{}", success(&format!("✓ Environment variable '{}' removed successfully", key)));
    println!();
    println!("{}", info("Backup created at:"));
    println!("  {}", backup_info.backup_path);
    println!();
    println!("{}", info("Restart your terminal for changes to take effect."));

    Ok(())
}
