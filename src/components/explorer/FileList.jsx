import React, { useState, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useContextMenu } from '../../providers/ContextMenuProvider';
import { invoke } from '@tauri-apps/api/core';
import { showError } from '../../utils/NotificationSystem';
import FileItem from './FileItem';
import EmptyState from './EmptyState';
import './fileList.css';

/**
 * Component to display a list of files and directories
 * @param {Object} props - Component properties
 * @param {Object} props.data - The file/directory data to display
 * @param {boolean} props.isLoading - Whether the file list is currently loading
 * @param {string} [props.viewMode='grid'] - Display mode: 'grid', 'list', or 'details'
 * @param {boolean} [props.isSearching=false] - Whether the list is showing search results
 * @returns {React.ReactElement} File list component
 */
const FileList = ({ data, isLoading, viewMode = 'grid', isSearching = false }) => {
    const { selectedItems, selectItem, loadDirectory, clearSelection } = useFileSystem();
    const { openContextMenu } = useContextMenu();
    const [sortConfig, setSortConfig] = useState({ key: 'name', direction: 'asc' });
    const [isShiftKeyPressed, setIsShiftKeyPressed] = useState(false);
    const [isCtrlKeyPressed, setIsCtrlKeyPressed] = useState(false);
    const [lastSelectedIndex, setLastSelectedIndex] = useState(-1);

    /**
     * Handles click on the container (empty space)
     * @param {React.MouseEvent} e - The click event
     */
    const handleContainerClick = (e) => {
        // Only clear if clicking directly on the container, not on items
        // Also check that it's not a scroll-related interaction
        if (e.target === e.currentTarget && e.detail !== 0) {
            clearSelection();
            setLastSelectedIndex(-1);
        }
    };

    /**
     * Handles right-click context menu
     * @param {React.MouseEvent} e - The context menu event
     */
    const handleContextMenu = (e) => {
        // Always prevent default browser context menu in our container
        e.preventDefault();
        e.stopPropagation();

        // Determine if we clicked on an item or empty space
        const clickedItem = e.target.closest('[data-path]');
        const item = clickedItem ? sortedItems.find(item => item.path === clickedItem.dataset.path) : null;

        openContextMenu(e, item);
    };

    /**
     * Sets up keyboard event listeners for multi-selection
     */
    useEffect(() => {
        const handleKeyDown = (e) => {
            setIsShiftKeyPressed(e.shiftKey);
            setIsCtrlKeyPressed(e.ctrlKey || e.metaKey);
        };

        const handleKeyUp = (e) => {
            setIsShiftKeyPressed(e.shiftKey);
            setIsCtrlKeyPressed(e.ctrlKey || e.metaKey);
        };

        /**
         * Prevents default browser context menu while allowing scrolling
         * @param {MouseEvent} e - The mouse event
         */
        const preventDefaultContextMenu = (e) => {
            // Only prevent if the target is within our file list AND it's actually a right-click
            if (e.button === 2 && (e.target.closest('.file-list-container') || e.target.closest('.empty-state-container'))) {
                e.preventDefault();
            }
        };

        /**
         * Handles select-item events from context menu
         * @param {CustomEvent} e - The custom event
         */
        const handleSelectItem = (e) => {
            if (e.detail && e.detail.item) {
                selectItem(e.detail.item, false);
            }
        };

        /**
         * Handles clear selection events
         */
        const handleClearSelection = () => {
            clearSelection();
            setLastSelectedIndex(-1);
        };

        // Use passive listeners where possible to improve scroll performance
        window.addEventListener('keydown', handleKeyDown, { passive: true });
        window.addEventListener('keyup', handleKeyUp, { passive: true });
        window.addEventListener('mousedown', preventDefaultContextMenu, { passive: false });
        document.addEventListener('select-item', handleSelectItem);
        document.addEventListener('clear-selection', handleClearSelection);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('keyup', handleKeyUp);
            window.removeEventListener('mousedown', preventDefaultContextMenu);
            document.removeEventListener('select-item', handleSelectItem);
            document.removeEventListener('clear-selection', handleClearSelection);
        };
    }, [selectItem, clearSelection]);

    /**
     * Clears selection when displayed data changes
     */
    useEffect(() => {
        clearSelection();
        setLastSelectedIndex(-1);
    }, [data, clearSelection]);

    // If data is null or loading, show loading state
    if (isLoading) {
        return (
            <div className="file-list-container">
                <div className="loading-state">
                    <div className="loading-spinner"></div>
                    <p>Loading...</p>
                </div>
            </div>
        );
    }

    // If data is empty, show empty state
    if (!data || (!data.directories?.length && !data.files?.length)) {
        return (
            <div className="file-list-container">
                <div
                    className="empty-state-container"
                    onClick={handleContainerClick}
                    onContextMenu={handleContextMenu}
                    style={{ height: '100%', width: '100%' }}
                >
                    <EmptyState
                        type={isSearching ? 'no-results' : 'empty-folder'}
                        searchTerm={isSearching ? "your search" : undefined}
                    />
                </div>
            </div>
        );
    }

    /**
     * Returns sorted data based on current sort configuration
     * @returns {Array} Sorted array of files and directories
     */
    const getSortedData = () => {
        const { key, direction } = sortConfig;

        // Combine directories and files for sorting
        const combinedItems = [
            ...(data.directories || []).map(dir => ({ ...dir, isDirectory: true })),
            ...(data.files || []).map(file => ({ ...file, isDirectory: false }))
        ];

        // Always put directories first
        const sortedItems = [...combinedItems].sort((a, b) => {
            // Directories always come before files
            if (a.isDirectory && !b.isDirectory) return -1;
            if (!a.isDirectory && b.isDirectory) return 1;

            // Sort by the specified key
            let aValue = a[key];
            let bValue = b[key];

            // Handle string comparisons
            if (typeof aValue === 'string' && typeof bValue === 'string') {
                aValue = aValue.toLowerCase();
                bValue = bValue.toLowerCase();
            }

            // Handle date strings
            if (key === 'created' || key === 'last_modified' || key === 'accessed') {
                aValue = new Date(aValue).getTime();
                bValue = new Date(bValue).getTime();
            }

            if (aValue < bValue) return direction === 'asc' ? -1 : 1;
            if (aValue > bValue) return direction === 'asc' ? 1 : -1;
            return 0;
        });

        return sortedItems;
    };

    const sortedItems = getSortedData();

    /**
     * Handles changing the sort column/direction
     * @param {string} key - The column key to sort by
     */
    const handleSort = (key) => {
        setSortConfig(prevConfig => {
            // If clicking the same column, toggle direction
            if (prevConfig.key === key) {
                return {
                    ...prevConfig,
                    direction: prevConfig.direction === 'asc' ? 'desc' : 'asc'
                };
            }

            // Otherwise, sort by the new column in ascending order
            return { key, direction: 'asc' };
        });
    };

    /**
     * Handles item click (selection and opening)
     * @param {Object} item - The clicked item
     * @param {number} index - Index of the clicked item
     * @param {boolean} [isDoubleClick=false] - Whether this is a double-click
     */
    const handleItemClick = (item, index, isDoubleClick = false) => {
        // For double-click, open the item
        if (isDoubleClick) {
            if (item.isDirectory) {
                loadDirectory(item.path);
            } else {
                // Use the correct API for opening files in default app
                const openFile = async () => {
                    try {
                        await invoke('open_in_default_app', { path: item.path });
                    } catch (error) {
                        console.error('Failed to open file:', error);
                        showError(`Failed to open file: ${error.message || error}`);
                    }
                };
                openFile();
            }
            return;
        }

        // For single click, handle selection
        const isAlreadySelected = selectedItems.some(selected => selected.path === item.path);

        if (isShiftKeyPressed && lastSelectedIndex !== -1) {
            // Multi-select with shift key
            const start = Math.min(lastSelectedIndex, index);
            const end = Math.max(lastSelectedIndex, index);
            const itemsToSelect = sortedItems.slice(start, end + 1);

            // Clear current selection and select range
            clearSelection();
            itemsToSelect.forEach(rangeItem => {
                selectItem(rangeItem, true);
            });
            setLastSelectedIndex(index);
        } else if (isCtrlKeyPressed) {
            // Toggle selection with Ctrl key
            if (isAlreadySelected) {
                // Deselect by clearing and re-selecting others
                const otherSelected = selectedItems.filter(selected => selected.path !== item.path);
                clearSelection();
                otherSelected.forEach(otherItem => {
                    selectItem(otherItem, true);
                });
            } else {
                selectItem(item, true);
            }
            setLastSelectedIndex(index);
        } else {
            // Single selection
            if (isAlreadySelected && selectedItems.length === 1) {
                // If clicking on the only selected item, deselect it
                clearSelection();
                setLastSelectedIndex(-1);
            } else {
                selectItem(item, false);
                setLastSelectedIndex(index);
            }
        }
    };

    return (
        <div className="file-list-wrapper">
            <div
                className={`file-list-container view-mode-${viewMode.toLowerCase()} scrollable-content`}
                onClick={handleContainerClick}
                onContextMenu={handleContextMenu}
            >
                {/* Details view header with sortable columns */}
                {viewMode === 'details' && (
                    <div className="file-list-header">
                        <div
                            className={`file-list-column column-name ${sortConfig.key === 'name' ? `sorted-${sortConfig.direction}` : ''}`}
                            onClick={() => handleSort('name')}
                        >
                            Name
                            {sortConfig.key === 'name' && (
                                <span className={`sort-icon sort-${sortConfig.direction}`}></span>
                            )}
                        </div>
                        <div
                            className={`file-list-column column-size ${sortConfig.key === 'size_in_bytes' ? `sorted-${sortConfig.direction}` : ''}`}
                            onClick={() => handleSort('size_in_bytes')}
                        >
                            Size
                            {sortConfig.key === 'size_in_bytes' && (
                                <span className={`sort-icon sort-${sortConfig.direction}`}></span>
                            )}
                        </div>
                        <div
                            className={`file-list-column column-type ${sortConfig.key === 'type' ? `sorted-${sortConfig.direction}` : ''}`}
                            onClick={() => handleSort('type')}
                        >
                            Type
                            {sortConfig.key === 'type' && (
                                <span className={`sort-icon sort-${sortConfig.direction}`}></span>
                            )}
                        </div>
                        <div
                            className={`file-list-column column-modified ${sortConfig.key === 'last_modified' ? `sorted-${sortConfig.direction}` : ''}`}
                            onClick={() => handleSort('last_modified')}
                        >
                            Modified
                            {sortConfig.key === 'last_modified' && (
                                <span className={`sort-icon sort-${sortConfig.direction}`}></span>
                            )}
                        </div>
                    </div>
                )}

                {/* File list content */}
                <div className={`file-list view-mode-${viewMode.toLowerCase()} scrollable-content`}>
                    {sortedItems.map((item, index) => (
                        <FileItem
                            key={item.path}
                            item={item}
                            viewMode={viewMode}
                            isSelected={selectedItems.some(selected => selected.path === item.path)}
                            onClick={(e) => handleItemClick(item, index)}
                            onDoubleClick={() => handleItemClick(item, index, true)}
                            onContextMenu={handleContextMenu}
                        />
                    ))}
                </div>
            </div>
        </div>
    );
};

export default FileList;

