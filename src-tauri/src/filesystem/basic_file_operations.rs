use std::fs;
use std::fs::File;
use std::path::Path;



pub fn create_all_directories(path: &str) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)
}

pub fn create_directory(path: &str) -> Result<(), std::io::Error> {
    fs::create_dir(path)
}

pub fn create_file(filename: &str) -> std::io::Result<File> {
    File::create(Path::new(filename))
}

pub fn delete_all_directories(path: &str) -> Result<(), std::io::Error> {
    fs::remove_dir_all(path)
}

pub fn delete_directory(path: &str) -> Result<(), std::io::Error> {
    fs::remove_dir(path)
}

pub fn delete_file(path: &str) -> Result<(), std::io::Error> {
    fs::remove_file(path)
}


#[cfg(test)]
mod tests {
    use std::{env, fs};
    use std::fs::remove_dir_all;
    use std::path::Path;
    use std::time::Instant;
    use log::{error, info};

    fn exec_with_dir_setup<F, T, E>(dir_name: &str, func: F, parameter: &str) -> Result<T, E>
    where
        F: FnOnce(&str)-> Result<T, E>,
    {
        env_logger::init();
        
        info!("Starting with setup of test environment");

        let mut path = env::current_dir().expect("could not determine current path");
        path.push(dir_name);
        path.push(parameter);
        
        //create the folder for testing
        if !Path::new(dir_name).exists() {
            fs::create_dir_all(dir_name).expect("could not create testing directories");
        }

        //set environment variables if needed

        //------//
        info!("Test setup completed");
        info!("Executing the given function");

        let time_before = Instant::now();

        let result = func(path.to_str().unwrap());

        info!("Execution finished in {}", time_before.elapsed().as_millis());
        info!("Beginning with cleanup");

        match remove_dir_all(dir_name) {
            Ok(_) => info!("Successfully removed {}", dir_name),
            Err(e) => error!("Failed to clean up test directory: {}", e),
        }

        result
    }
    
    mod test_create_files {
        use crate::filesystem::basic_file_operations::create_file;
        use crate::filesystem::basic_file_operations::tests::exec_with_dir_setup;

        #[test]
        fn test_create_file() {
            exec_with_dir_setup("test-directory", |file_name| {create_file(&file_name)}, "test-file.txt").expect("Error during creating test file");
        }
    }
}