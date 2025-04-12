use serde::{Deserialize, Serialize};
use crate::filesystem::models;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Entries {
    pub(crate) directories: Vec<models::Directory>,
    pub(crate) files: Vec<models::File>,
}