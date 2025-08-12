import React, { useState, useEffect, useRef, useCallback } from 'react';
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
 * @param {string} [props.searchTerm=''] - The search term being used
 * @param {boolean} [props.disableArrowKeys=false] - Whether to disable arrow key navigation
 * @param {Function} [props.onColumnsChange] - Callback when columns per row changes
 * @returns {React.ReactElement} File list component
 */
const FileList = ({ data, isLoading, viewMode = 'grid', isSearching = false, searchTerm = '', disableArrowKeys = false, onColumnsChange }) => {
    const { selectedItems, selectItem, loadDirectory, clearSelection, focusedItem, setFocusedItem } = useFileSystem();
    const { openContextMenu } = useContextMenu();
    const [sortConfig, setSortConfig] = useState({ key: 'name', direction: 'asc' });
    const [isShiftKeyPressed, setIsShiftKeyPressed] = useState(false);
    const [isCtrlKeyPressed, setIsCtrlKeyPressed] = useState(false);
    const [columnsPerRow, setColumnsPerRow] = useState(4); // Dynamic column calculation
    const containerRef = useRef(null);
    const [lastSelectedIndex, setLastSelectedIndex] = useState(-1);

    /**
     * Calculate columns per row based on container width and item size
     */
    const calculateColumnsPerRow = useCallback(() => {
        if (!containerRef.current || viewMode !== 'grid') {
            setColumnsPerRow(1);
            onColumnsChange?.(1);
            return;
        }

        const container = containerRef.current;
        
        // Try to get computed styles first
        const computedStyle = window.getComputedStyle(container);
        const gridTemplateColumns = computedStyle.getPropertyValue('grid-template-columns');
        
        // If CSS Grid is being used, count the columns from grid-template-columns
        if (gridTemplateColumns && gridTemplateColumns !== 'none') {
            const columns = gridTemplateColumns.split(' ').length;
            setColumnsPerRow(columns);
            onColumnsChange?.(columns);
            return;
        }
        
        // Fallback: Calculate based on container width
        const containerWidth = container.offsetWidth || container.clientWidth;
        
        if (containerWidth === 0) {
            // Container not ready yet, use default
            setColumnsPerRow(4);
            onColumnsChange?.(4);
            return;
        }
        
        // Try counting actual file items in the DOM
        const fileItems = container.querySelectorAll('[data-path]');
        if (fileItems.length >= 2) {
            const firstItem = fileItems[0];
            const firstItemRect = firstItem.getBoundingClientRect();
            const containerRect = container.getBoundingClientRect();
            
            let columnsInFirstRow = 1;
            for (let i = 1; i < fileItems.length; i++) {
                const itemRect = fileItems[i].getBoundingClientRect();
                if (Math.abs(itemRect.top - firstItemRect.top) < 10) {
                    // Same row (within 10px tolerance)
                    columnsInFirstRow++;
                } else {
                    // Different row, stop counting
                    break;
                }
            }
            
            if (columnsInFirstRow > 1) {
                setColumnsPerRow(columnsInFirstRow);
                onColumnsChange?.(columnsInFirstRow);
                return;
            }
        }
        
        // More conservative estimates for item width
        const estimatedItemWidth = 160;
        const gap = 12;
        const padding = 32;
        
        const availableWidth = containerWidth - padding;
        const columns = Math.max(1, Math.floor((availableWidth + gap) / (estimatedItemWidth + gap)));
        
        setColumnsPerRow(columns);
        onColumnsChange?.(columns);
    }, [viewMode, onColumnsChange]);

    // Calculate columns on mount and resize
    useEffect(() => {
        calculateColumnsPerRow();
        
        const handleResize = () => {
            setTimeout(calculateColumnsPerRow, 100);
        };
        
        window.addEventListener('resize', handleResize);
        
        // Use ResizeObserver if available for more accurate detection
        let resizeObserver;
        if (containerRef.current && window.ResizeObserver) {
            resizeObserver = new ResizeObserver(() => {
                setTimeout(calculateColumnsPerRow, 50);
            });
            resizeObserver.observe(containerRef.current);
        }
        
        return () => {
            window.removeEventListener('resize', handleResize);
            if (resizeObserver) {
                resizeObserver.disconnect();
            }
        };
    }, [calculateColumnsPerRow]);

    // Also recalculate when view mode changes or data changes
    useEffect(() => {
        setTimeout(calculateColumnsPerRow, 100);
    }, [viewMode, data, calculateColumnsPerRow]);

    /**
     * Returns sorted data based on current sort configuration
     * @returns {Array} Sorted array of files and directories
     */
    const getSortedData = () => {
        if (!data || (!data.directories?.length && !data.files?.length)) {
            return [];
        }

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

        const currentSortedItems = getSortedData();
        
        // Determine if we clicked on an item or empty space
        const clickedItem = e.target.closest('[data-path]');
        const item = clickedItem ? currentSortedItems.find(item => item.path === clickedItem.dataset.path) : null;

        openContextMenu(e, item);
    };

    // Get sorted data - call this before useEffects that need it
    const sortedItems = getSortedData();

    /**
     * Sets up keyboard event listeners for multi-selection
     */
    useEffect(() => {
        const handleKeyDown = (e) => {
            setIsShiftKeyPressed(e.shiftKey);
            setIsCtrlKeyPressed(e.ctrlKey || e.metaKey);

            // Don't handle arrow keys if user is typing in an input or textarea
            if (e.target instanceof HTMLInputElement || 
                e.target instanceof HTMLTextAreaElement || 
                e.target.isContentEditable ||
                disableArrowKeys) {
                return;
            }

            // Handle arrow key navigation for focused item
            if (sortedItems && sortedItems.length > 0) {
                const currentFocusedIndex = focusedItem 
                    ? sortedItems.findIndex(item => item.path === focusedItem.path)
                    : -1;

                let newFocusedIndex = currentFocusedIndex;
                let moved = false;

                switch (e.key) {
                    case 'ArrowDown':
                        e.preventDefault();
                        if (viewMode === 'grid') {
                            newFocusedIndex = currentFocusedIndex + columnsPerRow;
                            if (newFocusedIndex < sortedItems.length) {
                                setFocusedItem(sortedItems[newFocusedIndex]);
                                moved = true;
                            } else {
                                const remainder = currentFocusedIndex % columnsPerRow;
                                setFocusedItem(sortedItems[remainder]);
                                newFocusedIndex = remainder;
                                moved = true;
                            }
                        } else {
                            newFocusedIndex = currentFocusedIndex < sortedItems.length - 1 
                                ? currentFocusedIndex + 1 
                                : 0;
                            setFocusedItem(sortedItems[newFocusedIndex]);
                            moved = true;
                        }
                        break;
                    case 'ArrowUp':
                        e.preventDefault();
                        if (viewMode === 'grid') {
                            newFocusedIndex = currentFocusedIndex - columnsPerRow;
                            if (newFocusedIndex >= 0) {
                                setFocusedItem(sortedItems[newFocusedIndex]);
                                moved = true;
                            } else {
                                const remainder = currentFocusedIndex % columnsPerRow;
                                const totalRows = Math.ceil(sortedItems.length / columnsPerRow);
                                const lastRowStartIndex = (totalRows - 1) * columnsPerRow;
                                const targetIndex = Math.min(lastRowStartIndex + remainder, sortedItems.length - 1);
                                setFocusedItem(sortedItems[targetIndex]);
                                newFocusedIndex = targetIndex;
                                moved = true;
                            }
                        } else {
                            newFocusedIndex = currentFocusedIndex > 0 
                                ? currentFocusedIndex - 1 
                                : sortedItems.length - 1;
                            setFocusedItem(sortedItems[newFocusedIndex]);
                            moved = true;
                        }
                        break;
                    case 'ArrowRight':
                        e.preventDefault();
                        if (viewMode === 'grid') {
                            if ((currentFocusedIndex + 1) % columnsPerRow === 0 || currentFocusedIndex === sortedItems.length - 1) {
                                const currentRow = Math.floor(currentFocusedIndex / columnsPerRow);
                                const rowStartIndex = currentRow * columnsPerRow;
                                setFocusedItem(sortedItems[rowStartIndex]);
                                newFocusedIndex = rowStartIndex;
                                moved = true;
                            } else {
                                setFocusedItem(sortedItems[currentFocusedIndex + 1]);
                                newFocusedIndex = currentFocusedIndex + 1;
                                moved = true;
                            }
                        } else {
                            newFocusedIndex = currentFocusedIndex < sortedItems.length - 1 
                                ? currentFocusedIndex + 1 
                                : 0;
                            setFocusedItem(sortedItems[newFocusedIndex]);
                            moved = true;
                        }
                        break;
                    case 'ArrowLeft':
                        e.preventDefault();
                        if (viewMode === 'grid') {
                            if (currentFocusedIndex % columnsPerRow === 0) {
                                const currentRow = Math.floor(currentFocusedIndex / columnsPerRow);
                                const nextRowLastIndex = Math.min((currentRow + 1) * columnsPerRow - 1, sortedItems.length - 1);
                                setFocusedItem(sortedItems[nextRowLastIndex]);
                                newFocusedIndex = nextRowLastIndex;
                                moved = true;
                            } else {
                                setFocusedItem(sortedItems[currentFocusedIndex - 1]);
                                newFocusedIndex = currentFocusedIndex - 1;
                                moved = true;
                            }
                        } else {
                            newFocusedIndex = currentFocusedIndex > 0 
                                ? currentFocusedIndex - 1 
                                : sortedItems.length - 1;
                            setFocusedItem(sortedItems[newFocusedIndex]);
                            moved = true;
                        }
                        break;
                    case 'Enter':
                        if (focusedItem) {
                            e.preventDefault();
                            if (focusedItem.isDirectory) {
                                loadDirectory(focusedItem.path);
                            } else {
                                const openFile = async () => {
                                    try {
                                        await invoke('open_in_default_app', { path: focusedItem.path });
                                    } catch (error) {
                                        console.error('Failed to open file:', error);
                                        showError(`Failed to open file: ${error.message || error}`);
                                    }
                                };
                                openFile();
                            }
                        }
                        break;
                }

                const item = sortedItems[newFocusedIndex];
                if (moved && item) {
                    if (e.shiftKey && lastSelectedIndex !== -1) {
                        // Shift+Arrow: select range from lastSelectedIndex to newFocusedIndex
                        const start = Math.min(lastSelectedIndex, newFocusedIndex);
                        const end = Math.max(lastSelectedIndex, newFocusedIndex);
                        const itemsToSelect = sortedItems.slice(start, end + 1);
                        clearSelection();
                        itemsToSelect.forEach(rangeItem => {
                            selectItem(rangeItem, true);
                        });
                    } else if (e.ctrlKey || e.metaKey) {
                        // Cmd/Ctrl+Arrow: add/remove focused item to selection
                        const isAlreadySelected = selectedItems.some(selected => selected.path === item.path);
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
                        setLastSelectedIndex(newFocusedIndex);
                    } else {
                        // No modifier: single selection
                        selectItem(item, false);
                        setLastSelectedIndex(newFocusedIndex);
                    }
                }
            }
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
        // Note: keydown cannot be passive because we need preventDefault for arrow keys
        window.addEventListener('keydown', handleKeyDown, { passive: false });
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
    }, [selectItem, clearSelection, sortedItems, focusedItem, setFocusedItem, loadDirectory, viewMode, showError, disableArrowKeys, columnsPerRow]);

    /**
     * Clears selection when displayed data changes
     */

    // Track if this is the first mount or a navigation (not a deselection)
    const isFirstMount = useRef(true);
    const prevDataRef = useRef();

    useEffect(() => {
        // Only clear selection if the data object reference actually changed (navigation), not just selection
        if (prevDataRef.current !== data) {
            clearSelection();
            setLastSelectedIndex(-1);
            prevDataRef.current = data;
        }
    }, [data, clearSelection]);

    useEffect(() => {
        // Only focus first item on first mount or navigation, not after deselection
        if (sortedItems.length > 0 && !focusedItem && isFirstMount.current) {
            setFocusedItem(sortedItems[0]);
            isFirstMount.current = false;
        }
    }, [sortedItems, focusedItem, setFocusedItem]);

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
                        searchTerm={isSearching ? searchTerm : undefined}
                    />
                </div>
            </div>
        );
    }

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
        // Always set the focused item when clicking on it
        setFocusedItem(item);

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
                ref={containerRef}
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
                            isFocused={focusedItem && focusedItem.path === item.path}
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

