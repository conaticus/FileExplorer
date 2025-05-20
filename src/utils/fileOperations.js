import { invoke } from '@tauri-apps/api/core';

/**
 * Open a file with the default application.
 * @param {string} filePath - The path to the file to open.
 * @returns {Promise<void>}
 */
export const openFile = async (filePath) => {
    return invoke('open_file', { file_path: filePath });
};

/**
 * Create a new file in the specified folder.
 * @param {string} folderPath - The absolute path to the folder where the file will be created.
 * @param {string} fileName - The name of the file to create.
 * @returns {Promise<void>}
 */
export const createFile = async (folderPath, fileName) => {
    return invoke('create_file', {
        folder_path_abs: folderPath,
        file_name: fileName
    });
};

/**
 * Create a new directory in the specified folder.
 * @param {string} folderPath - The absolute path to the folder where the directory will be created.
 * @param {string} directoryName - The name of the directory to create.
 * @returns {Promise<void>}
 */
export const createDirectory = async (folderPath, directoryName) => {
    return invoke('create_directory', {
        folder_path_abs: folderPath,
        directory_name: directoryName
    });
};

/**
 * Rename a file or directory.
 * @param {string} oldPath - The current absolute path of the file or directory.
 * @param {string} newPath - The new absolute path for the file or directory.
 * @returns {Promise<void>}
 */
export const renameItem = async (oldPath, newPath) => {
    return invoke('rename', { old_path: oldPath, new_path: newPath });
};

/**
 * Move a file or directory to the trash.
 * @param {string} path - The absolute path to the file or directory to move to trash.
 * @returns {Promise<void>}
 */
export const moveToTrash = async (path) => {
    return invoke('move_to_trash', { path });
};

/**
 * Load the contents of a directory.
 * @param {string} path - The absolute path to the directory to load.
 * @returns {Promise<Object>} - An object containing directories and files.
 */
export const loadDirectory = async (path) => {
    const dirContent = await invoke('open_directory', { path });
    return JSON.parse(dirContent);
};

/**
 * Get system volumes information.
 * @returns {Promise<Array>} - An array of volume objects.
 */
export const getSystemVolumes = async () => {
    const volumesJson = await invoke('get_system_volumes_information_as_json');
    return JSON.parse(volumesJson);
};

/**
 * Zip one or more files or directories.
 * @param {Array<string>} sourcePaths - Array of paths to files/directories to zip.
 * @param {string|null} destinationPath - Optional destination path for the zip file.
 * @returns {Promise<void>}
 */
export const zipItems = async (sourcePaths, destinationPath = null) => {
    return invoke('zip', {
        source_paths: sourcePaths,
        destination_path: destinationPath
    });
};

/**
 * Extract a zip file.
 * @param {string} zipPath - Path to the zip file to extract.
 * @param {string|null} destinationPath - Optional destination path for extraction.
 * @returns {Promise<void>}
 */
export const unzipItem = async (zipPath, destinationPath = null) => {
    return invoke('unzip', {
        zip_paths: [zipPath],
        destination_path: destinationPath
    });
};

/**
 * Get available templates.
 * @returns {Promise<Array<string>>} - An array of template paths.
 */
export const getTemplatePaths = async () => {
    const templatesJson = await invoke('get_template_paths_as_json');
    return JSON.parse(templatesJson);
};

/**
 * Add a template.
 * @param {string} templatePath - Path to the file or directory to add as a template.
 * @returns {Promise<string>} - Success message.
 */
export const addTemplate = async (templatePath) => {
    return invoke('add_template', { template_path: templatePath });
};

/**
 * Use a template.
 * @param {string} templatePath - Path to the template to use.
 * @param {string} destPath - Destination path where to apply the template.
 * @returns {Promise<string>} - Success message.
 */
export const useTemplate = async (templatePath, destPath) => {
    return invoke('use_template', {
        template_path: templatePath,
        dest_path: destPath
    });
};

/**
 * Remove a template.
 * @param {string} templatePath - Path to the template to remove.
 * @returns {Promise<string>} - Success message.
 */
export const removeTemplate = async (templatePath) => {
    return invoke('remove_template', { template_path: templatePath });
};

/**
 * Generate a hash for a file.
 * @param {string} path - Path to the file to hash.
 * @returns {Promise<string>} - The generated hash.
 */
export const generateHash = async (path) => {
    return invoke('gen_hash_and_return_string', { path });
};

/**
 * Check if the clipboard has file paths.
 * @returns {Promise<boolean>} - Whether the clipboard has file paths.
 */
export const hasClipboardFiles = async () => {
    // This is a mock since we don't have a direct Tauri API for this
    // In a real implementation, this would check the system clipboard
    return Promise.resolve(false);
};

/**
 * Copy files to the clipboard.
 * @param {Array<string>} paths - Array of file/directory paths to copy.
 * @returns {Promise<void>}
 */
export const copyFilesToClipboard = async (paths) => {
    // This is a mock since we don't have a direct Tauri API for this
    // In a real implementation, this would interact with the system clipboard
    console.log('Copied to clipboard:', paths);
    return Promise.resolve();
};

/**
 * Cut files to the clipboard.
 * @param {Array<string>} paths - Array of file/directory paths to cut.
 * @returns {Promise<void>}
 */
export const cutFilesToClipboard = async (paths) => {
    // This is a mock since we don't have a direct Tauri API for this
    // In a real implementation, this would interact with the system clipboard
    console.log('Cut to clipboard:', paths);
    return Promise.resolve();
};

/**
 * Paste files from the clipboard.
 * @param {string} destinationPath - Destination directory path.
 * @returns {Promise<void>}
 */
export const pasteFilesFromClipboard = async (destinationPath) => {
    // This is a mock since we don't have a direct Tauri API for this
    // In a real implementation, this would interact with the system clipboard
    console.log('Pasting to:', destinationPath);
    return Promise.resolve();
};