use std::fmt;
use std::future::Future;

use crate::app_config::AppType;
use crate::cli::i18n::texts;
use crate::cli::ui::{create_table, error, highlight, info, success};
use crate::error::AppError;
use crate::services::skill::{SkillRepo, SkillService as SkillServiceType, SyncMethod};
use crate::services::SkillService;

use super::utils::{
    clear_screen, pause, prompt_confirm, prompt_multiselect, prompt_select, prompt_text,
};

fn run_async<T>(fut: impl Future<Output = Result<T, AppError>>) -> Result<T, AppError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| AppError::Message(format!("Failed to create runtime: {e}")))?
        .block_on(fut)
}

#[derive(Clone)]
struct DiscoverChoice {
    key: String,
    directory: String,
    name: String,
    installed: bool,
}

impl fmt::Display for DiscoverChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.installed { "✓" } else { " " };
        write!(f, "[{marker}] {} — {}", self.directory, self.name)
    }
}

#[derive(Clone)]
struct InstalledChoice {
    directory: String,
    name: String,
    enabled_for_app: bool,
}

impl fmt::Display for InstalledChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.enabled_for_app { "✓" } else { " " };
        write!(f, "[{marker}] {} — {}", self.directory, self.name)
    }
}

pub fn manage_skills_menu(app_type: &AppType) -> Result<(), AppError> {
    loop {
        clear_screen();
        println!("\n{}", highlight(texts::skills_management()));
        println!("{}", "─".repeat(60));

        let installed = SkillService::list_installed()?;
        if installed.is_empty() {
            println!("{}", info(texts::no_skills_installed()));
        } else {
            let mut table = create_table();
            table.set_header(vec!["", "Directory", "Name"]);
            for s in &installed {
                let enabled = s.apps.is_enabled_for(app_type);
                table.add_row(vec![
                    if enabled { "✓" } else { " " }.to_string(),
                    s.directory.clone(),
                    s.name.clone(),
                ]);
            }
            println!("{}", table);
        }

        println!();
        let choices = vec![
            texts::skills_discover(),
            texts::skills_install(),
            texts::skills_uninstall(),
            texts::skills_toggle_for_app(),
            texts::skills_show_info(),
            texts::skills_sync_now(),
            texts::skills_sync_method(),
            texts::skills_scan_unmanaged(),
            texts::skills_import_from_apps(),
            texts::skills_manage_repos(),
            texts::back_to_main(),
        ];

        let Some(choice) = prompt_select(texts::choose_action(), choices)? else {
            break;
        };

        if choice == texts::skills_discover() {
            discover_and_install(app_type)?;
        } else if choice == texts::skills_install() {
            install_by_spec(app_type)?;
        } else if choice == texts::skills_uninstall() {
            uninstall_installed_skill()?;
        } else if choice == texts::skills_toggle_for_app() {
            toggle_for_app(app_type)?;
        } else if choice == texts::skills_show_info() {
            show_installed_skill_info(app_type)?;
        } else if choice == texts::skills_sync_now() {
            sync_now()?;
        } else if choice == texts::skills_sync_method() {
            change_sync_method()?;
        } else if choice == texts::skills_scan_unmanaged() {
            scan_unmanaged()?;
        } else if choice == texts::skills_import_from_apps() {
            import_from_apps_flow()?;
        } else if choice == texts::skills_manage_repos() {
            manage_repos_menu()?;
        } else {
            break;
        }
    }

    Ok(())
}

fn discover_and_install(app_type: &AppType) -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_discover()));
    println!("{}", "─".repeat(60));

    let query = prompt_text(texts::skills_enter_query())?.unwrap_or_default();
    let service = SkillService::new()?;
    let mut skills = run_async(service.list_skills())?;

    let query = query.trim();
    if !query.is_empty() {
        let q = query.to_lowercase();
        skills.retain(|s| {
            s.name.to_lowercase().contains(&q) || s.directory.to_lowercase().contains(&q)
        });
    }

    if skills.is_empty() {
        println!("{}", info("No skills found."));
        pause();
        return Ok(());
    }

    let options: Vec<DiscoverChoice> = skills
        .into_iter()
        .map(|s| DiscoverChoice {
            key: s.key,
            directory: s.directory,
            name: s.name,
            installed: s.installed,
        })
        .collect();

    let Some(choice) = prompt_select(texts::skills_select_skill(), options)? else {
        return Ok(());
    };

    if choice.installed {
        println!("{}", info("Already installed."));
        pause();
        return Ok(());
    }

    let Some(confirm) = prompt_confirm(
        &texts::skills_confirm_install(&choice.directory, app_type.as_str()),
        true,
    )?
    else {
        return Ok(());
    };

    if !confirm {
        println!("{}", info(texts::cancelled()));
        pause();
        return Ok(());
    }

    let service = SkillService::new()?;
    match run_async(service.install(&choice.key, app_type)) {
        Ok(_) => println!("{}", success("✓ Installed.")),
        Err(e) => println!("{}", error(&e.to_string())),
    }
    pause();
    Ok(())
}

