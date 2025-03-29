import React, { useState, useEffect, useCallback } from 'react';
import { useAppState } from '../providers/AppStateProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import { useTheme } from '../providers/ThemeProvider';
import { useSettings } from '../providers/SettingsProvider';

// Import Components
import SideBar from '../components/panels/SideBar';
import DetailPanel from '../components/panels/DetailPanel';
import TerminalPanel from '../components/panels/TerminalPanel';
import StatusBar from '../components/panels/StatusBar';
import LocationBar from '../components/navigation/LocationBar';
import NavButtons from '../components/navigation/NavButtons';
import GlobalSearch from '../components/search/GlobalSearch';
import FileView from '../components/file-view/FileView';
import FileContextMenu from '../components/context-menu/FileContextMenu';
import { SettingsButton } from '../components/settings/Settings';

// Import modern glass background
import bgImage from '../assets/themes/background.svg';

const MainLayout = () => {
    const { state, actions } = useAppState();
    const { isLoading, error } = useFileSystem();
    const { colors, themeSettings, activeTheme } = useTheme();
    const { openSettings } = useSettings();

    const [items, setItems] = useState([]);
    const [isContextMenuOpen, setIsContextMenuOpen] = useState(false);
    const [contextMenuPosition, setContextMenuPosition] = useState({ x: 0, y: 0 });
    const [contextMenuTargetItem, setContextMenuTargetItem] = useState(null);

    // Compute directly instead of using state and useEffect
    const showBgImage = activeTheme.includes('Glass') || themeSettings.enableGlassEffect;

    // Set a default path if none is set - Only run once on mount
    useEffect(() => {
        if (!state.currentPath) {
            actions.setCurrentPath('C:\\Users\\User\\Documents');
        }

        // The following is mock data for the sidebar - also only run once
        if (actions.addRecent && actions.addFavorite) {
            actions.addRecent('C:\\Users\\User\\Documents');
            actions.addRecent('C:\\Users\\User\\Pictures');
            actions.addRecent('C:\\Users\\User\\Desktop');
            actions.addFavorite('C:\\Users\\User\\Documents');
            actions.addFavorite('C:\\Users\\User\\Pictures');
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []); // Empty dependency array so it only runs once

    // Memoize the load items function to avoid recreating it on every render
    const loadItems = useCallback(async () => {
        if (!state.currentPath) return;

        actions.setLoading(true);
        try {
            // Example data for display
            const mockItems = [
                { name: 'Pictures', path: `${state.currentPath}\\Pictures`, type: 'directory', modified: '2023-05-03T10:30:00Z' },
                { name: 'Folder1', path: `${state.currentPath}\\Folder1`, type: 'directory', modified: '2023-01-05T11:20:00Z' },
                { name: 'Folder2', path: `${state.currentPath}\\Folder2`, type: 'directory', modified: '2023-12-02T16:40:00Z' },
                { name: 'config.json', path: `${state.currentPath}\\config.json`, type: 'file', size: '4 KB', modified: '2023-02-04T09:30:00Z' },
                { name: 'Document1.docx', path: `${state.currentPath}\\Document1.docx`, type: 'file', size: '25 KB', modified: '2023-01-15T10:30:00Z' },
                { name: 'Presentation.pptx', path: `${state.currentPath}\\Presentation.pptx`, type: 'file', size: '2.3 MB', modified: '2023-10-03T09:15:00Z' },
                { name: 'Spreadsheet.xlsx', path: `${state.currentPath}\\Spreadsheet.xlsx`, type: 'file', size: '156 KB', modified: '2023-02-20T14:45:00Z' },
                { name: 'test.txt', path: `${state.currentPath}\\test.txt`, type: 'file', size: '2 KB', modified: '2023-04-01T08:00:00Z' },
                { name: 'image.png', path: `${state.currentPath}\\image.png`, type: 'file', size: '1.5 MB', modified: '2023-08-12T13:45:00Z' },
                { name: 'video.mp4', path: `${state.currentPath}\\video.mp4`, type: 'file', size: '24.8 MB', modified: '2023-11-20T16:22:00Z' },
                { name: 'audio.mp3', path: `${state.currentPath}\\audio.mp3`, type: 'file', size: '3.4 MB', modified: '2023-07-08T09:15:00Z' },
                { name: 'archive.zip', path: `${state.currentPath}\\archive.zip`, type: 'file', size: '15.2 MB', modified: '2023-09-17T14:30:00Z' },
                { name: 'script.js', path: `${state.currentPath}\\script.js`, type: 'file', size: '12 KB', modified: '2023-10-05T11:20:00Z' },
                { name: 'styles.css', path: `${state.currentPath}\\styles.css`, type: 'file', size: '8 KB', modified: '2023-10-05T11:25:00Z' },
            ];

            setItems(mockItems);
        } catch (error) {
            console.error('Error loading directory contents:', error);
            actions.setError(error.message);
        } finally {
            actions.setLoading(false);
        }
    }, [state.currentPath, actions]);

    // Load files and folders when path changes
    useEffect(() => {
        loadItems();
    }, [loadItems]); // Only depend on the memoized function

    // Sort the items - memoize this calculation to avoid recomputing on every render
    const sortedItems = React.useMemo(() => {
        return [...items].sort((a, b) => {
            // Sort folders before files
            if (a.type !== b.type) {
                return a.type === 'directory' ? -1 : 1;
            }

            // Sort by the selected sort attribute
            switch (state.sortBy) {
                case 'name':
                    return state.sortDirection === 'asc'
                        ? a.name.localeCompare(b.name)
                        : b.name.localeCompare(a.name);

                case 'date':
                    return state.sortDirection === 'asc'
                        ? new Date(a.modified) - new Date(b.modified)
                        : new Date(b.modified) - new Date(a.modified);

                case 'size':
                    // Only relevant for files
                    if (a.type === 'directory' && b.type === 'directory') {
                        return state.sortDirection === 'asc'
                            ? a.name.localeCompare(b.name)
                            : b.name.localeCompare(a.name);
                    }

                    // Parse size (remove "KB", "MB", etc.)
                    const getSizeInBytes = (sizeStr) => {
                        if (!sizeStr) return 0;
                        const num = parseFloat(sizeStr);
                        if (sizeStr.includes('KB')) return num * 1024;
                        if (sizeStr.includes('MB')) return num * 1024 * 1024;
                        if (sizeStr.includes('GB')) return num * 1024 * 1024 * 1024;
                        return num;
                    };

                    return state.sortDirection === 'asc'
                        ? getSizeInBytes(a.size) - getSizeInBytes(b.size)
                        : getSizeInBytes(b.size) - getSizeInBytes(a.size);

                case 'type':
                    // Extract file extension
                    const getExtension = (filename) => {
                        if (!filename || !filename.includes('.')) return '';
                        return filename.split('.').pop().toLowerCase();
                    };

                    return state.sortDirection === 'asc'
                        ? getExtension(a.name).localeCompare(getExtension(b.name))
                        : getExtension(b.name).localeCompare(getExtension(a.name));

                default:
                    return 0;
            }
        });
    }, [items, state.sortBy, state.sortDirection]);

    // Open context menu
    const handleContextMenu = (e, item) => {
        e.preventDefault();
        setContextMenuPosition({ x: e.clientX, y: e.clientY });
        setContextMenuTargetItem(item);
        setIsContextMenuOpen(true);
    };

    // Close context menu
    const closeContextMenu = () => {
        setIsContextMenuOpen(false);
    };

    // Handle click on a file or folder
    const handleItemClick = (item, isDoubleClick = false) => {
        // If it's a directory and double-clicked, open the directory
        if (item.type === 'directory' && isDoubleClick) {
            actions.setCurrentPath(item.path);
        }
        // If it's a file and double-clicked, open the file
        else if (item.type === 'file' && isDoubleClick) {
            console.log(`Opening file: ${item.path}`);
        }
        // On single click select the item
        else {
            // Check if Ctrl key is pressed (for multiple selection)
            const isCtrlPressed = false; // TODO: Implement Ctrl key check

            if (isCtrlPressed) {
                // Add the item to selection or remove it if already selected
                if (state.selectedItems.includes(item.path)) {
                    actions.removeSelectedItem(item.path);
                } else {
                    actions.addSelectedItem(item.path);
                }
            } else {
                // Set selection to this item
                actions.setSelectedItems([item.path]);
            }
        }
    };

    // Update when sort attribute or direction changes
    const handleSortChange = (sortBy) => {
        if (state.sortBy === sortBy) {
            // Change sort direction if the same attribute is selected again
            actions.setSortDirection(state.sortDirection === 'asc' ? 'desc' : 'asc');
        } else {
            // Set the new sort attribute and default direction (ascending)
            actions.setSortBy(sortBy);
            actions.setSortDirection('asc');
        }
    };

    return (
        <div className="explorer-layout">
            {/* Background image for glass effect (conditionally rendered) */}
            {showBgImage && (
                <div
                    className="bg-pattern"
                    style={{
                        backgroundImage: `url(${bgImage})`,
                        opacity: themeSettings.enableGlassEffect ? 0.05 : 0
                    }}
                ></div>
            )}

            {/* Main navigation bar */}
            <div className={`explorer-header ${themeSettings.enableGlassEffect ? 'glass-effect' : ''}`}>
                <div className="explorer-header-left">
                    <NavButtons
                        canGoBack={state.historyIndex > 0}
                        canGoForward={state.historyIndex < state.history.length - 1}
                        onGoBack={actions.goBack}
                        onGoForward={actions.goForward}
                    />
                    <LocationBar
                        currentPath={state.currentPath}
                        onPathChange={actions.setCurrentPath}
                    />
                </div>

                <div className="explorer-header-right">
                    <GlobalSearch
                        isSearchActive={state.isSearchActive}
                        searchQuery={state.searchQuery}
                        onSearch={actions.search}
                        onClearSearch={actions.clearSearch}
                    />
                    <SettingsButton onClick={openSettings} />
                </div>
            </div>

            {/* Main content area */}
            <div className="main-container">
                {/* Sidebar */}
                <SideBar
                    isOpen={state.isSidebarOpen}
                    favorites={state.favorites}
                    recentPaths={state.recentPaths}
                    onToggle={() => actions.toggleSidebar()}
                    onNavigate={actions.setCurrentPath}
                    onAddFavorite={actions.addFavorite}
                    onRemoveFavorite={actions.removeFavorite}
                    enableGlassEffect={themeSettings.enableGlassEffect}
                />

                {/* Main content */}
                <div className="content-area">
                    <FileView
                        items={sortedItems}
                        viewMode={state.viewMode || themeSettings.defaultView}
                        selectedItems={state.selectedItems}
                        onItemClick={handleItemClick}
                        onContextMenu={handleContextMenu}
                        onSortChange={handleSortChange}
                        sortBy={state.sortBy}
                        sortDirection={state.sortDirection}
                        isLoading={isLoading}
                        error={error}
                        enableGlassEffect={themeSettings.enableGlassEffect}
                    />
                </div>

                {/* Detail panel (right) */}
                <DetailPanel
                    isOpen={state.isDetailPanelOpen}
                    selectedItems={state.selectedItems}
                    onClose={() => actions.toggleDetailPanel(false)}
                    enableGlassEffect={themeSettings.enableGlassEffect}
                />
            </div>

            {/* Terminal (bottom) */}
            <TerminalPanel
                isOpen={state.isTerminalPanelOpen}
                currentPath={state.currentPath}
                onClose={() => actions.toggleTerminalPanel(false)}
                enableGlassEffect={themeSettings.enableGlassEffect}
            />

            {/* Status bar */}
            <StatusBar
                selectedItems={state.selectedItems}
                currentPath={state.currentPath}
                isLoading={isLoading}
                enableGlassEffect={themeSettings.enableGlassEffect}
            />

            {/* Context menu */}
            {isContextMenuOpen && (
                <FileContextMenu
                    position={contextMenuPosition}
                    targetItem={contextMenuTargetItem}
                    selectedItems={state.selectedItems}
                    onClose={closeContextMenu}
                    inEmptySpace={!contextMenuTargetItem}
                    currentPath={state.currentPath}
                />
            )}
        </div>
    );
};

export default MainLayout;