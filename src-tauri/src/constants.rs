use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

pub static VERSION: &str = "0.1.0";

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    env::current_dir()
        .expect("Could not determine current path")
        .join("config")
});

pub static META_DATA_CONFIG_ABS_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_PATH.join(META_DATA_CONFIG_FILE_NAME));

pub static META_DATA_CONFIG_FILE_NAME: &str = "meta_data.json";
