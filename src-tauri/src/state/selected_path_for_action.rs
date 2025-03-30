use std::path::PathBuf;

pub struct SelectedPathForAction {
    pub abs_file_path: Option<PathBuf>,
    pub time_of_selection: Option<u64>,
}

impl SelectedPathForAction {
    pub fn default() -> Self {
        SelectedPathForAction {
            abs_file_path: None,
            time_of_selection: None,
        }
    }
}