fn install_by_spec(app_type: &AppType) -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_install()));
    println!("{}", "─".repeat(60));

    let Some(spec) = prompt_text(texts::skills_enter_install_spec())? else {
        return Ok(());
    };

    let spec = spec.trim();
    if spec.is_empty() {
        println!("{}", info(texts::cancelled()));
        pause();
        return Ok(());
    }

    let service = SkillService::new()?;
    match run_async(service.install(spec, app_type)) {
        Ok(_) => println!("{}", success("✓ Installed.")),
        Err(e) => println!("{}", error(&e.to_string())),
    }
    pause();
    Ok(())
}

fn uninstall_installed_skill() -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_uninstall()));
    println!("{}", "─".repeat(60));

    let installed = SkillService::list_installed()?;
    if installed.is_empty() {
        println!("{}", info(texts::no_skills_installed()));
        pause();
        return Ok(());
    }

    let options: Vec<InstalledChoice> = installed
        .into_iter()
        .map(|s| InstalledChoice {
            directory: s.directory,
            name: s.name,
            enabled_for_app: true,
        })
        .collect();

    let Some(choice) = prompt_select(texts::skills_select_skill(), options)? else {
        return Ok(());
    };

    let Some(confirm) = prompt_confirm(&texts::skills_confirm_uninstall(&choice.directory), false)?
    else {
        return Ok(());
    };

    if !confirm {
        println!("{}", info(texts::cancelled()));
        pause();
        return Ok(());
    }

    match SkillService::uninstall(&choice.directory) {
        Ok(()) => println!("{}", success("✓ Uninstalled.")),
        Err(e) => println!("{}", error(&e.to_string())),
    }
    pause();
    Ok(())
}

fn toggle_for_app(app_type: &AppType) -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_toggle_for_app()));
    println!("{}", "─".repeat(60));

    let installed = SkillService::list_installed()?;
    if installed.is_empty() {
        println!("{}", info(texts::no_skills_installed()));
        pause();
        return Ok(());
    }

    let options: Vec<InstalledChoice> = installed
        .into_iter()
        .map(|s| InstalledChoice {
            directory: s.directory,
            name: s.name,
            enabled_for_app: s.apps.is_enabled_for(app_type),
        })
        .collect();

    let Some(choice) = prompt_select(texts::skills_select_skill(), options)? else {
        return Ok(());
    };

    let target_enabled = !choice.enabled_for_app;
    let Some(confirm) = prompt_confirm(
        &texts::skills_confirm_toggle(&choice.directory, app_type.as_str(), target_enabled),
        true,
    )?
    else {
        return Ok(());
    };
    if !confirm {
        println!("{}", info(texts::cancelled()));
        pause();
        return Ok(());
    }

    match SkillServiceType::toggle_app(&choice.directory, app_type, target_enabled) {
        Ok(()) => println!("{}", success("✓ Updated.")),
        Err(e) => println!("{}", error(&e.to_string())),
    }
    pause();
    Ok(())
}

fn sync_now() -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_sync_now()));
    println!("{}", "─".repeat(60));

    match SkillServiceType::sync_all_enabled(None) {
        Ok(()) => println!("{}", success("✓ Synced.")),
        Err(e) => println!("{}", error(&e.to_string())),
    }
    pause();
    Ok(())
}

