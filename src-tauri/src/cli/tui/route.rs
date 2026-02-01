#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Main,
    Providers,
    ProviderDetail { id: String },
    Mcp,
    Prompts,
    Config,
    Skills,
    SkillsDiscover,
    SkillsRepos,
    SkillsUnmanaged,
    SkillDetail { directory: String },
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Main,
    Providers,
    Mcp,
    Prompts,
    Config,
    Skills,
    Settings,
    Exit,
}

impl NavItem {
    pub const ALL: [NavItem; 8] = [
        NavItem::Main,
        NavItem::Providers,
        NavItem::Mcp,
        NavItem::Prompts,
        NavItem::Config,
        NavItem::Skills,
        NavItem::Settings,
        NavItem::Exit,
    ];

    pub fn to_route(self) -> Option<Route> {
        match self {
            NavItem::Main => Some(Route::Main),
            NavItem::Providers => Some(Route::Providers),
            NavItem::Mcp => Some(Route::Mcp),
            NavItem::Prompts => Some(Route::Prompts),
            NavItem::Config => Some(Route::Config),
            NavItem::Skills => Some(Route::Skills),
            NavItem::Settings => Some(Route::Settings),
            NavItem::Exit => None,
        }
    }
}
