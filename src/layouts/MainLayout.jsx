import React, { useState, useCallback, useEffect } from 'react';
import { useTheme } from '../providers/ThemeProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import { useContextMenu } from '../providers/ContextMenuProvider';
import { useHistory } from '../providers/HistoryProvider';
import { invoke } from '@tauri-apps/api/core';
import { showError, showConfirm, showSuccess } from '../utils/NotificationSystem';

// Core Components
import Sidebar from '../components/sidebar/Sidebar';
import PathBreadcrumb from '../components/explorer/PathBreadcrumb';
import NavigationButtons from '../components/explorer/NavigationButtons';
import FileList from '../components/explorer/FileList';
import SearchBar from '../components/search/SearchBar';
import DetailsPanel from '../components/explorer/DetailsPanel';
import ContextMenu from '../components/contextMenu/ContextMenu';
import ViewModes from '../components/explorer/ViewModes';

// Additional Components
import CreateFileButton from '../components/explorer/CreateFileButton';
import RenameModal from '../components/common/RenameModal';
import Terminal from '../components/terminal/Terminal';
import TabManager from '../components/tabs/TabManager';
import GlobalSearch from '../components/search/GlobalSearch';
import SettingsPanel from '../components/settings/SettingsPanel';
import ThisPCView from '../components/thisPc/ThisPCView';
import TemplateList from '../components/templates/TemplateList';

// Hash Modals
import HashFileModal from '../components/common/HashFileModal.jsx';
import HashCompareModal from '../components/common/HashCompareModal.jsx';

