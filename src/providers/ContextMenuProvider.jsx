import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { useFileSystem } from './FileSystemProvider';

// Create context
const ContextMenuContext = createContext({
    isOpen: false,
    position: { x: 0, y: 0 },
    target: null,
    items: [],
    openContextMenu: () => {},
    closeContextMenu: () => {},
});

export default function ContextMenuProvider({ children }) {
    const [isOpen, setIsOpen] = useState(false);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const [target, setTarget] = useState(null);
    const [items, setItems] = useState([]);
    const { selectedItems } = useFileSystem();

    // Close menu when clicking outside
    useEffect(() => {
        const handleGlobalClick = () => {
            setIsOpen(false);
        };

        if (isOpen) {
            document.addEventListener('click', handleGlobalClick);
        }

        return () => {
            document.removeEventListener('click', handleGlobalClick);
        };
    }, [isOpen]);

    // Handle escape key
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.key === 'Escape') {
                setIsOpen(false);
            }
        };

        document.addEventListener('keydown', handleKeyDown);

        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, []);

    // Function to generate menu items based on context
    const getMenuItemsForContext = useCallback((contextTarget) => {
        // Default items for all contexts
        const defaultItems = [
            {
                id: 'refresh',
                label: 'Refresh',
                icon: 'refresh',
                action: () => {
                    // Refresh logic will be handled by components
                },
            },
        ];

        // If no target, return minimal context menu
        if (!contextTarget) {
            return [
                ...defaultItems,
                {
                    id: 'create-new',
                    label: 'Create New',
                    icon: 'add',
                    submenu: [
                        {
                            id: 'new-file',
                            label: 'File',
                            icon: 'file',
                            action: () => {
                                // Create file logic will be handled by components
                            },
                        },
                        {
                            id: 'new-folder',
                            label: 'Folder',
                            icon: 'folder',
                            action: () => {
                                // Create folder logic will be handled by components
                            },
                        },
                    ],
                },
                {
                    id: 'paste',
                    label: 'Paste',
                    icon: 'paste',
                    action: () => {
                        // Paste logic will be handled by components
                    },
                    disabled: true, // Placeholder - should be enabled when clipboard has content
                },
            ];
        }

        // File/folder specific menu items
        const fileItems = [
            {
                id: 'open',
                label: 'Open',
                icon: 'open',
                action: () => {
                    // Open logic will be handled by components
                },
            },
            { type: 'separator' },
            {
                id: 'cut',
                label: 'Cut',
                icon: 'cut',
                action: () => {
                    // Cut logic will be handled by components
                },
            },
            {
                id: 'copy',
                label: 'Copy',
                icon: 'copy',
                action: () => {
                    // Copy logic will be handled by components
                },
            },
            { type: 'separator' },
            {
                id: 'rename',
                label: 'Rename',
                icon: 'rename',
                action: () => {
                    // Rename logic will be handled by components
                },
            },
            {
                id: 'delete',
                label: 'Delete',
                icon: 'delete',
                action: () => {
                    // Delete logic will be handled by components
                },
            },
            { type: 'separator' },
            {
                id: 'properties',
                label: 'Properties',
                icon: 'properties',
                action: () => {
                    // Properties logic will be handled by components
                },
            },
        ];

        // Add compression items for files and folders
        const compressionItems = [
            { type: 'separator' },
            {
                id: 'compress',
                label: 'Compress',
                icon: 'compress',
                action: () => {
                    // Compression logic will be handled by components
                },
            },
        ];

        // Add decompression for zip files
        const isZipFile = contextTarget.name?.toLowerCase().endsWith('.zip');
        const decompressionItems = isZipFile ? [
            {
                id: 'extract',
                label: 'Extract Here',
                icon: 'extract',
                action: () => {
                    // Extraction logic will be handled by components
                },
            },
            {
                id: 'extract-to',
                label: 'Extract To...',
                icon: 'extract-to',
                action: () => {
                    // Extraction to specific location logic will be handled by components
                },
            },
        ] : [];

        // Add template items for directories
        const isDirectory = 'sub_file_count' in contextTarget;
        const templateItems = isDirectory ? [
            { type: 'separator' },
            {
                id: 'save-as-template',
                label: 'Save as Template',
                icon: 'template',
                action: () => {
                    // Save as template logic will be handled by components
                },
            },
        ] : [];

        return [
            ...defaultItems,
            ...fileItems,
            ...compressionItems,
            ...decompressionItems,
            ...templateItems,
        ];
    }, []);

    // Open context menu
    const openContextMenu = useCallback((e, contextTarget = null) => {
        e.preventDefault();

        // Get menu position
        let x = e.clientX;
        let y = e.clientY;

        // Check if items already selected and contextTarget is one of them
        const targetIsInSelection = contextTarget && selectedItems.some(
            item => item.path === contextTarget.path
        );

        // Update target based on selection state
        const effectiveTarget = targetIsInSelection && selectedItems.length > 1
            ? selectedItems
            : contextTarget;

        // Get menu items based on context
        const menuItems = getMenuItemsForContext(effectiveTarget);

        // Boundary checking can be added here if needed

        setPosition({ x, y });
        setTarget(effectiveTarget);
        setItems(menuItems);
        setIsOpen(true);
    }, [getMenuItemsForContext, selectedItems]);

    // Close context menu
    const closeContextMenu = useCallback(() => {
        setIsOpen(false);
    }, []);

    const contextValue = {
        isOpen,
        position,
        target,
        items,
        openContextMenu,
        closeContextMenu,
    };

    return (
        <ContextMenuContext.Provider value={contextValue}>
            {children}
        </ContextMenuContext.Provider>
    );
}

// Custom hook for using the context menu
export const useContextMenu = () => useContext(ContextMenuContext);