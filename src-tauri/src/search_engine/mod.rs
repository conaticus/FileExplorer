mod art_v4;
pub mod autocomplete_engine;
mod fast_fuzzy_v2;
mod lru_cache_v2;
mod path_cache_wrapper;

#[cfg(test)]
pub mod test_generate_test_data {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use crate::log_info;

    /// Generates a test data directory structure with random folder and file names.
    /// This function creates a hierarchical directory structure with random file and folder names
    /// for testing purposes. It creates a specified number of folders per level, with files
    /// in each folder, up to a maximum depth.
    ///
    /// # Arguments
    /// * `base_path` - A PathBuf that specifies the root directory where the test data will be created.
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - The path to the created test data directory if successful.
    /// * `Err(std::io::Error)` - If there was an error during directory or file creation.
    ///
    /// # Example
    /// ```rust
    /// use std::path::PathBuf;
    /// use crate::search_engine::test_generate_test_data::generate_test_data;
    ///
    /// let test_dir = PathBuf::from("/path/to/test_data");
    /// match generate_test_data(test_dir) {
    ///     Ok(path) => println!("Test data created at: {:?}", path),
    ///     Err(err) => println!("Failed to create test data: {}", err),
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn generate_test_data(base_path: PathBuf) -> Result<PathBuf, std::io::Error> {
        use rand::{thread_rng, Rng};
        use std::fs::{create_dir_all, File};
        use std::time::Instant;

        // Constants for the directory structure
        const FOLDERS_PER_LEVEL: usize = 20;
        const FILES_PER_FOLDER: usize = 20;
        const MAX_DEPTH: usize = 3;

        // Remove the directory if it already exists
        if base_path.exists() {
            log_info!("Removing existing test data at: {:?}", base_path);
            std::fs::remove_dir_all(&base_path)?;
        }

        // Create the base directory
        create_dir_all(&base_path)?;
        log_info!("Creating test data at: {:?}", base_path);

        let start_time = Instant::now();

        // Function to generate random strings based on a predefined set
        let generate_random_name = || -> String {
            let charset: Vec<&str> =
                "banana,apple,orange,grape,watermelon,kiwi,mango,peach,cherry,\
        strawberry,blueberry,raspberry,blackberry,lemon,lime,coconut,papaya,pineapple,tangerine,\
        car,truck,motorcycle,bicycle,bus,train,airplane,helicopter,boat,ship,submarine,scooter,van,\
        ambulance,taxi,firetruck,tractor,yacht,jetski,speedboat,racecar"
                    .split(",")
                    .collect::<Vec<_>>();

            let mut rng = thread_rng();

            let idx = rng.gen_range(0, charset.len());
            return charset[idx].to_string();
        };

        // Function to create file extensions
        let generate_extension = || -> &str {
            const EXTENSIONS: [&str; 20] = [
                "txt", "pdf", "doc", "jpg", "png", "mp3", "mp4", "html", "css", "js", "rs", "json",
                "xml", "md", "csv", "zip", "exe", "dll", "sh", "py",
            ];

            let mut rng = thread_rng();
            let idx = rng.gen_range(0, EXTENSIONS.len());
            EXTENSIONS[idx]
        };

        // Counter to track progress
        let entry_count = Arc::new(Mutex::new(0usize));

        // Recursive function to create the folder structure
        fn create_structure(
            path: &PathBuf,
            depth: usize,
            max_depth: usize,
            folders_per_level: usize,
            files_per_folder: usize,
            name_generator: &dyn Fn() -> String,
            ext_generator: &dyn Fn() -> &'static str,
            counter: &Arc<Mutex<usize>>,
        ) -> Result<(), std::io::Error> {
            // Create files in current folder
            for _ in 0..files_per_folder {
                let file_name = format!("{}.{}", name_generator(), ext_generator());
                let file_path = path.join(file_name);
                File::create(file_path)?;

                // Increment counter
                if let Ok(mut count) = counter.lock() {
                    *count += 1;
                    if *count % 1000 == 0 {
                        log_info!("Created {} entries so far...", *count);
                    }
                }
            }

            // Stop creating subfolders if we've reached max depth
            if depth >= max_depth {
                return Ok(());
            }

            // Create subfolders and recurse
            for _ in 0..folders_per_level {
                let folder_name = name_generator();
                let folder_path = path.join(folder_name);
                create_dir_all(&folder_path)?;

                // Increment counter for folder
                if let Ok(mut count) = counter.lock() {
                    *count += 1;
                }

                // Recurse into subfolder
                create_structure(
                    &folder_path,
                    depth + 1,
                    max_depth,
                    folders_per_level,
                    files_per_folder,
                    name_generator,
                    ext_generator,
                    counter,
                )?;
            }

            Ok(())
        }

        // Start the recursive creation
        create_structure(
            &base_path,
            0,
            MAX_DEPTH,
            FOLDERS_PER_LEVEL,
            FILES_PER_FOLDER,
            &generate_random_name,
            &generate_extension,
            &entry_count,
        )?;

        let total_count = *entry_count.lock().unwrap();
        let elapsed = start_time.elapsed();

        log_info!("Test data generation complete!");
        log_info!("Created {} total entries in {:?}", total_count, elapsed);
        log_info!("Path: {:?}", base_path);

        Ok(base_path)
    }
}
