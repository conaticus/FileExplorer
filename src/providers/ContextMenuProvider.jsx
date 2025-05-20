import React, { createContext, useContext, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useFileSystem } from './FileSystemProvider';
import { useHistory } from './HistoryProvider';

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

    // Add to favorites
    const addToFavorites = useCallback((item) => {
        try {
            const existingFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');

            // Check if already in favorites
            const alreadyExists = existingFavorites.some(fav => fav.path === item.path);
            if (alreadyExists) {
                alert('This item is already in your favorites.');
                return;
            }

            const newFavorite = {
                name: item.name,
                path: item.path,
                icon: item.isDirectory || ('sub_file_count' in item) ? 'folder' : 'file'
            };

            const updatedFavorites = [...existingFavorites, newFavorite];
            localStorage.setItem('fileExplorerFavorites', JSON.stringify(updatedFavorites));

            // Dispatch event to update sidebar
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerFavorites',
                newValue: JSON.stringify(updatedFavorites)
            }));

            alert(`Added "${item.name}" to favorites.`);
        } catch (error) {
            console.error('Failed to add to favorites:', error);
            alert('Failed to add to favorites.');
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
                        old_path: sourcePath,
                        new_path: destPath
                    });
                } else {
                    // Copy operation - we need to use system copy command
                    const command = process.platform === 'win32'
                        ? `copy "${sourcePath}" "${destPath}"`
                        : `cp -r "${sourcePath}" "${destPath}"`;

                    await invoke('execute_command', { command });
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
            alert(`Failed to paste: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [clipboard, currentPath, loadDirectory]);

    // Delete items
    const deleteItems = useCallback(async (items) => {
        if (!items.length) return;

        const itemNames = items.map(item => item.name).join(', ');
        const confirmMessage = `Are you sure you want to move ${items.length === 1 ? itemNames : `${items.length} items`} to trash?`;

        if (!confirm(confirmMessage)) return;

        setIsProcessing(true);
        try {
            for (const item of items) {
                await invoke('move_to_trash', { path: item.path });
            }

            clearSelection();
            await loadDirectory(currentPath);
        } catch (error) {
            console.error('Delete operation failed:', error);
            alert(`Failed to delete: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory, clearSelection]);

    // Rename item
    const renameItem = useCallback(async (item, newName) => {
        if (!newName || newName === item.name) return;

        const pathParts = item.path.split('/');
        pathParts[pathParts.length - 1] = newName;
        const newPath = pathParts.join('/');

        setIsProcessing(true);
        try {
            await invoke('rename', {
                old_path: item.path,
                new_path: newPath
            });

            await loadDirectory(currentPath);
        } catch (error) {
            console.error('Rename operation failed:', error);
            if (error.message && error.message.includes('already exists')) {
                const shouldCreateCopy = confirm(`A file named "${newName}" already exists. Create a copy instead?`);
                if (shouldCreateCopy) {
                    const extension = newName.includes('.') ? newName.split('.').pop() : '';
                    const baseName = extension ? newName.replace(`.${extension}`, '') : newName;
                    const copyName = extension ? `${baseName} - Copy.${extension}` : `${baseName} - Copy`;
                    await renameItem(item, copyName);
                }
            } else {
                alert(`Failed to rename: ${error.message || error}`);
            }
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory]);

    // Show properties - dispatch event to open details panel
    const showProperties = useCallback((item) => {
        // First, select the item
        document.dispatchEvent(new CustomEvent('select-item', {
            detail: { item }
        }));
        // Then show properties
        document.dispatchEvent(new CustomEvent('show-properties', {
            detail: { item }
        }));
    }, []);

    // Generate menu items
    const getMenuItemsForContext = useCallback((contextTarget) => {
        const isFile = contextTarget && !('sub_file_count' in contextTarget);
        const isDirectory = contextTarget && ('sub_file_count' in contextTarget);
        const hasClipboard = clipboard.items.length > 0;

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
                    action: () => showProperties({ name: 'Current Folder', path: currentPath })
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

        return [
            {
                id: 'open',
                label: 'Open',
                icon: 'open',
                disabled: selectedItems.length > 1,
                action: async () => {
                    if (isDirectory) {
                        loadDirectory(contextTarget.path);
                    } else {
                        try {
                            await invoke('open_file', { file_path: contextTarget.path });
                        } catch (error) {
                            console.error('Failed to open file:', error);
                            alert(`Failed to open file: ${error.message || error}`);
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
                action: () => {
                    const newName = prompt('Enter new name:', contextTarget.name);
                    if (newName) {
                        renameItem(contextTarget, newName);
                    }
                }
            },
            {
                id: 'delete',
                label: 'Delete',
                icon: 'delete',
                disabled: isProcessing,
                action: () => deleteItems(targetItems)
            },
            { type: 'separator' },
            {
                id: 'add-to-favorites',
                label: 'Add to Favorites',
                icon: 'star',
                disabled: selectedItems.length > 1,
                action: () => addToFavorites(contextTarget)
            },
            { type: 'separator' },
            {
                id: 'properties',
                label: 'Properties',
                icon: 'properties',
                disabled: selectedItems.length > 1,
                action: () => showProperties(contextTarget)
            }
        ];
    }, [selectedItems, clipboard, isProcessing, currentPath, copyToClipboard, cutToClipboard, pasteFromClipboard, deleteItems, renameItem, loadDirectory, showProperties, addToFavorites]);

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
    };

    return (
        <ContextMenuContext.Provider value={contextValue}>
            {children}
        </ContextMenuContext.Provider>
    );
}

export const useContextMenu = () => useContext(ContextMenuContext);