fn show_installed_skill_info(app_type: &AppType) -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_show_info()));
    println!("{}", "─".repeat(60));

    let installed = SkillService::list_installed()?;
    if installed.is_empty() {
        println!("{}", info(texts::no_skills_installed()));
        pause();
        return Ok(());
    }

    #[derive(Clone)]
    struct InfoChoice {
        directory: String,
        name: String,
        apps: crate::services::skill::SkillApps,
        description: Option<String>,
        id: String,
        repo_owner: Option<String>,
        repo_name: Option<String>,
        repo_branch: Option<String>,
        readme_url: Option<String>,
    }

    impl fmt::Display for InfoChoice {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} — {}", self.directory, self.name)
        }
    }

    let options: Vec<InfoChoice> = installed
        .into_iter()
        .map(|s| InfoChoice {
            directory: s.directory,
            name: s.name,
            apps: s.apps,
            description: s.description,
            id: s.id,
            repo_owner: s.repo_owner,
            repo_name: s.repo_name,
            repo_branch: s.repo_branch,
            readme_url: s.readme_url,
        })
        .collect();

    let Some(choice) = prompt_select(texts::skills_select_skill(), options)? else {
        return Ok(());
    };

    println!("{}", highlight(&choice.name));
    println!("Directory: {}", choice.directory);
    println!(
        "Enabled:   {}={} {}={} {}={}",
        "claude", choice.apps.claude, "codex", choice.apps.codex, "gemini", choice.apps.gemini
    );
    if let Some(desc) = choice
        .description
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        println!("Desc:      {}", desc);
    }

    if let (Some(owner), Some(name)) = (choice.repo_owner.as_deref(), choice.repo_name.as_deref()) {
        let branch = choice.repo_branch.as_deref().unwrap_or("main");
        println!("Repo:      {owner}/{name}@{branch}");
    } else {
        println!("Repo:      local");
    }

    if let Some(url) = choice
        .readme_url
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        println!("Readme:    {url}");
    }

    println!();
    println!(
        "{}",
        info(&texts::skills_current_app_note(app_type.as_str()))
    );
    pause();
    Ok(())
}

fn change_sync_method() -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_sync_method()));
    println!("{}", "─".repeat(60));

    let current = SkillServiceType::get_sync_method()?;
    println!(
        "{}",
        info(&texts::skills_current_sync_method(&format!("{current:?}")))
    );
    println!();

    #[derive(Clone)]
    struct SyncMethodChoice {
        method: SyncMethod,
    }

    impl fmt::Display for SyncMethodChoice {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let label = match self.method {
                SyncMethod::Auto => "auto (symlink → copy)",
                SyncMethod::Symlink => "symlink",
                SyncMethod::Copy => "copy",
            };
            write!(f, "{label}")
        }
    }

    let options = vec![
        SyncMethodChoice {
            method: SyncMethod::Auto,
        },
        SyncMethodChoice {
            method: SyncMethod::Symlink,
        },
        SyncMethodChoice {
            method: SyncMethod::Copy,
        },
    ];

    let Some(selected) = prompt_select(texts::skills_select_sync_method(), options)? else {
        return Ok(());
    };

    SkillServiceType::set_sync_method(selected.method)?;
    println!("{}", success("✓ Updated."));
    pause();
    Ok(())
}

fn scan_unmanaged() -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_scan_unmanaged()));
    println!("{}", "─".repeat(60));

    let unmanaged = SkillServiceType::scan_unmanaged()?;
    if unmanaged.is_empty() {
        println!("{}", info(texts::skills_no_unmanaged_found()));
        pause();
        return Ok(());
    }

    let mut table = create_table();
    table.set_header(vec!["Directory", "Found In", "Name"]);
    for s in &unmanaged {
        table.add_row(vec![
            s.directory.clone(),
            s.found_in.join(", "),
            s.name.clone(),
        ]);
    }
    println!("{}", table);
    pause();
    Ok(())
}

fn import_from_apps_flow() -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_import_from_apps()));
    println!("{}", "─".repeat(60));

    let unmanaged = SkillServiceType::scan_unmanaged()?;
    if unmanaged.is_empty() {
        println!("{}", info(texts::skills_no_unmanaged_found()));
        pause();
        return Ok(());
    }

    let options: Vec<String> = unmanaged.into_iter().map(|s| s.directory).collect();
    let Some(selected) = prompt_multiselect(texts::skills_select_unmanaged_to_import(), options)?
    else {
        return Ok(());
    };

    if selected.is_empty() {
        println!("{}", info(texts::cancelled()));
        pause();
        return Ok(());
    }

    match SkillServiceType::import_from_apps(selected) {
        Ok(imported) => {
            println!(
                "{}",
                success(&format!("✓ Imported {} skill(s).", imported.len()))
            );
        }
        Err(e) => println!("{}", error(&e.to_string())),
    }

    pause();
    Ok(())
}

