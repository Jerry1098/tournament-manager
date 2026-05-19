use std::path::PathBuf;
use std::sync::Mutex;
use crate::domain::model::Tournament;

pub struct AppState {
    pub current: Option<Tournament>,
    pub current_path: Option<PathBuf>,
    pub dirty: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState { current: None, current_path: None, dirty: false }
    }
}

pub type SharedState = Mutex<AppState>;
