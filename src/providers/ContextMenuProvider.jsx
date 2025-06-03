import React, { createContext, useContext, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useFileSystem } from './FileSystemProvider';
import { useHistory } from './HistoryProvider';
import { showNotification, showError, showSuccess, showConfirm } from '../utils/NotificationSystem';

const ContextMenuContext = createContext({
    isOpen: false,
    position: { x: 0, y: 0 },
    target: null,
    items: [],
    openContextMenu: () => {},
    closeContextMenu: () => {},
    clipboard: { items: [], operation: null }, // 'copy' or 'cut'
});

export default function ContextMenuProvider({ children }) {
    const [isOpen, setIsOpen] = useState(false);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const [target, setTarget] = useState(null);
    const [items, setItems] = useState([]);
    const [clipboard, setClipboard] = useState({ items: [], operation: null });
    const [isProcessing, setIsProcessing] = useState(false);

    const { selectedItems, loadDirectory, clearSelection } = useFileSystem();
    const { currentPath } = useHistory();

    // Check if item is in favorites
    const isInFavorites = useCallback((item) => {
        try {
            const existingFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');
            return existingFavorites.some(fav => fav.path === item.path);
        } catch (error) {
            console.error('Failed to check favorites:', error);
            return false;
        }
    }, []);

    // Add to favorites with live update
    const addToFavorites = useCallback((item) => {
        try {
            const existingFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');

            // Check if already in favorites
            const alreadyExists = existingFavorites.some(fav => fav.path === item.path);
            if (alreadyExists) {
                showNotification('This item is already in your favorites.');
                return;
            }

            const newFavorite = {
                name: item.name,
                path: item.path,
                icon: item.isDirectory || ('sub_file_count' in item) ? 'folder' : 'file'
            };

            const updatedFavorites = [...existingFavorites, newFavorite];
            localStorage.setItem('fileExplorerFavorites', JSON.stringify(updatedFavorites));

            // Dispatch events to update the UI immediately
            window.dispatchEvent(new CustomEvent('favorites-updated'));
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerFavorites',
                newValue: JSON.stringify(updatedFavorites)
            }));

            showSuccess(`Added "${item.name}" to favorites.`);
        } catch (error) {
            console.error('Failed to add to favorites:', error);
            showError('Failed to add to favorites.');
        }
    }, []);

    // Remove from favorites with live update
    const removeFromFavorites = useCallback((path) => {
        try {
            const existingFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');
            const updatedFavorites = existingFavorites.filter(fav => fav.path !== path);
            localStorage.setItem('fileExplorerFavorites', JSON.stringify(updatedFavorites));

            // Dispatch events to update the UI immediately
            window.dispatchEvent(new CustomEvent('favorites-updated'));
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerFavorites',
                newValue: JSON.stringify(updatedFavorites)
            }));
        } catch (error) {
            console.error('Failed to remove from favorites:', error);
        }
    }, []);

    // Update navigation history with live update
    const updateNavigationHistory = useCallback((path) => {
        try {
            const existingHistory = JSON.parse(sessionStorage.getItem('fileExplorerHistory') || '[]');
            const updatedHistory = [path, ...existingHistory.filter(p => p !== path)].slice(0, 10);
            sessionStorage.setItem('fileExplorerHistory', JSON.stringify(updatedHistory));

            // Dispatch events to update quick access immediately
            window.dispatchEvent(new CustomEvent('navigation-changed'));
            window.dispatchEvent(new CustomEvent('quick-access-updated'));
        } catch (error) {
            console.error('Failed to update navigation history:', error);
        }
    }, []);

    // Copy items to clipboard
    const copyToClipboard = useCallback(async (items) => {
        setClipboard({ items, operation: 'copy' });
        // Copy paths to system clipboard as well
        const paths = items.map(item => item.path).join('\n');
        try {
            await navigator.clipboard.writeText(paths);
        } catch (err) {
            console.warn('Failed to copy to system clipboard:', err);
        }
    }, []);

    // Cut items to clipboard
    const cutToClipboard = useCallback(async (items) => {
        setClipboard({ items, operation: 'cut' });
        const paths = items.map(item => item.path).join('\n');
        try {
            await navigator.clipboard.writeText(paths);
        } catch (err) {
            console.warn('Failed to copy to system clipboard:', err);
        }
    }, []);

    // Paste items from clipboard
    const pasteFromClipboard = useCallback(async () => {
        if (!clipboard.items.length || !currentPath) return;

        setIsProcessing(true);
        try {
            for (const item of clipboard.items) {
                const fileName = item.name;
                const sourcePath = item.path;
                const destPath = `${currentPath}/${fileName}`;

                if (clipboard.operation === 'cut') {
                    // Move operation
                    await invoke('rename', {
                        oldPath: sourcePath,
                        newPath: destPath
                    });
                } else {
                    // Copy operation - for now we'll skip the complex copy logic
                    // In a real implementation, you'd use a proper file copy API
                    console.warn('Copy operation not fully implemented');
                    throw new Error('Copy operation not available - use cut/move instead');
                }
            }

            // Clear clipboard if it was a cut operation
            if (clipboard.operation === 'cut') {
                setClipboard({ items: [], operation: null });
            }

            // Reload directory
            await loadDirectory(currentPath);
        } catch (error) {
            console.error('Paste operation failed:', error);
            showError(`Failed to paste: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [clipboard, currentPath, loadDirectory]);

    // Delete items
    const deleteItems = useCallback(async (items) => {
        if (!items.length) return;

        const itemNames = items.map(item => item.name).join(', ');
        const confirmMessage = `Are you sure you want to move ${items.length === 1 ? itemNames : `${items.length} items`} to trash?`;

        // Use custom confirm dialog
        const shouldDelete = await showConfirm(confirmMessage, 'Move to Trash');
        if (!shouldDelete) return;

        setIsProcessing(true);
        try {
            for (const item of items) {
                await invoke('move_to_trash', { path: item.path });
            }

            clearSelection();
            await loadDirectory(currentPath);
        } catch (error) {
            console.error('Delete operation failed:', error);
            showError(`Failed to delete: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory, clearSelection]);

    // Rename item - dispatch event to open rename modal
    const renameItem = useCallback((item) => {
        document.dispatchEvent(new CustomEvent('open-rename-modal', {
            detail: { item }
        }));
    }, []);

    // Zip items
    const zipItems = useCallback(async (items) => {
        if (!items.length) return;

        const sourcePaths = items.map(item => item.path);
        let destinationPath = null;

        if (items.length === 1) {
            // For single item, use its name as base for zip name
            const item = items[0];
            const baseName = item.name;
            destinationPath = `${currentPath}/${baseName}.zip`;
        } else {
            // For multiple items, ask user for zip name
            const zipName = window.prompt('Enter name for the zip file:', 'archive.zip');
            if (!zipName) return;
            destinationPath = `${currentPath}/${zipName}`;
            if (!destinationPath.endsWith('.zip')) {
                destinationPath += '.zip';
            }
        }

        setIsProcessing(true);
        try {
            await invoke('zip', {
                sourcePaths: sourcePaths,
                destinationPath: destinationPath
            });

            await loadDirectory(currentPath);
            showSuccess(`Successfully created ${destinationPath.split('/').pop()}`);
        } catch (error) {
            console.error('Zip operation failed:', error);
            showError(`Failed to create zip: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory]);

    // Unzip item
    const unzipItem = useCallback(async (item) => {
        if (!item.name.toLowerCase().endsWith('.zip')) return;

        setIsProcessing(true);
        try {
            await invoke('unzip', {
                zipPaths: [item.path],
                destinationPath: currentPath
            });

            await loadDirectory(currentPath);
            showSuccess(`Successfully extracted ${item.name}`);
        } catch (error) {
            console.error('Unzip operation failed:', error);
            showError(`Failed to extract: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory]);

    // Generate hash for a file
    const generateHash = useCallback(async (item) => {
        if (!item || item.isDirectory || 'sub_file_count' in item) {
            showError('Hash generation is only available for files.');
            return;
        }

        setIsProcessing(true);
        try {
            const hash = await invoke('gen_hash_and_return_string', { path: item.path });

            // Copy hash to clipboard
            await navigator.clipboard.writeText(hash);
            showSuccess(`Hash generated and copied to clipboard: ${hash.substring(0, 16)}...`);
        } catch (error) {
            console.error('Hash generation failed:', error);
            showError(`Failed to generate hash: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, []);

    // Generate hash and save to file - trigger modal
    const generateHashToFile = useCallback((item) => {
        if (!item || item.isDirectory || 'sub_file_count' in item) {
            showError('Hash generation is only available for files.');
            return;
        }

        // Dispatch event to open hash file modal
        document.dispatchEvent(new CustomEvent('open-hash-file-modal', {
            detail: { item }
        }));
    }, []);

    // Compare file with hash - trigger modal
    const compareHash = useCallback((item) => {
        if (!item || item.isDirectory || 'sub_file_count' in item) {
            showError('Hash comparison is only available for files.');
            return;
        }

        // Dispatch event to open hash compare modal
        document.dispatchEvent(new CustomEvent('open-hash-compare-modal', {
            detail: { item }
        }));
    }, []);

    // Get current folder metadata by loading parent directory
    const getCurrentFolderMetadata = useCallback(async (folderPath) => {
        if (!folderPath) return null;

        try {
            // Get parent directory path
            const separator = folderPath.includes('\\') ? '\\' : '/';
            const pathParts = folderPath.split(separator);
            const folderName = pathParts.pop();
            const parentPath = pathParts.join(separator) || separator;

            // Load parent directory to get metadata for current folder
            const parentContent = await invoke('open_directory', { path: parentPath });
            const parentData = JSON.parse(parentContent);

            // Find current folder in parent directory listing
            const currentFolderMeta = parentData.directories?.find(dir => dir.name === folderName);

            if (currentFolderMeta) {
                return currentFolderMeta;
            }

            // If not found in directories, create a basic folder object
            return {
                name: folderName || 'Root',
                path: folderPath,
                isDirectory: true,
                sub_file_count: 0,
                sub_dir_count: 0,
                is_symlink: false,
                access_rights_as_string: 'rwxr-xr-x',
                access_rights_as_number: 16877,
                size_in_bytes: 0,
                created: new Date().toISOString().replace('T', ' ').split('.')[0],
                last_modified: new Date().toISOString().replace('T', ' ').split('.')[0],
                accessed: new Date().toISOString().replace('T', ' ').split('.')[0]
            };
        } catch (error) {
            console.error('Failed to get folder metadata:', error);

            // Return a basic folder object as fallback
            const folderName = folderPath.split(/[/\\]/).pop() || 'Root';
            return {
                name: folderName,
                path: folderPath,
                isDirectory: true,
                sub_file_count: 0,
                sub_dir_count: 0,
                is_symlink: false,
                access_rights_as_string: 'rwxr-xr-x',
                access_rights_as_number: 16877,
                size_in_bytes: 0,
                created: new Date().toISOString().replace('T', ' ').split('.')[0],
                last_modified: new Date().toISOString().replace('T', ' ').split('.')[0],
                accessed: new Date().toISOString().replace('T', ' ').split('.')[0]
            };
        }
    }, []);

    // Show properties - dispatch event to open details panel
    const showProperties = useCallback(async (item) => {
        // If it's a folder path (string), get real metadata
        if (typeof item === 'string' || (item && !item.size_in_bytes && !item.sub_file_count)) {
            const folderPath = typeof item === 'string' ? item : item.path;
            try {
                const folderMetadata = await getCurrentFolderMetadata(folderPath);
                if (folderMetadata) {
                    // First, select the item with real metadata
                    document.dispatchEvent(new CustomEvent('select-item', {
                        detail: { item: folderMetadata }
                    }));
                    // Then show properties
                    document.dispatchEvent(new CustomEvent('show-properties', {
                        detail: { item: folderMetadata }
                    }));
                    return;
                }
            } catch (error) {
                console.error('Failed to get folder metadata for properties:', error);
            }
        }

        // Fallback to original item
        // First, select the item
        document.dispatchEvent(new CustomEvent('select-item', {
            detail: { item }
        }));
        // Then show properties
        document.dispatchEvent(new CustomEvent('show-properties', {
            detail: { item }
        }));
    }, [getCurrentFolderMetadata]);

    // Generate menu items
    const getMenuItemsForContext = useCallback((contextTarget) => {
        const isFile = contextTarget && !('sub_file_count' in contextTarget);
        const isDirectory = contextTarget && ('sub_file_count' in contextTarget);
        const hasClipboard = clipboard.items.length > 0;
        const isZipFile = contextTarget && contextTarget.name.toLowerCase().endsWith('.zip');

        // Empty space context menu
        if (!contextTarget) {
            return [
                {
                    id: 'paste',
                    label: 'Paste',
                    icon: 'paste',
                    disabled: !hasClipboard || isProcessing,
                    action: pasteFromClipboard
                },
                { type: 'separator' },
                {
                    id: 'new-folder',
                    label: 'New Folder',
                    icon: 'folder',
                    action: () => {
                        document.dispatchEvent(new CustomEvent('create-folder'));
                    }
                },
                {
                    id: 'new-file',
                    label: 'New File',
                    icon: 'file',
                    action: () => {
                        document.dispatchEvent(new CustomEvent('create-file'));
                    }
                },
                { type: 'separator' },
                {
                    id: 'properties',
                    label: 'Properties',
                    icon: 'properties',
                    action: () => showProperties(currentPath)
                },
                { type: 'separator' },
                {
                    id: 'refresh',
                    label: 'Refresh',
                    icon: 'refresh',
                    action: () => loadDirectory(currentPath)
                }
            ];
        }

        // File/folder context menu
        const targetItems = selectedItems.length > 1 ? selectedItems : [contextTarget];
        const itemIsInFavorites = isInFavorites(contextTarget);

        const menuItems = [
            {
                id: 'open',
                label: 'Open',
                icon: 'open',
                disabled: selectedItems.length > 1,
                action: async () => {
                    if (isDirectory) {
                        await loadDirectory(contextTarget.path);
                        updateNavigationHistory(contextTarget.path);
                    } else {
                        try {
                            await invoke('open_in_default_app', { path: contextTarget.path });
                        } catch (error) {
                            console.error('Failed to open file:', error);
                            showError(`Failed to open file: ${error.message || error}`);
                        }
                    }
                }
            },
            { type: 'separator' },
            {
                id: 'copy',
                label: 'Copy',
                icon: 'copy',
                disabled: isProcessing,
                action: () => copyToClipboard(targetItems)
            },
            {
                id: 'cut',
                label: 'Cut',
                icon: 'cut',
                disabled: isProcessing,
                action: () => cutToClipboard(targetItems)
            },
            {
                id: 'paste',
                label: 'Paste',
                icon: 'paste',
                disabled: !hasClipboard || !isDirectory || isProcessing,
                action: pasteFromClipboard
            },
            { type: 'separator' },
            {
                id: 'rename',
                label: 'Rename',
                icon: 'rename',
                disabled: selectedItems.length > 1 || isProcessing,
                action: () => renameItem(contextTarget)
            },
            {
                id: 'delete',
                label: 'Delete',
                icon: 'delete',
                disabled: isProcessing,
                action: () => deleteItems(targetItems)
            }
        ];

        // Add zip/unzip options
        if (isZipFile && selectedItems.length === 1) {
            menuItems.push(
                { type: 'separator' },
                {
                    id: 'extract',
                    label: 'Extract Here',
                    icon: 'extract',
                    disabled: isProcessing,
                    action: () => unzipItem(contextTarget)
                }
            );
        } else if (!isZipFile) {
            menuItems.push(
                { type: 'separator' },
                {
                    id: 'compress',
                    label: selectedItems.length > 1 ? 'Add to Archive...' : 'Compress to ZIP',
                    icon: 'compress',
                    disabled: isProcessing,
                    action: () => zipItems(targetItems)
                }
            );
        }

        // Add hash options for files only - as submenu
        if (isFile && selectedItems.length === 1) {
            menuItems.push(
                { type: 'separator' },
                {
                    id: 'hash-options',
                    label: 'Hash',
                    icon: 'hash',
                    disabled: isProcessing,
                    submenu: [
                        {
                            id: 'generate-hash',
                            label: 'Generate & Copy to Clipboard',
                            icon: 'hash',
                            disabled: isProcessing,
                            action: () => generateHash(contextTarget)
                        },
                        {
                            id: 'generate-hash-file',
                            label: 'Save Hash to File...',
                            icon: 'hash',
                            disabled: isProcessing,
                            action: () => generateHashToFile(contextTarget)
                        },
                        { type: 'separator' },
                        {
                            id: 'compare-hash',
                            label: 'Compare with Hash...',
                            icon: 'hash',
                            disabled: isProcessing,
                            action: () => compareHash(contextTarget)
                        }
                    ]
                }
            );
        }

        // Add favorites option
        menuItems.push(
            { type: 'separator' },
            {
                id: itemIsInFavorites ? 'remove-from-favorites' : 'add-to-favorites',
                label: itemIsInFavorites ? 'Remove from Favorites' : 'Add to Favorites',
                icon: 'star',
                disabled: selectedItems.length > 1,
                action: () => {
                    if (itemIsInFavorites) {
                        removeFromFavorites(contextTarget.path);
                    } else {
                        addToFavorites(contextTarget);
                    }
                }
            }
        );

        // Add properties at the end
        menuItems.push(
            { type: 'separator' },
            {
                id: 'properties',
                label: 'Properties',
                icon: 'properties',
                disabled: selectedItems.length > 1,
                action: () => showProperties(contextTarget)
            }
        );

        return menuItems;
    }, [selectedItems, clipboard, isProcessing, currentPath, copyToClipboard, cutToClipboard, pasteFromClipboard, deleteItems, renameItem, loadDirectory, showProperties, addToFavorites, removeFromFavorites, updateNavigationHistory, zipItems, unzipItem, isInFavorites, getCurrentFolderMetadata, generateHash, generateHashToFile, compareHash]);

    // Open context menu
    const openContextMenu = useCallback((e, contextTarget = null) => {
        e.preventDefault();

        const menuItems = getMenuItemsForContext(contextTarget);

        setPosition({ x: e.clientX, y: e.clientY });
        setTarget(contextTarget);
        setItems(menuItems);
        setIsOpen(true);
    }, [getMenuItemsForContext]);

    // Close context menu
    const closeContextMenu = useCallback(() => {
        setIsOpen(false);
    }, []);

    const contextValue = {
        isOpen,
        position,
        target,
        items,
        clipboard,
        isProcessing,
        openContextMenu,
        closeContextMenu,
        removeFromFavorites, // Export this for Sidebar to use
    };

    return (
        <ContextMenuContext.Provider value={contextValue}>
            {children}
        </ContextMenuContext.Provider>
    );
}

export const useContextMenu = () => useContext(ContextMenuContext);