fn manage_repos_menu() -> Result<(), AppError> {
    loop {
        clear_screen();
        println!("\n{}", highlight(texts::skills_repos_management()));
        println!("{}", "─".repeat(60));

        let choices = vec![
            texts::skills_repo_list(),
            texts::skills_repo_add(),
            texts::skills_repo_remove(),
            texts::back_to_main(),
        ];

        let Some(choice) = prompt_select(texts::choose_action(), choices)? else {
            break;
        };

        if choice == texts::skills_repo_list() {
            show_repos()?;
        } else if choice == texts::skills_repo_add() {
            add_repo()?;
        } else if choice == texts::skills_repo_remove() {
            remove_repo()?;
        } else {
            break;
        }
    }

    Ok(())
}

fn show_repos() -> Result<(), AppError> {
    clear_screen();
    println!("\n{}", highlight(texts::skills_repo_list()));
    println!("{}", "─".repeat(60));

    let repos = SkillServiceType::list_repos()?;
    if repos.is_empty() {
        println!("{}", info("No repos configured."));
        pause();
        return Ok(());
    }

    let mut table = create_table();
    table.set_header(vec!["Enabled", "Repo", "Branch"]);
    for r in repos {
        table.add_row(vec![
            if r.enabled { "✓" } else { " " }.to_string(),
            format!("{}/{}", r.owner, r.name),
            r.branch,
        ]);
    }
    println!("{}", table);
    pause();
    Ok(())
}

fn add_repo() -> Result<(), AppError> {
    let Some(raw) = prompt_text(texts::skills_repo_enter_spec())? else {
        return Ok(());
    };
    let repo = parse_repo_spec(&raw)?;
    SkillServiceType::upsert_repo(repo)?;
    println!("{}", success("✓ Repo added."));
    pause();
    Ok(())
}

fn remove_repo() -> Result<(), AppError> {
    let repos = SkillServiceType::list_repos()?;
    if repos.is_empty() {
        println!("{}", info("No repos configured."));
        pause();
        return Ok(());
    }

    #[derive(Clone)]
    struct RepoChoice {
        owner: String,
        name: String,
        branch: String,
    }
    impl fmt::Display for RepoChoice {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}/{}@{}", self.owner, self.name, self.branch)
        }
    }

    let options: Vec<RepoChoice> = repos
        .into_iter()
        .map(|r| RepoChoice {
            owner: r.owner,
            name: r.name,
            branch: r.branch,
        })
        .collect();

    let Some(choice) = prompt_select(texts::skills_repo_remove(), options)? else {
        return Ok(());
    };
    SkillServiceType::remove_repo(&choice.owner, &choice.name)?;
    println!("{}", success("✓ Repo removed."));
    pause();
    Ok(())
}

fn parse_repo_spec(raw: &str) -> Result<SkillRepo, AppError> {
    let raw = raw.trim().trim_end_matches('/');
    if raw.is_empty() {
        return Err(AppError::InvalidInput(
            "Repository cannot be empty".to_string(),
        ));
    }

    let without_prefix = raw
        .strip_prefix("https://github.com/")
        .or_else(|| raw.strip_prefix("http://github.com/"))
        .unwrap_or(raw);
    let without_git = without_prefix.trim_end_matches(".git");

    let (path, branch) = if let Some((left, right)) = without_git.rsplit_once('@') {
        (left, Some(right))
    } else {
        (without_git, None)
    };

    let Some((owner, name)) = path.split_once('/') else {
        return Err(AppError::InvalidInput(
            "Invalid repo format. Use owner/name or https://github.com/owner/name".to_string(),
        ));
    };

    Ok(SkillRepo {
        owner: owner.to_string(),
        name: name.to_string(),
        branch: branch.unwrap_or("main").to_string(),
        enabled: true,
    })
}
