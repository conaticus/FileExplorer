import React, { useState, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useContextMenu } from '../../providers/ContextMenuProvider';
import FileItem from './FileItem';
import EmptyState from './EmptyState';
import './fileList.css';

const FileList = ({ data, isLoading, viewMode = 'grid', isSearching = false }) => {
    const { selectedItems, selectItem, loadDirectory, clearSelection } = useFileSystem();
    const { openContextMenu } = useContextMenu();
    const [sortConfig, setSortConfig] = useState({ key: 'name', direction: 'asc' });
    const [isShiftKeyPressed, setIsShiftKeyPressed] = useState(false);
    const [isCtrlKeyPressed, setIsCtrlKeyPressed] = useState(false);
    const [lastSelectedIndex, setLastSelectedIndex] = useState(-1);

    // Handle keyboard events for multi-selection
    useEffect(() => {
        const handleKeyDown = (e) => {
            setIsShiftKeyPressed(e.shiftKey);
            setIsCtrlKeyPressed(e.ctrlKey || e.metaKey);
        };

        const handleKeyUp = (e) => {
            setIsShiftKeyPressed(e.shiftKey);
            setIsCtrlKeyPressed(e.ctrlKey || e.metaKey);
        };

        // Listen for select-item events from context menu
        const handleSelectItem = (e) => {
            if (e.detail && e.detail.item) {
                selectItem(e.detail.item, false);
            }
        };

        // Listen for clear selection events
        const handleClearSelection = () => {
            clearSelection();
            setLastSelectedIndex(-1);
        };

        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);
        document.addEventListener('select-item', handleSelectItem);
        document.addEventListener('clear-selection', handleClearSelection);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('keyup', handleKeyUp);
            document.removeEventListener('select-item', handleSelectItem);
            document.removeEventListener('clear-selection', handleClearSelection);
        };
    }, [selectItem, clearSelection]);

    // Clear selection when data changes
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
                <EmptyState
                    type={isSearching ? 'no-results' : 'empty-folder'}
                    searchTerm={isSearching ? "your search" : undefined}
                />
            </div>
        );
    }

    // Sort data
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

    // Handle sort change
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

    // Handle item click
    const handleItemClick = (item, index, isDoubleClick = false) => {
        // For double-click, open the item
        if (isDoubleClick) {
            if (item.isDirectory) {
                loadDirectory(item.path);
            } else {
                // Open file using the open_file endpoint
                const openFile = async () => {
                    try {
                        const { invoke } = await import('@tauri-apps/api/core');
                        await invoke('open_file', { file_path: item.path });
                    } catch (error) {
                        console.error('Failed to open file:', error);
                        alert(`Failed to open file: ${error.message || error}`);
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

    // Handle context menu
    const handleContextMenu = (e, item) => {
        openContextMenu(e, item);
    };

    // Handle container click (click on empty space)
    const handleContainerClick = (e) => {
        // Only clear if clicking directly on the container, not on items
        if (e.target === e.currentTarget) {
            clearSelection();
            setLastSelectedIndex(-1);
        }
    };

    return (
        <div className="file-list-wrapper">
            <div
                className={`file-list-container view-mode-${viewMode} scrollable-content`}
                onClick={handleContainerClick}
                onContextMenu={(e) => {
                    if (e.target === e.currentTarget) {
                        handleContextMenu(e, null);
                    }
                }}
            >
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

                <div className={`file-list view-mode-${viewMode}`}>
                    {sortedItems.map((item, index) => (
                        <FileItem
                            key={item.path}
                            item={item}
                            viewMode={viewMode}
                            isSelected={selectedItems.some(selected => selected.path === item.path)}
                            onClick={(e) => handleItemClick(item, index)}
                            onDoubleClick={() => handleItemClick(item, index, true)}
                            onContextMenu={(e) => handleContextMenu(e, item)}
                        />
                    ))}
                </div>
            </div>
        </div>
    );
};

export default FileList;