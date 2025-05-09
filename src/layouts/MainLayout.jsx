import React, { useState, useCallback, useEffect } from 'react';
import { useTheme } from '../providers/ThemeProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import { useContextMenu } from '../providers/ContextMenuProvider';
import Sidebar from '../components/sidebar/Sidebar';
import PathBreadcrumb from '../components/explorer/PathBreadcrumb';
import NavigationButtons from '../components/explorer/NavigationButtons';
import FileList from '../components/explorer/FileList';
import SearchBar from '../components/search/SearchBar';
import DetailsPanel from '../components/explorer/DetailsPanel';
import ContextMenu from '../components/contextMenu/ContextMenu';
import Terminal from '../components/terminal/Terminal';
import ViewModes from '../components/explorer/ViewModes';
import CreateFileButton from '../components/explorer/CreateFileButton';

import '../styles/layouts/mainLayout.css';

const MainLayout = () => {
    const { theme, toggleTheme } = useTheme();
    const { isLoading, currentDirData, selectedItems, loadDirectory, volumes } = useFileSystem();
    const { isOpen: isContextMenuOpen, position, items, closeContextMenu } = useContextMenu();

    // State for UI components
    const [isDetailsPanelOpen, setIsDetailsPanelOpen] = useState(false);
    const [isTerminalOpen, setIsTerminalOpen] = useState(false);
    const [viewMode, setViewMode] = useState('grid'); // 'list', 'grid', 'details'
    const [searchValue, setSearchValue] = useState('');
    const [searchResults, setSearchResults] = useState(null);

    // Load default location on first render (i.e., system volumes/This PC view)
    useEffect(() => {
        if (volumes.length > 0 && !currentDirData) {
            // For development/testing, load first volume if available
            const firstVolume = volumes[0];
            if (firstVolume?.mount_point) {
                loadDirectory(firstVolume.mount_point);
            }
        }
    }, [volumes, currentDirData, loadDirectory]);

    // Handle search
    const handleSearch = useCallback((value) => {
        setSearchValue(value);

        if (!value.trim()) {
            setSearchResults(null);
            return;
        }

        // In a real implementation, this would use the backend search API
        // For now, filter the current directory data
        if (currentDirData) {
            const filteredFiles = currentDirData.files.filter(file =>
                file.name.toLowerCase().includes(value.toLowerCase())
            );

            const filteredDirs = currentDirData.directories.filter(dir =>
                dir.name.toLowerCase().includes(value.toLowerCase())
            );

            setSearchResults({
                directories: filteredDirs,
                files: filteredFiles
            });
        }
    }, [currentDirData]);

    // Toggle details panel
    const toggleDetailsPanel = useCallback(() => {
        setIsDetailsPanelOpen(prev => !prev);
    }, []);

    // Toggle terminal
    const toggleTerminal = useCallback(() => {
        setIsTerminalOpen(prev => !prev);
    }, []);

    // Get the data to display (search results or current directory)
    const displayData = searchResults || currentDirData;

    // Clear search when changing directory
    useEffect(() => {
        setSearchValue('');
        setSearchResults(null);
    }, [currentDirData]);

    return (
        <div className="main-layout">
            {/* Sidebar */}
            <Sidebar />

            {/* Main content area */}
            <div className="content-area">
                {/* Toolbar with navigation and actions */}
                <div className="toolbar">
                    <div className="toolbar-left">
                        <NavigationButtons />
                        <PathBreadcrumb />
                    </div>
                    <div className="toolbar-right">
                        <SearchBar
                            value={searchValue}
                            onChange={handleSearch}
                        />
                        <ViewModes
                            currentMode={viewMode}
                            onChange={setViewMode}
                        />
                        <button
                            className="icon-button"
                            onClick={toggleDetailsPanel}
                            aria-label={isDetailsPanelOpen ? "Hide details" : "Show details"}
                            title={isDetailsPanelOpen ? "Hide details" : "Show details"}
                        >
                            <span className={`icon icon-details${isDetailsPanelOpen ? '-active' : ''}`}></span>
                        </button>
                        <button
                            className="icon-button"
                            onClick={toggleTerminal}
                            aria-label={isTerminalOpen ? "Hide terminal" : "Show terminal"}
                            title={isTerminalOpen ? "Hide terminal" : "Show terminal"}
                        >
                            <span className={`icon icon-terminal${isTerminalOpen ? '-active' : ''}`}></span>
                        </button>
                        <button
                            className="icon-button"
                            onClick={toggleTheme}
                            aria-label={theme === 'light' ? "Switch to dark theme" : "Switch to light theme"}
                            title={theme === 'light' ? "Switch to dark theme" : "Switch to light theme"}
                        >
                            <span className={`icon icon-${theme === 'light' ? 'moon' : 'sun'}`}></span>
                        </button>
                    </div>
                </div>

                {/* Main content with file list and optional details panel */}
                <div className="main-content">
                    {/* File list container */}
                    <div className="files-container">
                        {/* Action bar (Create new file/folder, etc.) */}
                        <div className="action-bar">
                            <CreateFileButton />
                        </div>

                        {/* File list view */}
                        <FileList
                            data={displayData}
                            isLoading={isLoading}
                            viewMode={viewMode}
                            isSearching={!!searchValue}
                        />
                    </div>

                    {/* Details panel (when selected) */}
                    {isDetailsPanelOpen && (
                        <>
                            <div className="panel-resize-handle"></div>
                            <DetailsPanel
                                item={selectedItems[0] || null}
                                isMultipleSelection={selectedItems.length > 1}
                            />
                        </>
                    )}
                </div>

                {/* Terminal (when opened) */}
                {isTerminalOpen && (
                    <Terminal />
                )}
            </div>

            {/* Context menu */}
            {isContextMenuOpen && (
                <ContextMenu
                    position={position}
                    items={items}
                    onClose={closeContextMenu}
                />
            )}
        </div>
    );
};

export default MainLayout;