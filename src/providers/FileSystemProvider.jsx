import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from './HistoryProvider';
import { useSettings } from './SettingsProvider';
import { useSftp } from './SftpProvider';
import { getDirectoryPath } from '../utils/pathUtils';

// Create file system context
const FileSystemContext = createContext({
    currentDirData: null,
    isLoading: false,
    selectedItems: [],
    focusedItem: null,
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
    setFocusedItem: () => {},
    zipItems: () => {},
    unzipItem: () => {},
    loadVolumes: () => {},
});

export default function FileSystemProvider({ children }) {
    const [currentDirData, setCurrentDirData] = useState(null);
    const [isLoading, setIsLoading] = useState(true); // Starten Sie mit isLoading=true
    const [selectedItems, setSelectedItems] = useState([]);
    const [focusedItem, setFocusedItem] = useState(null);
    const [volumes, setVolumes] = useState([]);
    const [error, setError] = useState(null);
    const { navigateTo, currentPath } = useHistory();
    const { settings } = useSettings();
    const { 
        isSftpPath, 
        loadSftpDirectory, 
        createSftpFile, 
        createSftpDirectory, 
        deleteSftpItem, 
        renameSftpItem, 
        downloadAndOpenSftpFile 
    } = useSftp();

    // Helper function to check if a file or directory is hidden
    const isHiddenItem = useCallback((name) => {
        // Files/folders starting with a dot are considered hidden on Unix-like systems
        return name.startsWith('.');
    }, []);

    // Helper function to filter directory data based on hidden files setting
    const filterDirectoryData = useCallback((dirData) => {
        if (!dirData) return dirData;
        
        // If show_hidden_files_and_folders is true, return data as-is
        if (settings.show_hidden_files_and_folders) {
            return dirData;
        }

        // Filter out hidden files and directories
        const filteredDirectories = dirData.directories ? 
            dirData.directories.filter(dir => !isHiddenItem(dir.name)) : [];
        const filteredFiles = dirData.files ? 
            dirData.files.filter(file => !isHiddenItem(file.name)) : [];

        return {
            ...dirData,
            directories: filteredDirectories,
            files: filteredFiles
        };
    }, [settings.show_hidden_files_and_folders, isHiddenItem]);

    // Load system volumes information
    const loadVolumes = useCallback(async () => {
        try {
            const volumesJson = await invoke('get_system_volumes_information_as_json');
            const volumesData = JSON.parse(volumesJson);
            setVolumes(volumesData);
            return volumesData; // Zurückgeben zur weiteren Verwendung
        } catch (err) {
            console.error('Failed to load volumes:', err);
            setError(`Failed to load volumes: ${err.message || err}`);
            // Fallback to mock data for development
            const mockVolumes = [
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
            ];
            setVolumes(mockVolumes);
            return mockVolumes;
        }
    }, []);

    // Load directory contents - enhanced with SFTP support
    const loadDirectory = useCallback(async (path) => {
        if (!path) {
            console.error("Cannot load directory: path is empty");
            setIsLoading(false);
            return false;
        }

        console.log(`Attempting to load directory: ${path}`);
        setIsLoading(true);
        setError(null);

        try {
            // Check if it's an SFTP path
            if (isSftpPath(path)) {
                console.log(`Loading SFTP directory: ${path}`);
                const sftpData = await loadSftpDirectory(path);
                if (sftpData) {
                    const filteredData = filterDirectoryData(sftpData);
                    setCurrentDirData(filteredData);
                    navigateTo(path);
                    console.log(`Successfully loaded SFTP directory: ${path}`);
                    return true;
                } else {
                    throw new Error('Failed to load SFTP directory');
                }
            }

            // Regular file system path
            const timeoutPromise = new Promise((_, reject) => {
                setTimeout(() => reject(new Error(`Directory loading timed out: ${path}`)), 10000);
            });

            const loadPromise = invoke('open_directory', { path });
            const dirContent = await Promise.race([loadPromise, timeoutPromise]);

            if (!dirContent) {
                throw new Error(`Empty response from open_directory: ${path}`);
            }

            try {
                const dirData = JSON.parse(dirContent);
                const filteredData = filterDirectoryData(dirData);
                setCurrentDirData(filteredData);
                navigateTo(path);
                console.log(`Successfully loaded directory: ${path}`);
                return true;
            } catch (parseError) {
                throw new Error(`Failed to parse directory data: ${parseError.message}`);
            }
        } catch (err) {
            console.error(`Failed to load directory: ${path}`, err);
            
            // Handle permission errors specifically for macOS user directories
            const errorMessage = err.message || err.toString();
            if (errorMessage.includes('permission denied') || errorMessage.includes('Permission denied')) {
                const folderName = path.split('/').pop() || path;
                const isUserDir = ['Desktop', 'Documents', 'Downloads', 'Pictures', 'Movies', 'Music'].some(dir => 
                    path.toLowerCase().includes(dir.toLowerCase())
                );
                
                if (isUserDir) {
                    setError(`Access denied to "${folderName}". This app needs permission to access your ${folderName} folder. Please grant permission in System Preferences > Security & Privacy > Privacy > Files and Folders.`);
                } else {
                    setError(`Permission denied: Cannot access "${folderName}". You may need to grant additional permissions to this application.`);
                }
            } else {
                setError(`Failed to load directory: ${errorMessage}`);
            }
            return false;
        } finally {
            setIsLoading(false);
        }
    }, [navigateTo, filterDirectoryData, isSftpPath, loadSftpDirectory]);

// Verbesserte getDefaultDirectory-Funktion
    const getDefaultDirectory = useCallback(async () => {
        // Setze isLoading auf true, um den Ladezustand anzuzeigen
        setIsLoading(true);

        try {
            // 1. Versuche zuerst, die Volumes zu laden
            console.log("Attempting to load volumes...");
            const volumesList = await loadVolumes();

            if (volumesList && volumesList.length > 0) {
                console.log(`Found volumes, using first mount point: ${volumesList[0].mount_point}`);
                return volumesList[0].mount_point;
            }

            console.warn('No volumes available, trying common paths instead');

            // 2. Liste gängiger Pfade für verschiedene Betriebssysteme
            //const commonPaths = ['/', 'C:\\', '/home', '/Users', '/tmp', '/var', '/opt'];
            const commonPaths = ['C:\\', '/Users', '/home'];

            // 3. Prüfe jeden Pfad einzeln
            for (const path of commonPaths) {
                console.log(`Checking if path is accessible: ${path}`);
                try {
                    // Verwende einen separaten try-catch für jeden Pfad
                    const result = await invoke('open_directory', { path });
                    if (result) {
                        console.log(`Successfully found accessible path: ${path}`);
                        return path;
                    }
                } catch (e) {
                    console.log(`Path ${path} not accessible`);
                    // Fehler ignorieren und mit dem nächsten Pfad fortfahren
                }
            }

            // 4. Hartcodierter Fallback als letzte Möglichkeit
            console.warn('All paths failed, using hardcoded default');
            return '/';
        } catch (error) {
            console.error('Error in getDefaultDirectory:', error);
            return '/';
        } finally {
            // Stelle sicher, dass ein Ladeindikator nicht nur wegen dieser Funktion aktiv bleibt
            // setIsLoading(false); -- Dies sollte in loadDirectory gesetzt werden, nicht hier
        }
    }, [loadVolumes]);

// Verbesserte initializeFirstDirectory-Funktion
    const initializeFirstDirectory = useCallback(async () => {
        console.log("Initializing first directory...");

        // 1. Start mit einem Timeout-Mechanismus
        let timeoutId = setTimeout(() => {
            console.error("Directory initialization timed out");
            setIsLoading(false);
            setError("Failed to initialize directory within the time limit");
        }, 10000);

        try {
            // 2. Versuche ein Standardverzeichnis zu bekommen
            const defaultDir = await getDefaultDirectory();
            console.log(`Got default directory: ${defaultDir}`);

            // 3. Versuche das Verzeichnis zu laden
            const success = await loadDirectory(defaultDir);

            if (!success) {
                // 4. Wenn der erste Versuch fehlschlägt, versuche absolute Fallback-Pfade
                console.warn("First directory load failed, trying fallbacks...");
                const fallbacks = ['/', 'C:\\', '/tmp'];
                //const fallbacks = ['C:\\', '/Users', '/home'];

                for (const fallback of fallbacks) {
                    if (fallback !== defaultDir) {
                        console.log(`Trying fallback directory: ${fallback}`);
                        if (await loadDirectory(fallback)) {
                            console.log(`Successfully loaded fallback directory: ${fallback}`);
                            break;
                        }
                    }
                }
            }
        } catch (err) {
            console.error('Failed to initialize directory:', err);
            setError('Failed to load any directory. Please check file system permissions.');
        } finally {
            // Cleanup Timeout und stelle sicher, dass der Ladezustand beendet wird
            clearTimeout(timeoutId);
            setIsLoading(false);
        }
    }, [getDefaultDirectory, loadDirectory]);

// Rufe initializeFirstDirectory nur einmal beim Laden auf
    useEffect(() => {
        // Nur initialisieren, wenn noch kein Verzeichnis geladen wurde
        if (!currentDirData && !currentPath) {
            console.log("No directory data or current path, initializing first directory...");
            initializeFirstDirectory();
        } else {
            console.log("Directory already loaded or path set, skipping initialization");
        }
    }, [initializeFirstDirectory, currentDirData, currentPath]);

// Reagiere auf Navigation/currentPath Änderungen
    useEffect(() => {
        if (currentPath) {
            console.log(`Current path changed to: ${currentPath}, loading directory...`);
            loadDirectory(currentPath);
        }
    }, [currentPath, loadDirectory]);

    // Open a file - enhanced with SFTP support
    const openFile = useCallback(async (filePath) => {
        setIsLoading(true);
        setError(null);

        try {
            if (isSftpPath(filePath)) {
                // Download and open SFTP file with default application
                await downloadAndOpenSftpFile(filePath);
            } else {
                // Open local file with default application
                await invoke('open_in_default_app', { path: filePath });
            }
        } catch (err) {
            console.error(`Failed to open file: ${filePath}`, err);
            setError(`Failed to open file: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [isSftpPath, downloadAndOpenSftpFile]);

    // Create a new file - enhanced with SFTP support
    const createFile = useCallback(async (folderPath, fileName) => {
        setIsLoading(true);
        setError(null);

        try {
            if (isSftpPath(folderPath)) {
                const success = await createSftpFile(folderPath, fileName);
                if (success) {
                    await loadDirectory(folderPath);
                }
            } else {
                await invoke('create_file', {
                    folderPathAbs: folderPath,
                    fileName: fileName
                });
                await loadDirectory(folderPath);
            }
        } catch (err) {
            console.error(`Failed to create file: ${fileName}`, err);
            setError(`Failed to create file: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory, isSftpPath, createSftpFile]);

    // Create a new directory - enhanced with SFTP support
    const createDirectory = useCallback(async (folderPath, directoryName) => {
        setIsLoading(true);
        setError(null);

        try {
            if (isSftpPath(folderPath)) {
                const success = await createSftpDirectory(folderPath, directoryName);
                if (success) {
                    await loadDirectory(folderPath);
                }
            } else {
                await invoke('create_directory', {
                    folderPathAbs: folderPath,
                    folderName: directoryName
                });
                await loadDirectory(folderPath);
            }
        } catch (err) {
            console.error(`Failed to create directory: ${directoryName}`, err);
            setError(`Failed to create directory: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory, isSftpPath, createSftpDirectory]);

    // Rename an item - enhanced with SFTP support
    const renameItem = useCallback(async (oldPath, newPath) => {
        setIsLoading(true);
        setError(null);

        try {
            console.log(`FileSystemProvider: Renaming "${oldPath}" -> "${newPath}"`);

            if (isSftpPath(oldPath)) {
                const pathParts = newPath.split('/');
                const newName = pathParts[pathParts.length - 1];
                const success = await renameSftpItem(oldPath, newName);
                if (success) {
                    // For SFTP paths, reload the current directory instead of trying to extract parent path
                    if (currentPath) {
                        await loadDirectory(currentPath);
                    }
                }
            } else {
                await invoke('rename', { oldPath, newPath });
                const dirPath = getDirectoryPath(oldPath);
                await loadDirectory(dirPath);
            }

            console.log('FileSystemProvider: Rename operation completed successfully');
        } catch (err) {
            console.error(`Failed to rename item: ${oldPath}`, err);
            setError(`Failed to rename item: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory, isSftpPath, renameSftpItem]);

    // Move item to trash - enhanced with SFTP support
    const moveToTrash = useCallback(async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            if (isSftpPath(path)) {
                const success = await deleteSftpItem(path);
                if (success) {
                    // For SFTP paths, reload the current directory instead of trying to extract parent path
                    if (currentPath) {
                        await loadDirectory(currentPath);
                    }
                    // Clear selection if the deleted item was selected
                    setSelectedItems(prev => prev.filter(item => !item.path.startsWith(path)));
                }
            } else {
                await invoke('move_to_trash', { path });
                const dirPath = path.substring(0, path.lastIndexOf('/'));
                await loadDirectory(dirPath);
                // Clear selection if the deleted item was selected
                setSelectedItems(prev => prev.filter(item => !item.path.startsWith(path)));
            }
        } catch (err) {
            console.error(`Failed to move item to trash: ${path}`, err);
            setError(`Failed to move item to trash: ${err.message || err}`);
        } finally {
            setIsLoading(false);
        }
    }, [loadDirectory, isSftpPath, deleteSftpItem, currentPath]);

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
        setFocusedItem(null);
    }, []);

    // Set focused item (for keyboard navigation and preview)
    const setFocusedItemCallback = useCallback((item) => {
        setFocusedItem(item);
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

    // Reload current directory when hidden files setting changes
    useEffect(() => {
        if (currentPath) {
            loadDirectory(currentPath);
        }
    }, [settings.show_hidden_files_and_folders, currentPath, loadDirectory]);

    const contextValue = {
        currentDirData,
        isLoading,
        selectedItems,
        focusedItem,
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
        setFocusedItem: setFocusedItemCallback,
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