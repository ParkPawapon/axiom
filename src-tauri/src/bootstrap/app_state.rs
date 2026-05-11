#[derive(Debug, Clone)]
pub struct AppState {
    pub app_name: &'static str,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            app_name: "AxiomPHP",
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
