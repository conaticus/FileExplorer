use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct File {
    name: String,
    size_in_byte: u32,
    json_meta_data: String,
}

impl File {
    fn new(name: &str, size: u32, json_meta_data: &str) -> Self {
        File {
            name: name.to_string(),
            size_in_byte: size,
            json_meta_data: json_meta_data.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Directory {
    name: String,
    entries: HashMap<String, FsEntry>,
}

impl Directory {
    fn new(name: &str) -> Self {
        Directory {
            name: name.to_string(),
            entries: HashMap::new(),
        }
    }

    fn add_entry(&mut self, entry: FsEntry) {
        self.entries.insert(entry.name().parse().unwrap(), entry);
    }
}

#[derive(Debug, Clone)]
pub enum FsEntry {
    File(File),
    Directory(Directory),
}

impl FsEntry {
    fn name(&self) -> &str {
        match self {
            FsEntry::File(file) => &file.name,
            FsEntry::Directory(dir) => &dir.name,
        }
    }

    fn size(&self) -> u32 {
        match self {
            FsEntry::File(file) => file.size_in_byte,
            FsEntry::Directory(dir) => dir.entries.values().map(|e| e.size()).sum(),
        }
    }
}

#[test]
fn test_that_shit() {
    // Create files
    let file1 = File::new("file1.txt", 100, "{\"author\": \"user1\"}");
    let file2 = File::new("file2.txt", 200, "{\"author\": \"user2\"}");

    // Create directories
    let mut dir1 = Directory::new("dir1");
    let mut subdir = Directory::new("subdir");

    // Add files to the directory
    dir1.add_entry(FsEntry::File(file1.clone()));
    dir1.add_entry(FsEntry::File(file2.clone()));

    // Add file to subdirectory
    subdir.add_entry(FsEntry::File(File::new(
        "file3.txt",
        50,
        "{\"author\": \"user3\"}",
    )));

    // Add subdirectory to the main directory
    dir1.add_entry(FsEntry::Directory(subdir));

    //TODO: Implement logging

    // Print out the directory structure
    //println!("{:?}", dir1);

    // Print the total size_in_byte of the directory
    //println!("Total size_in_byte of 'dir1': {}", dir1.entries.values().map(|e| e.size()).sum::<u32>());
}

#[test]
fn entry_option_file_creation_test() {
    let file1 = File::new("file1.txt", 100, "{}");
    let file2 = File::new("file2.txt", 200, "{}");

    let mut root_dir = Directory::new("root_dir");

    root_dir.add_entry(FsEntry::File(file1.clone()));
}
