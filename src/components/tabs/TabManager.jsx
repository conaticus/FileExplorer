import React, { useState, useEffect } from 'react';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useSftp } from '../../providers/SftpProvider';
import IconButton from '../common/IconButton';
import './tabs.css';

/**
 * TabManager component - Manages multiple tabs for file explorer navigation
 *
 * @param {Object} props - Component props
 * @param {React.ReactNode} props.children - Child components to render in the active tab
 * @returns {React.ReactElement} TabManager component
 */
const TabManager = ({ children }) => {
    const [tabs, setTabs] = useState([]);
    const [activeTabId, setActiveTabId] = useState(null);
    const { currentPath } = useHistory();
    const { loadDirectory } = useFileSystem();
    const { isSftpPath, parseSftpPath } = useSftp();

    /**
     * Initialize with current path
     */
    useEffect(() => {
        if (currentPath && tabs.length === 0) {
            const initialTab = {
                id: generateTabId(),
                title: getTabTitle(currentPath),
                path: currentPath,
                isActive: true
            };
            setTabs([initialTab]);
            setActiveTabId(initialTab.id);
        }
    }, [currentPath, tabs.length]);

    /**
     * Update active tab when path changes
     */
    useEffect(() => {
        if (currentPath && activeTabId) {
            setTabs(prevTabs =>
                prevTabs.map(tab =>
                    tab.id === activeTabId
                        ? { ...tab, title: getTabTitle(currentPath), path: currentPath }
                        : tab
                )
            );
        }
    }, [currentPath, activeTabId]);

    /**
     * Generates a unique ID for a new tab
     * @returns {string} Unique tab ID
     */
    const generateTabId = () => {
        return 'tab_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    };

    /**
     * Extracts a display title from a file path
     * @param {string} path - The file path
     * @returns {string} The extracted tab title
     */
    const getTabTitle = (path) => {
        if (!path) return 'Home';
        
        // Handle SFTP paths
        if (isSftpPath(path)) {
            const parsed = parseSftpPath(path);
            if (parsed && parsed.connection) {
                const remotePath = parsed.remotePath || '.';
                if (remotePath === '.' || remotePath === '/') {
                    return parsed.connection.name;
                } else {
                    const segments = remotePath.split('/').filter(Boolean);
                    const folderName = segments.length > 0 ? segments[segments.length - 1] : parsed.connection.name;
                    return `${parsed.connection.name}: ${folderName}`;
                }
            }
            return 'SFTP';
        }
        
        // Handle regular file system paths
        const segments = path.split(/[/\\]/).filter(Boolean);
        return segments.length > 0 ? segments[segments.length - 1] : 'Root';
    };

    /**
     * Gets the full path for tooltip display
     * @param {string} path - The file path
     * @returns {string} The full path for tooltip
     */
    const getFullPathForTooltip = (path) => {
        if (!path) return 'Home';
        
        // Handle SFTP paths - convert to standard format
        if (isSftpPath(path)) {
            const parsed = parseSftpPath(path);
            if (parsed && parsed.connection) {
                const remotePath = parsed.remotePath || '/';
                return `sftp://${parsed.connection.username}@${parsed.connection.host}:${parsed.connection.port}${remotePath}`;
            }
            return path;
        }
        
        // Return regular path as-is
        return path;
    };

    /**
     * Creates a new tab with the current path
     */
    const createNewTab = () => {
        const newTab = {
            id: generateTabId(),
            title: getTabTitle(currentPath),
            path: currentPath,
            isActive: false
        };

        setTabs(prevTabs => [...prevTabs, newTab]);
        switchToTab(newTab.id);
    };

    /**
     * Closes a tab and handles switching to another tab if needed
     * @param {string} tabId - ID of the tab to close
     * @param {React.MouseEvent} [event] - Optional click event
     */
    const closeTab = (tabId, event) => {
        event?.stopPropagation();

        if (tabs.length <= 1) return; // Don't close the last tab

        const tabIndex = tabs.findIndex(tab => tab.id === tabId);
        const isActiveTab = tabId === activeTabId;

        setTabs(prevTabs => prevTabs.filter(tab => tab.id !== tabId));

        // If we closed the active tab, switch to another tab
        if (isActiveTab) {
            const remainingTabs = tabs.filter(tab => tab.id !== tabId);
            if (remainingTabs.length > 0) {
                // Switch to the tab to the right, or the last tab if we closed the rightmost one
                const nextTabIndex = Math.min(tabIndex, remainingTabs.length - 1);
                const nextTab = remainingTabs[nextTabIndex];
                switchToTab(nextTab.id);
            }
        }
    };

    /**
     * Switches to a different tab and loads its directory
     * @param {string} tabId - ID of the tab to switch to
     * @async
     */
    const switchToTab = async (tabId) => {
        const tab = tabs.find(t => t.id === tabId);
        if (!tab) return;

        setActiveTabId(tabId);

        // Load the directory for this tab
        if (tab.path !== currentPath) {
            try {
                await loadDirectory(tab.path);
            } catch (error) {
                console.error('Failed to load directory for tab:', error);
                // Optionally show an error or close the tab
            }
        }
    };

    /**
     * Creates a duplicate of an existing tab
     * @param {string} tabId - ID of the tab to duplicate
     */
    const duplicateTab = (tabId) => {
        const tab = tabs.find(t => t.id === tabId);
        if (!tab) return;

        const duplicatedTab = {
            id: generateTabId(),
            title: tab.title,
            path: tab.path,
            isActive: false
        };

        const tabIndex = tabs.findIndex(t => t.id === tabId);
        const newTabs = [...tabs];
        newTabs.splice(tabIndex + 1, 0, duplicatedTab);

        setTabs(newTabs);
        switchToTab(duplicatedTab.id);
    };

    /**
     * Reorders tabs after a drag and drop operation
     * @param {number} dragIndex - Index of the tab being dragged
     * @param {number} hoverIndex - Index where the tab should be inserted
     */
    const reorderTabs = (dragIndex, hoverIndex) => {
        const newTabs = [...tabs];
        const draggedTab = newTabs[dragIndex];
        newTabs.splice(dragIndex, 1);
        newTabs.splice(hoverIndex, 0, draggedTab);
        setTabs(newTabs);
    };

    /**
     * Handles double-click events to prevent text selection
     * @param {React.MouseEvent} e - The double-click event
     */
    const handleDoubleClick = (e) => {
        // Prevent text selection on double-click
        e.preventDefault();
        e.stopPropagation();
        
        // Clear any existing text selection
        if (window.getSelection) {
            window.getSelection().removeAllRanges();
        }
    };

    /**
     * Handles mouse down events to prevent text selection
     * @param {React.MouseEvent} e - The mouse down event
     */
    const handleMouseDown = (e) => {
        // Prevent text selection on mouse down
        e.preventDefault();
    };

    /**
     * Handles selectstart events to prevent text selection
     * @param {React.SyntheticEvent} e - The selectstart event
     */
    const handleSelectStart = (e) => {
        // Prevent any text selection
        e.preventDefault();
        return false;
    };

    /**
     * Handles close button interactions with text selection prevention
     * @param {string} tabId - ID of the tab to close
     * @param {React.MouseEvent} e - The click event
     */
    const handleCloseTab = (tabId, e) => {
        e.preventDefault();
        e.stopPropagation();
        
        // Clear any existing text selection
        if (window.getSelection) {
            window.getSelection().removeAllRanges();
        }
        
        closeTab(tabId, e);
    };

    if (tabs.length === 0) {
        return <div className="tab-manager-loading">Loading...</div>;
    }

    return (
        <div className="tab-manager">
            <div className="tab-bar">
                <div className="tab-list">
                    {tabs.map((tab, index) => (
                        <div
                            key={tab.id}
                            className={`tab ${tab.id === activeTabId ? 'active' : ''}`}
                            onClick={() => switchToTab(tab.id)}
                            onDoubleClick={handleDoubleClick}
                            onMouseDown={handleMouseDown}
                            onSelectStart={handleSelectStart}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                // Show context menu with options: duplicate, close, close others, etc.
                            }}
                            draggable
                            onDragStart={(e) => {
                                e.dataTransfer.setData('text/plain', '');
                                e.dataTransfer.effectAllowed = 'move';
                            }}
                            onDragOver={(e) => {
                                e.preventDefault();
                                e.dataTransfer.dropEffect = 'move';
                            }}
                            onDrop={(e) => {
                                e.preventDefault();
                                // Handle tab reordering
                                const dragIndex = tabs.findIndex(t => t.id === activeTabId);
                                reorderTabs(dragIndex, index);
                            }}
                        >
                            <div className="tab-content">
                                <span className="tab-icon">
                                    <span className="icon icon-folder"></span>
                                </span>
                                <span className="tab-title" title={getFullPathForTooltip(tab.path)}>
                                    {tab.title}
                                </span>
                                {tabs.length > 1 && (
                                    <button
                                        className="tab-close"
                                        onClick={(e) => handleCloseTab(tab.id, e)}
                                        onDoubleClick={handleDoubleClick}
                                        onMouseDown={handleMouseDown}
                                        onSelectStart={handleSelectStart}
                                        aria-label="Close tab"
                                    >
                                        <span className="icon icon-x"></span>
                                    </button>
                                )}
                            </div>
                        </div>
                    ))}
                    
                    <div className="new-tab-button">
                        <IconButton
                            icon="plus"
                            size="sm"
                            onClick={createNewTab}
                        />
                    </div>
                </div>
            </div>

            <div className="tab-content-area">
                {children}
            </div>
        </div>
    );
};

export default TabManager;