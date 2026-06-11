pub struct Theme {
    pub primary_color: &'static str,
    pub background_color: &'static str,
    pub text_color: &'static str,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: "#3b82f6",
            background_color: "#1e1e2e",
            text_color: "#cdd6f4",
        }
    }
}
