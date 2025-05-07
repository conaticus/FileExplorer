import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
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
    const [isLoading, setIsLoading] = useState(true); // Starten Sie mit isLoading=true
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

    // Load directory contents
    // Verbesserte loadDirectory-Funktion mit robuster Fehlerbehandlung
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
            // Setze ein Timeout für den Fall, dass die Operation hängen bleibt
            const timeoutPromise = new Promise((_, reject) => {
                setTimeout(() => reject(new Error(`Directory loading timed out: ${path}`)), 10000);
            });

            // Versuche das Verzeichnis zu laden
            const loadPromise = invoke('open_directory', { path });

            // Verwende Promise.race, um entweder das Ergebnis zu erhalten oder nach Timeout abzubrechen
            const dirContent = await Promise.race([loadPromise, timeoutPromise]);

            if (!dirContent) {
                throw new Error(`Empty response from open_directory: ${path}`);
            }

            try {
                const dirData = JSON.parse(dirContent);
                setCurrentDirData(dirData);
                navigateTo(path);
                console.log(`Successfully loaded directory: ${path}`);
                return true;
            } catch (parseError) {
                throw new Error(`Failed to parse directory data: ${parseError.message}`);
            }
        } catch (err) {
            console.error(`Failed to load directory: ${path}`, err);
            setError(`Failed to load directory: ${err.message || err}`);
            return false;
        } finally {
            // Stelle sicher, dass isLoading auf jeden Fall auf false gesetzt wird
            setIsLoading(false);
        }
    }, [navigateTo]);

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