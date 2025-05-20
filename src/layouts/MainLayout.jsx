import React, { useState, useCallback, useEffect } from 'react';
import { useTheme } from '../providers/ThemeProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import { useContextMenu } from '../providers/ContextMenuProvider';
import { useHistory } from '../providers/HistoryProvider';

// Core Components
import Sidebar from '../components/sidebar/Sidebar';
import PathBreadcrumb from '../components/explorer/PathBreadcrumb';
import NavigationButtons from '../components/explorer/NavigationButtons';
import FileList from '../components/explorer/FileList';
import SearchBar from '../components/search/SearchBar';
import DetailsPanel from '../components/explorer/DetailsPanel';
import ContextMenu from '../components/contextMenu/ContextMenu';
import ViewModes from '../components/explorer/ViewModes';

//  Components
import CreateFileButton from '../components/explorer/CreateFileButton';
import Terminal from '../components/terminal/Terminal';
import TabManager from '../components/tabs/TabManager';
import GlobalSearch from '../components/search/GlobalSearch';
import SettingsPanel from '../components/settings/SettingsPanel';
import ThisPCView from '../components/thisPc/ThisPCView';
import TemplateList from '../components/templates/TemplateList';

import '../styles/layouts/mainLayout.css';

const MainLayout = () => {
    const { theme, toggleTheme } = useTheme();
    const { isLoading, currentDirData, selectedItems, loadDirectory, volumes } = useFileSystem();
    const { isOpen: isContextMenuOpen, position, items, closeContextMenu } = useContextMenu();
    const { currentPath } = useHistory();

    // UI State
    const [isDetailsPanelOpen, setIsDetailsPanelOpen] = useState(false);
    const [isTerminalOpen, setIsTerminalOpen] = useState(false);
    const [viewMode, setViewMode] = useState('grid');
    const [searchValue, setSearchValue] = useState('');
    const [searchResults, setSearchResults] = useState(null);
    const [currentView, setCurrentView] = useState('explorer'); // 'explorer', 'this-pc', 'templates'

    // Modal states
    const [isGlobalSearchOpen, setIsGlobalSearchOpen] = useState(false);
    const [isSettingsOpen, setIsSettingsOpen] = useState(false);
    const [isTemplatesOpen, setIsTemplatesOpen] = useState(false);

    // Load default location on first render
    useEffect(() => {
        if (volumes.length > 0 && !currentDirData && !currentPath) {
            // Show This PC view by default
            setCurrentView('this-pc');
        }
    }, [volumes, currentDirData, currentPath]);

    // Listen for custom events
    useEffect(() => {
        const handleOpenTemplates = () => {
            setIsTemplatesOpen(true);
        };

        const handleShowProperties = (e) => {
            setIsDetailsPanelOpen(true);
        };

        document.addEventListener('open-templates', handleOpenTemplates);
        document.addEventListener('show-properties', handleShowProperties);

        return () => {
            document.removeEventListener('open-templates', handleOpenTemplates);
            document.removeEventListener('show-properties', handleShowProperties);
        };
    }, []);

    // Handle search
    const handleSearch = useCallback((value) => {
        setSearchValue(value);

        if (!value.trim()) {
            setSearchResults(null);
            return;
        }

        // Simple local search for now
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

    // Handle keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e) => {
            // Global search: Ctrl+Shift+F
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
                e.preventDefault();
                setIsGlobalSearchOpen(true);
            }

            // Settings: Ctrl+,
            if ((e.ctrlKey || e.metaKey) && e.key === ',') {
                e.preventDefault();
                setIsSettingsOpen(true);
            }

            // New folder: Ctrl+Shift+N
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'N') {
                e.preventDefault();
                document.dispatchEvent(new CustomEvent('create-folder'));
            }

            // New file: Ctrl+N
            if ((e.ctrlKey || e.metaKey) && e.key === 'n' && !e.shiftKey) {
                e.preventDefault();
                document.dispatchEvent(new CustomEvent('create-file'));
            }

            // Toggle terminal: Ctrl+`
            if ((e.ctrlKey || e.metaKey) && e.key === '`') {
                e.preventDefault();
                setIsTerminalOpen(prev => !prev);
            }

            // This PC: Ctrl+Shift+C
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'C') {
                e.preventDefault();
                setCurrentView('this-pc');
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, []);

    // Copy current path to clipboard
    const copyCurrentPath = useCallback(async () => {
        if (!currentPath) return;

        try {
            await navigator.clipboard.writeText(currentPath);
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
    }, [currentPath]);

    // Clear search when changing directory
    useEffect(() => {
        setSearchValue('');
        setSearchResults(null);
    }, [currentDirData]);

    // Get the data to display
    const displayData = searchResults || currentDirData;

    // Render main content based on current view
    const renderMainContent = () => {
        switch (currentView) {
            case 'this-pc':
                return <ThisPCView />;
            case 'templates':
                return <TemplateList onClose={() => setCurrentView('explorer')} />;
            default:
                return (
                    <div className="files-container">
                        <div className="action-bar">
                            <CreateFileButton />
                            <div className="action-divider"></div>
                            <button
                                className="copy-path-button"
                                onClick={copyCurrentPath}
                                title="Copy current path"
                            >
                                <span className="icon icon-copy"></span>
                                <span>Copy Path</span>
                            </button>
                        </div>

                        <FileList
                            data={displayData}
                            isLoading={isLoading}
                            viewMode={viewMode}
                            isSearching={!!searchValue}
                        />
                    </div>
                );
        }
    };

    return (
        <div className="main-layout">
            {/* Sidebar */}
            <Sidebar />

            {/* Main content area with tabs */}
            <div className="content-area">
                <TabManager>
                    {/* Toolbar with navigation and actions */}
                    <div className="toolbar">
                        <div className="toolbar-left">
                            <NavigationButtons />
                            <PathBreadcrumb />
                        </div>
                        <div className="toolbar-center">
                            <SearchBar
                                value={searchValue}
                                onChange={handleSearch}
                                placeholder="Search in current folder"
                            />
                        </div>
                        <div className="toolbar-right">
                            <button
                                className="icon-button"
                                onClick={() => setCurrentView('this-pc')}
                                title="This PC"
                                aria-label="This PC"
                            >
                                <span className="icon icon-computer"></span>
                            </button>

                            <button
                                className="icon-button"
                                onClick={() => setIsGlobalSearchOpen(true)}
                                title="Global Search (Ctrl+Shift+F)"
                                aria-label="Global Search"
                            >
                                <span className="icon icon-search-global"></span>
                            </button>

                            <ViewModes
                                currentMode={viewMode}
                                onChange={setViewMode}
                            />

                            <button
                                className={`icon-button ${isDetailsPanelOpen ? 'active' : ''}`}
                                onClick={() => setIsDetailsPanelOpen(!isDetailsPanelOpen)}
                                title="Details Panel"
                                aria-label="Toggle details panel"
                            >
                                <span className="icon icon-panel-right"></span>
                            </button>

                            <button
                                className={`icon-button ${isTerminalOpen ? 'active' : ''}`}
                                onClick={() => setIsTerminalOpen(!isTerminalOpen)}
                                title="Terminal (Ctrl+`)"
                                aria-label="Toggle terminal"
                            >
                                <span className="icon icon-terminal"></span>
                            </button>

                            <button
                                className="icon-button"
                                onClick={() => setIsSettingsOpen(true)}
                                title="Settings (Ctrl+,)"
                                aria-label="Settings"
                            >
                                <span className="icon icon-settings"></span>
                            </button>

                            <button
                                className="icon-button"
                                onClick={toggleTheme}
                                title={theme === 'light' ? 'Switch to dark theme' : 'Switch to light theme'}
                                aria-label="Toggle theme"
                            >
                                <span className={`icon icon-${theme === 'light' ? 'moon' : 'sun'}`}></span>
                            </button>
                        </div>
                    </div>

                    {/* Main content with file list and optional details panel */}
                    <div className="main-content">
                        {renderMainContent()}

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
                        <Terminal
                            isOpen={isTerminalOpen}
                            onToggle={() => setIsTerminalOpen(!isTerminalOpen)}
                        />
                    )}
                </TabManager>
            </div>

            {/* Context menu */}
            {isContextMenuOpen && (
                <ContextMenu
                    position={position}
                    items={items}
                    onClose={closeContextMenu}
                />
            )}

            {/* Modals */}
            <GlobalSearch
                isOpen={isGlobalSearchOpen}
                onClose={() => setIsGlobalSearchOpen(false)}
            />

            <SettingsPanel
                isOpen={isSettingsOpen}
                onClose={() => setIsSettingsOpen(false)}
            />

            {isTemplatesOpen && (
                <TemplateList onClose={() => setIsTemplatesOpen(false)} />
            )}
        </div>
    );
};

export default MainLayout;