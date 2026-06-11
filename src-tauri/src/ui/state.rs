#[derive(Clone, Debug, PartialEq)]
pub struct UiState {
    pub current_page: Page,
    pub sidebar_collapsed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Page {
    Dashboard,
    Connections,
    Terminal,
    Settings,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_page: Page::Dashboard,
            sidebar_collapsed: false,
        }
    }
}
