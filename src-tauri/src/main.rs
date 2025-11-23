use clap::Parser;
use cc_switch_lib::cli::{Cli, Commands};
use cc_switch_lib::AppError;
use std::process;

fn main() {
    // 解析命令行参数
    let cli = Cli::parse();

    // 初始化日志（交互模式下禁用，避免干扰 UI）
    let log_level = if matches!(cli.command, None | Some(Commands::Interactive)) {
        if cli.verbose {
            "info"
        } else {
            "error" // 交互模式下只显示错误日志
        }
    } else if cli.verbose {
        "debug"
    } else {
        "info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // 执行命令
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        // Default to interactive mode if no command is provided
        None | Some(Commands::Interactive) => cc_switch_lib::cli::interactive::run(cli.app),
        Some(Commands::Provider(cmd)) => {
            cc_switch_lib::cli::commands::provider::execute(cmd, cli.app)
        }
        Some(Commands::Mcp(cmd)) => cc_switch_lib::cli::commands::mcp::execute(cmd, cli.app),
        Some(Commands::Prompts(cmd)) => {
            cc_switch_lib::cli::commands::prompts::execute(cmd, cli.app)
        }
        Some(Commands::Skills(cmd)) => cc_switch_lib::cli::commands::skills::execute(cmd),
        Some(Commands::Config(cmd)) => cc_switch_lib::cli::commands::config::execute(cmd),
        Some(Commands::Env(cmd)) => cc_switch_lib::cli::commands::env::execute(cmd, cli.app),
        Some(Commands::App(cmd)) => cc_switch_lib::cli::commands::app::execute(cmd),
        Some(Commands::Completions { shell }) => {
            cc_switch_lib::cli::generate_completions(shell);
            Ok(())
        }
    }
}