import '../styles/layouts/mainLayout.css';
import {replaceFileName} from "../utils/pathUtils.js";

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
    const [isRenameModalOpen, setIsRenameModalOpen] = useState(false);
    const [renameItem, setRenameItem] = useState(null);

    // Hash Modal states
    const [isHashFileModalOpen, setIsHashFileModalOpen] = useState(false);
    const [isHashCompareModalOpen, setIsHashCompareModalOpen] = useState(false);
    const [hashModalItem, setHashModalItem] = useState(null);

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
            setCurrentView('templates');
            setIsTemplatesOpen(true);
        };

        const handleShowProperties = (e) => {
            setIsDetailsPanelOpen(true);
        };

        const handleOpenThisPC = () => {
            setCurrentView('this-pc');
        };

        const handleOpenSettings = () => {
            setIsSettingsOpen(true);
        };

        const handleToggleTerminal = () => {
            setIsTerminalOpen(prev => !prev);
        };

        const handleOpenRenameModal = (e) => {
            if (e.detail && e.detail.item) {
                setRenameItem(e.detail.item);
                setIsRenameModalOpen(true);
            }
        };

        const handleOpenHashFileModal = (e) => {
            if (e.detail && e.detail.item) {
                setHashModalItem(e.detail.item);
                setIsHashFileModalOpen(true);
            }
        };

        const handleOpenHashCompareModal = (e) => {
            if (e.detail && e.detail.item) {
                setHashModalItem(e.detail.item);
                setIsHashCompareModalOpen(true);
            }
        };

        document.addEventListener('open-templates', handleOpenTemplates);
        document.addEventListener('show-properties', handleShowProperties);
        document.addEventListener('open-this-pc', handleOpenThisPC);
        document.addEventListener('open-settings', handleOpenSettings);
        document.addEventListener('toggle-terminal', handleToggleTerminal);
        document.addEventListener('open-rename-modal', handleOpenRenameModal);
        document.addEventListener('open-hash-file-modal', handleOpenHashFileModal);
        document.addEventListener('open-hash-compare-modal', handleOpenHashCompareModal);

        return () => {
            document.removeEventListener('open-templates', handleOpenTemplates);
            document.removeEventListener('show-properties', handleShowProperties);
            document.removeEventListener('open-this-pc', handleOpenThisPC);
            document.removeEventListener('open-settings', handleOpenSettings);
            document.removeEventListener('toggle-terminal', handleToggleTerminal);
            document.removeEventListener('open-rename-modal', handleOpenRenameModal);
            document.removeEventListener('open-hash-file-modal', handleOpenHashFileModal);
            document.removeEventListener('open-hash-compare-modal', handleOpenHashCompareModal);
        };
    }, []);

    // Switch to explorer view when navigating to a directory
    useEffect(() => {
        if (currentPath && currentView !== 'explorer') {
            setCurrentView('explorer');
        }
    }, [currentPath]);

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

            // Rename: F2
            if (e.key === 'F2' && selectedItems.length === 1) {
                e.preventDefault();
                document.dispatchEvent(new CustomEvent('open-rename-modal', {
                    detail: { item: selectedItems[0] }
                }));
            }

            // Toggle terminal: Ctrl+`
            if ((e.ctrlKey || e.metaKey) && e.key === '`') {
                e.preventDefault();
                setIsTerminalOpen(prev => !prev);
            }

            // Escape to clear selection
            if (e.key === 'Escape') {
                document.dispatchEvent(new CustomEvent('clear-selection'));
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [selectedItems]);

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

    // Handle rename
    const handleRename = async (item, newName) => {
        if (!newName || newName === item.name) return;

        console.log(`!!! Renaming "${replaceFileName(item.path, newName)}"`);

        try {
            const separator = item.path.includes('\\') ? '\\' : '/';

            console.log("Debug - separator detected:", separator);
            console.log("Debug - original path:", item.path);

            const pathParts = item.path.split(separator);
            pathParts[pathParts.length - 1] = newName;
            const newPath = pathParts.join(separator);

            console.log("Debug - new path:", newPath);

            await invoke('rename', {
                oldPath: item.path,
                newPath: newPath
            });

            // Reload current directory
            if (currentPath) {
                await loadDirectory(currentPath);
            }
        } catch (error) {
            console.error('Rename operation failed:', error);
            if (error.message && error.message.includes('already exists')) {
                const shouldCreateCopy = await showConfirm(`A file named "${newName}" already exists. Create a copy instead?`, 'File Exists');
                if (shouldCreateCopy) {
                    const extension = newName.includes('.') ? newName.split('.').pop() : '';
                    const baseName = extension ? newName.replace(`.${extension}`, '') : newName;
                    const copyName = extension ? `${baseName} - Copy.${extension}` : `${baseName} - Copy`;
                    handleRename(item, copyName);
                }
            } else {
                showError(`Failed to rename: ${error.message || error}`);
            }
        }
    };

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
                return <TemplateList onClose={() => {
                    setCurrentView('explorer');
                    setIsTemplatesOpen(false);
                }} />;
            default:
                return (
                    <div className="files-container">
                        <div className="action-bar">
                            <CreateFileButton />
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
            <Sidebar
                onTerminalToggle={() => setIsTerminalOpen(!isTerminalOpen)}
                isTerminalOpen={isTerminalOpen}
                currentView={currentView}
            />

            {/* Main content area with tabs */}
            <div className="content-area">
                <TabManager>
                    {/* Toolbar with navigation and actions */}
                    <div className="toolbar">
                        <div className="toolbar-left">
                            <NavigationButtons />
                            <PathBreadcrumb
                                onCopyPath={copyCurrentPath}
                                isVisible={currentView === 'explorer'}
                            />
                        </div>
                        <div className="toolbar-center">
                            {currentView === 'explorer' && (
                                <SearchBar
                                    value={searchValue}
                                    onChange={handleSearch}
                                    placeholder="Search in current folder"
                                />
                            )}
                        </div>
                        <div className="toolbar-right">
                            <button
                                className="icon-button"
                                onClick={() => setIsGlobalSearchOpen(true)}
                                title="Global Search (Ctrl+Shift+F)"
                                aria-label="Global Search"
                            >
                                <span className="icon icon-search-global"></span>
                            </button>

                            {currentView === 'explorer' && (
                                <ViewModes
                                    currentMode={viewMode}
                                    onChange={setViewMode}
                                />
                            )}

                            <button
                                className={`icon-button ${isDetailsPanelOpen ? 'active' : ''}`}
                                onClick={() => setIsDetailsPanelOpen(!isDetailsPanelOpen)}
                                title="Details Panel"
                                aria-label="Toggle details panel"
                            >
                                <span className="icon icon-panel-right"></span>
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

            <RenameModal
                isOpen={isRenameModalOpen}
                onClose={() => setIsRenameModalOpen(false)}
                item={renameItem}
                onRename={handleRename}
            />

            {/* Hash Modals */}
            <HashFileModal
                isOpen={isHashFileModalOpen}
                onClose={() => {
                    setIsHashFileModalOpen(false);
                    setHashModalItem(null);
                }}
                item={hashModalItem}
            />

            <HashCompareModal
                isOpen={isHashCompareModalOpen}
                onClose={() => {
                    setIsHashCompareModalOpen(false);
                    setHashModalItem(null);
                }}
                item={hashModalItem}
            />
        </div>
    );
};

export default MainLayout;