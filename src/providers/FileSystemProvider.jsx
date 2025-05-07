import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { useHistory } from './HistoryProvider';

// Create file system context
const FileSystemContext = createContext({
    currentDirData: null,
    isLoading: false,
    selectedItems: [],
    volumes: [],
    error: null,
    loadDirectory: () => {},
    openFile: () => {},
    createFile: () => {},
    createDirectory: () => {},
    renameItem: () => {},
    moveToTrash: () => {},
    selectItem: () => {},
    selectMultiple: () => {},
    clearSelection: () => {},
    zipItems: () => {},
    unzipItem: () => {},
    loadVolumes: () => {},
});

export default function FileSystemProvider({ children }) {
    const [currentDirData, setCurrentDirData] = useState(null);
    const [isLoading, setIsLoading] = useState(false);
    const [selectedItems, setSelectedItems] = useState([]);
    const [volumes, setVolumes] = useState([]);
    const [error, setError] = useState(null);
    const { navigateTo, currentPath } = useHistory();

    // Load system volumes information
    const loadVolumes = useCallback(async () => {
        try {
            const volumesJson = await invoke('get_system_volumes_information_as_json');
            const volumesData = JSON.parse(volumesJson);
            setVolumes(volumesData);
        } catch (err) {
            console.error('Failed to load volumes:', err);
            setError(`Failed to load volumes: ${err.message || err}`);
            // Fallback to mock data for development
            setVolumes([
                {
                    volume_name: "Local Disk (C:)",
                    mount_point: "C:\\",
                    file_system: "NTFS",
                    size: 500107862016,
                    available_space: 158148874240,
                    is_removable: false,
                    total_written_bytes: 0,
                    total_read_bytes: 0
                }
            ]);
        }
    }, []);

    // Load directory contents
    const loadDirectory = useCallback(async (path) => {
        if (!path) return;

        setIsLoading(true);
        setError(null);

        try {
            const dirContent = await invoke('open_directory', { path });
            const dirData = JSON.parse(dirContent);

            setCurrentDirData(dirData);
            navigateTo(path);
        } catch (err) {
            console.error(`Failed to load directory: ${path}`, err);
            setError(`Failed to load directory: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [navigateTo]);

    // Open a file
    const openFile = useCallback(async (filePath) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('open_file', { filePath });
        } catch (err) {
            console.error(`Failed to open file: ${filePath}`, err);
            setError(`Failed to open file: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, []);

    // Create a new file
    const createFile = useCallback(async (folderPath, fileName) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('create_file', {
                folderPathAbs: folderPath,
                fileName
            });

            // Reload directory to show the new file
            await loadDirectory(folderPath);
        } catch (err) {
            console.error(`Failed to create file: ${fileName}`, err);
            setError(`Failed to create file: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory]);

    // Create a new directory
    const createDirectory = useCallback(async (folderPath, directoryName) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('create_directory', {
                folderPathAbs: folderPath,
                directoryName
            });

            // Reload directory to show the new directory
            await loadDirectory(folderPath);
        } catch (err) {
            console.error(`Failed to create directory: ${directoryName}`, err);
            setError(`Failed to create directory: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory]);

    // Rename an item (file or directory)
    const renameItem = useCallback(async (oldPath, newPath) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('rename', { oldPath, newPath });

            // Extract directory path from the old path to reload
            const dirPath = oldPath.substring(0, oldPath.lastIndexOf('/'));
            await loadDirectory(dirPath);
        } catch (err) {
            console.error(`Failed to rename item: ${oldPath}`, err);
            setError(`Failed to rename item: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory]);

    // Move item to trash
    const moveToTrash = useCallback(async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('move_to_trash', { path });

            // Extract directory path to reload
            const dirPath = path.substring(0, path.lastIndexOf('/'));
            await loadDirectory(dirPath);

            // Clear selection if the deleted item was selected
            setSelectedItems(prev => prev.filter(item => !item.path.startsWith(path)));
        } catch (err) {
            console.error(`Failed to move item to trash: ${path}`, err);
            setError(`Failed to move item to trash: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory]);

    // Zip selected items
    const zipItems = useCallback(async (sourcePaths, destinationPath = null) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('zip', {
                sourcePaths,
                destinationPath
            });

            // Reload current directory to show the new zip file
            if (currentPath) {
                await loadDirectory(currentPath);
            }
        } catch (err) {
            console.error('Failed to create zip:', err);
            setError(`Failed to create zip: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [currentPath, loadDirectory]);

    // Unzip an item
    const unzipItem = useCallback(async (zipPath, destinationPath = null) => {
        setIsLoading(true);
        setError(null);

        try {
            await invoke('unzip', {
                zipPaths: [zipPath],
                destinationPath
            });

            // Reload current directory to show the extracted files
            if (currentPath) {
                await loadDirectory(currentPath);
            }
        } catch (err) {
            console.error(`Failed to extract zip: ${zipPath}`, err);
            setError(`Failed to extract zip: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [currentPath, loadDirectory]);

    // Select an item
    const selectItem = useCallback((item, isMultiSelect = false) => {
        setSelectedItems(prevSelected => {
            // Check if the item is already selected
            const isAlreadySelected = prevSelected.some(
                selected => selected.path === item.path
            );

            if (isAlreadySelected) {
                // If multi-select, toggle selection
                return isMultiSelect
                    ? prevSelected.filter(selected => selected.path !== item.path)
                    : [item]; // Otherwise, make it the only selection
            } else {
                // Add to selection for multi-select, replace for single select
                return isMultiSelect ? [...prevSelected, item] : [item];
            }
        });
    }, []);

    // Select multiple items at once
    const selectMultiple = useCallback((items) => {
        setSelectedItems(items);
    }, []);

    // Clear selection
    const clearSelection = useCallback(() => {
        setSelectedItems([]);
    }, []);

    // Load volumes on mount
    useEffect(() => {
        loadVolumes();
    }, [loadVolumes]);

    // Reload current directory when navigating back/forward
    useEffect(() => {
        if (currentPath) {
            loadDirectory(currentPath);
        }
    }, [currentPath, loadDirectory]);

    const contextValue = {
        currentDirData,
        isLoading,
        selectedItems,
        volumes,
        error,
        loadDirectory,
        openFile,
        createFile,
        createDirectory,
        renameItem,
        moveToTrash,
        selectItem,
        selectMultiple,
        clearSelection,
        zipItems,
        unzipItem,
        loadVolumes,
    };

    return (
        <FileSystemContext.Provider value={contextValue}>
            {children}
        </FileSystemContext.Provider>
    );
}

// Custom hook for using the file system context
export const useFileSystem = () => useContext(FileSystemContext);