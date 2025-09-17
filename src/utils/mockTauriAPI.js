/**
 * Mock implementation of Tauri API for development without Tauri
 */

// Mock data for volumes
const MOCK_VOLUMES = [
    {
        volume_name: "Local Disk (C:)",
        mount_point: "C:\\",
        file_system: "NTFS",
        size: 500107862016,
        available_space: 158148874240,
        is_removable: false,
    },
    {
        volume_name: "Data (D:)",
        mount_point: "D:\\",
        file_system: "NTFS",
        size: 1000215724032,
        available_space: 350748937216,
        is_removable: false,
    }
];

// Mock data for directories
const MOCK_DIRECTORIES = {
    "C:\\": {
        directories: [
            {
                name: "Users",
                path: "C:\\Users",
                is_symlink: false,
                access_rights_as_string: "rwxr-xr-x",
                access_rights_as_number: 16877,
                size_in_bytes: 4096,
                sub_file_count: 5,
                sub_dir_count: 3,
                created: "2023-01-15 10:30:00",
                last_modified: "2023-05-20 15:45:00",
                accessed: "2023-05-25 08:20:00"
            },
            {
                name: "Program Files",
                path: "C:\\Program Files",
                is_symlink: false,
                access_rights_as_string: "rwxr-xr-x",
                access_rights_as_number: 16877,
                size_in_bytes: 4096,
                sub_file_count: 12,
                sub_dir_count: 8,
                created: "2023-01-15 10:30:00",
                last_modified: "2023-05-18 14:22:00",
                accessed: "2023-05-24 11:15:00"
            }
        ],
        files: [
            {
                name: "pagefile.sys",
                path: "C:\\pagefile.sys",
                is_symlink: false,
                access_rights_as_string: "rw-r--r--",
                access_rights_as_number: 33188,
                size_in_bytes: 16777216,
                created: "2023-01-15 10:30:00",
                last_modified: "2023-05-25 08:00:00",
                accessed: "2023-05-25 08:00:00"
            }
        ]
    },
    "C:\\Users": {
        directories: [
            {
                name: "Public",
                path: "C:\\Users\\Public",
                is_symlink: false,
                access_rights_as_string: "rwxr-xr-x",
                access_rights_as_number: 16877,
                size_in_bytes: 4096,
                sub_file_count: 3,
                sub_dir_count: 5,
                created: "2023-01-15 10:30:00",
                last_modified: "2023-05-19 09:10:00",
                accessed: "2023-05-23 16:45:00"
            },
            {
                name: "User",
                path: "C:\\Users\\User",
                is_symlink: false,
                access_rights_as_string: "rwxr-xr-x",
                access_rights_as_number: 16877,
                size_in_bytes: 4096,
                sub_file_count: 8,
                sub_dir_count: 12,
                created: "2023-01-15 10:30:00",
                last_modified: "2023-05-25 10:05:00",
                accessed: "2023-05-25 10:05:00"
            }
        ],
        files: []
    }
};

// Mock settings
const MOCK_SETTINGS = {
    theme: 'light',
    defaultView: 'grid',
    showHiddenFiles: false,
    sortBy: 'name',
    sortDirection: 'asc',
    showDetailsPanel: false,
    terminalHeight: 240,
};

// Mock templates
const MOCK_TEMPLATES = [
    {
        name: 'Project Template',
        path: '/templates/project-template',
        type: 'folder',
        size: 2048,
        createdAt: '2023-04-15'
    },
    {
        name: 'Document Template.docx',
        path: '/templates/document-template.docx',
        type: 'file',
        size: 1024,
        createdAt: '2023-03-20'
    }
];

/**
 * Mock implementation of Tauri's invoke function
 * @param {string} command - The command to invoke
 * @param {Object} params - The parameters for the command
 * @returns {Promise<any>} - The result of the command
 */
export const mockInvoke = async (command, params) => {
    console.log(`Mock Tauri invoke: ${command}`, params);

    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 300));

    switch (command) {
        case 'get_system_volumes_information_as_json':
            return JSON.stringify(MOCK_VOLUMES);

        case 'open_directory':
            const path = params.path;

            // Normalize path for matching with mock data
            const normalizedPath = path.replace(/\//g, '\\').replace(/\\+$/, '');

            if (MOCK_DIRECTORIES[normalizedPath]) {
                return JSON.stringify(MOCK_DIRECTORIES[normalizedPath]);
            }

            // Create empty directory if no mock data exists
            return JSON.stringify({
                directories: [],
                files: []
            });

        case 'get_settings_as_json':
            return JSON.stringify(MOCK_SETTINGS);

        case 'get_setting_field':
            return MOCK_SETTINGS[params.key] || null;

        case 'update_settings_field':
            // Update mock settings
            MOCK_SETTINGS[params.key] = params.value;
            return JSON.stringify(MOCK_SETTINGS);

        case 'update_multiple_settings_command':
            // Update multiple settings
            Object.assign(MOCK_SETTINGS, params.updates);
            return JSON.stringify(MOCK_SETTINGS);

        case 'reset_settings':
            // Reset to default
            Object.assign(MOCK_SETTINGS, {
                theme: 'light',
                defaultView: 'grid',
                showHiddenFiles: false,
                sortBy: 'name',
                sortDirection: 'asc',
                showDetailsPanel: false,
                terminalHeight: 240,
            });
            return;

        case 'get_template_paths_as_json':
            return JSON.stringify(MOCK_TEMPLATES);

        // Mock implementations for file operations
        case 'open_file':
            console.log(`Mock: Opening file ${params.file_path}`);
            return "File content would be here";

        case 'create_file':
            console.log(`Mock: Creating file ${params.file_name} in ${params.folder_path_abs}`);
            return;

        case 'create_directory':
            console.log(`Mock: Creating directory ${params.directory_name} in ${params.folder_path_abs}`);
            return;

        case 'rename':
            console.log(`Mock: Renaming ${params.old_path} to ${params.new_path}`);
            return;

        case 'move_to_trash':
            console.log(`Mock: Moving ${params.path} to trash`);
            return;

        case 'zip':
            console.log(`Mock: Zipping ${params.source_paths.join(', ')} to ${params.destination_path || 'auto'}`);
            return;

        case 'unzip':
            console.log(`Mock: Unzipping ${params.zip_paths.join(', ')} to ${params.destination_path || 'auto'}`);
            return;

        default:
            throw new Error(`Unhandled mock command: ${command}`);
    }
};

/**
 * Initialize mock Tauri API if needed
 */
export const initMockTauriApi = () => {
    // Check if we're in a development environment without Tauri
    if (typeof window.__TAURI__ === 'undefined') {
        console.log('Initializing mock Tauri API for development');

        // Create a mock __TAURI__ object
        window.__TAURI__ = {
            invoke: mockInvoke
        };

        // Also provide a global invoke function
        window.invoke = mockInvoke;
    }
};

export default initMockTauriApi;