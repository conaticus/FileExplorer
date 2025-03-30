use std::path::PathBuf;

pub struct SelectedFileForAction {
    pub abs_file_path: Option<PathBuf>,
    pub time_of_selection: Option<u64>,
}

impl SelectedFileForAction {
    pub fn default() -> Self {
        SelectedFileForAction {
            abs_file_path: None,
            time_of_selection: None,
        }
    }
}