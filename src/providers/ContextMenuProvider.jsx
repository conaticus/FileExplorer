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

    // Compress items
    const compressItems = useCallback(async (items) => {
        if (!items.length) return;

        setIsProcessing(true);
        try {
            const sourcePaths = items.map(item => item.path);
            const zipName = items.length === 1
                ? `${items[0].name}.zip`
                : `Archive_${new Date().toISOString().split('T')[0]}.zip`;

            const destinationPath = `${currentPath}/${zipName}`;

            await invoke('zip', {
                source_paths: sourcePaths,
                destination_path: destinationPath
            });

            await loadDirectory(currentPath);
        } catch (error) {
            console.error('Compress operation failed:', error);
            alert(`Failed to compress: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory]);

    // Extract archive
    const extractArchive = useCallback(async (item) => {
        setIsProcessing(true);
        try {
            await invoke('unzip', {
                zip_paths: [item.path],
                destination_path: currentPath
            });

            await loadDirectory(currentPath);
        } catch (error) {
            console.error('Extract operation failed:', error);
            alert(`Failed to extract: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, [currentPath, loadDirectory]);

    // Generate hash
    const generateHash = useCallback(async (item) => {
        setIsProcessing(true);
        try {
            const hash = await invoke('gen_hash_and_return_string', { path: item.path });

            // Copy to clipboard
            await navigator.clipboard.writeText(hash);
            alert(`Hash generated and copied to clipboard:\n${hash}`);
        } catch (error) {
            console.error('Hash generation failed:', error);
            alert(`Failed to generate hash: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, []);

    // Compare with hash
    const compareWithHash = useCallback(async (item) => {
        const hashToCompare = prompt('Enter the hash to compare with:');
        if (!hashToCompare) return;

        setIsProcessing(true);
        try {
            const matches = await invoke('compare_file_or_dir_with_hash', {
                path: item.path,
                hash_to_compare: hashToCompare.trim()
            });

            alert(matches ? 'Hash matches!' : 'Hash does not match!');
        } catch (error) {
            console.error('Hash comparison failed:', error);
            alert(`Failed to compare hash: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, []);

    // Copy path to clipboard
    const copyPath = useCallback(async (item) => {
        try {
            await navigator.clipboard.writeText(item.path);
            // Show temporary notification
            const notification = document.createElement('div');
            notification.textContent = 'Path copied to clipboard';
            notification.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                background: var(--accent);
                color: white;
                padding: 12px 20px;
                border-radius: 6px;
                z-index: 10000;
                animation: slideIn 0.3s ease-out;
            `;
            document.body.appendChild(notification);
            setTimeout(() => {
                notification.remove();
            }, 2000);
        } catch (error) {
            console.error('Failed to copy path:', error);
        }
    }, []);

    // Save as template
    const saveAsTemplate = useCallback(async (item) => {
        setIsProcessing(true);
        try {
            await invoke('add_template', { template_path: item.path });
            alert(`"${item.name}" has been saved as a template.`);
        } catch (error) {
            console.error('Save as template failed:', error);
            alert(`Failed to save as template: ${error.message || error}`);
        } finally {
            setIsProcessing(false);
        }
    }, []);

    // Generate menu items
    const getMenuItemsForContext = useCallback((contextTarget) => {
        const isFile = contextTarget && !('sub_file_count' in contextTarget);
        const isDirectory = contextTarget && ('sub_file_count' in contextTarget);
        const isArchive = isFile && (contextTarget.name.endsWith('.zip') || contextTarget.name.endsWith('.rar') || contextTarget.name.endsWith('.7z'));
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
                        // This will be handled by the CreateFileButton component
                        document.dispatchEvent(new CustomEvent('create-folder'));
                    }
                },
                {
                    id: 'new-file',
                    label: 'New File',
                    icon: 'file',
                    action: () => {
                        // This will be handled by the CreateFileButton component
                        document.dispatchEvent(new CustomEvent('create-file'));
                    }
                },
                { type: 'separator' },
                {
                    id: 'copy-path',
                    label: 'Copy Current Path',
                    icon: 'copy',
                    action: () => copyPath({ path: currentPath })
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
                id: 'compress',
                label: targetItems.length > 1 ? 'Compress Selection' : 'Compress',
                icon: 'compress',
                disabled: isProcessing,
                action: () => compressItems(targetItems)
            },
            ...(isArchive ? [{
                id: 'extract',
                label: 'Extract Here',
                icon: 'extract',
                disabled: isProcessing,
                action: () => extractArchive(contextTarget)
            }] : []),
            { type: 'separator' },
            {
                id: 'copy-path',
                label: 'Copy Path',
                icon: 'copy',
                disabled: selectedItems.length > 1,
                action: () => copyPath(contextTarget)
            },
            { type: 'separator' },
            {
                id: 'hash-submenu',
                label: 'Hash',
                icon: 'hash',
                disabled: selectedItems.length > 1 || isDirectory,
                submenu: [
                    {
                        id: 'generate-hash',
                        label: 'Generate Hash',
                        icon: 'generate',
                        action: () => generateHash(contextTarget)
                    },
                    {
                        id: 'compare-hash',
                        label: 'Compare with Hash',
                        icon: 'compare',
                        action: () => compareWithHash(contextTarget)
                    }
                ]
            },
            ...(isDirectory ? [{
                id: 'save-template',
                label: 'Save as Template',
                icon: 'template',
                disabled: selectedItems.length > 1 || isProcessing,
                action: () => saveAsTemplate(contextTarget)
            }] : []),
            { type: 'separator' },
            {
                id: 'properties',
                label: 'Properties',
                icon: 'properties',
                disabled: selectedItems.length > 1,
                action: () => {
                    // This will be handled by showing properties in details panel
                    document.dispatchEvent(new CustomEvent('show-properties', {
                        detail: { item: contextTarget }
                    }));
                }
            }
        ];
    }, [selectedItems, clipboard, isProcessing, currentPath, copyToClipboard, cutToClipboard, pasteFromClipboard, deleteItems, renameItem, compressItems, extractArchive, generateHash, compareWithHash, copyPath, saveAsTemplate, loadDirectory]);

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