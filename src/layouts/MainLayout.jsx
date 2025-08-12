import React, { useState, useCallback, useEffect } from 'react';
import { useTheme } from '../providers/ThemeProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import { useContextMenu } from '../providers/ContextMenuProvider';
import { useHistory } from '../providers/HistoryProvider';
import { useSettings } from '../providers/SettingsProvider';
import { invoke } from '@tauri-apps/api/core';
import { showError, showConfirm, showSuccess } from '../utils/NotificationSystem';

// Core Components
import Sidebar from '../components/sidebar/Sidebar';
import PathBreadcrumb from '../components/explorer/PathBreadcrumb';
import NavigationButtons from '../components/explorer/NavigationButtons';
import FileList from '../components/explorer/FileList';
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
import PreviewModal from '../components/preview/PreviewModal';

// Hash Modals
import HashFileModal from '../components/common/HashFileModal.jsx';
import HashCompareModal from '../components/common/HashCompareModal.jsx';
import HashDisplayModal from '../components/common/HashDisplayModal.jsx';

// Settings Applier
import SettingsApplier from '../utils/SettingsApplier.js';

// Hooks
import { usePreview } from '../hooks/usePreview';

import '../styles/layouts/mainLayout.css';
import {replaceFileName} from "../utils/pathUtils.js";

/**
 * MainLayout component that serves as the primary layout structure for the application.
 * Manages UI state, event handling, and renders the main application interface.
 *
 * @returns {JSX.Element} The MainLayout component
 */
const MainLayout = () => {
    const { theme, toggleTheme } = useTheme();
    const { isLoading, currentDirData, selectedItems, loadDirectory, volumes, focusedItem, setFocusedItem } = useFileSystem();
    const { 
        isOpen: isContextMenuOpen, 
        position, 
        items, 
        closeContextMenu, 
        clipboard,
        copyToClipboard,
        cutToClipboard,
        pasteFromClipboard,
        deleteItems,
        renameItem,
        showProperties
    } = useContextMenu();
    const { currentPath } = useHistory();
    const { settings, updateSetting } = useSettings();

    // UI State - Initialize from settings
    const [isDetailsPanelOpen, setIsDetailsPanelOpen] = useState(settings.show_details_panel || false);
    const [isTerminalOpen, setIsTerminalOpen] = useState(false);
    const [viewMode, setViewMode] = useState(settings.default_view || 'grid');
    const [searchValue, setSearchValue] = useState('');
    const [searchResults, setSearchResults] = useState(null);
    const [currentView, setCurrentView] = useState('explorer'); // 'explorer', 'this-pc', 'templates'

    // Modal states
    const [isGlobalSearchOpen, setIsGlobalSearchOpen] = useState(false);
    const [isSettingsOpen, setIsSettingsOpen] = useState(false);
    const [isTemplatesOpen, setIsTemplatesOpen] = useState(false);
    const [isRenameModalOpen, setIsRenameModalOpen] = useState(false);
    const [itemToRename, setItemToRename] = useState(null);

    // Hash Modal states
    const [isHashFileModalOpen, setIsHashFileModalOpen] = useState(false);
    const [isHashCompareModalOpen, setIsHashCompareModalOpen] = useState(false);
    const [isHashDisplayModalOpen, setIsHashDisplayModalOpen] = useState(false);
    const [hashModalItem, setHashModalItem] = useState(null);
    const [hashDisplayData, setHashDisplayData] = useState({ hash: '', fileName: '' });
    const [columnsPerRow, setColumnsPerRow] = useState(4); // Track columns for preview navigation

    // Get terminal height for padding calculations
    const terminalHeight = settings.terminal_height || 240;

    // Get sorted data the same way FileList does
    const getSortedData = useCallback(() => {
        const data = searchResults || currentDirData;
        if (!data || (!data.directories?.length && !data.files?.length)) {
            return [];
        }

        // Combine directories and files for sorting (same as FileList)
        const combinedItems = [
            ...(data.directories || []).map(dir => ({ ...dir, isDirectory: true })),
            ...(data.files || []).map(file => ({ ...file, isDirectory: false }))
        ];

        // Sort by name with directories first (same as FileList default)
        const sortedItems = [...combinedItems].sort((a, b) => {
            // Directories always come before files
            if (a.isDirectory && !b.isDirectory) return -1;
            if (!a.isDirectory && b.isDirectory) return 1;

            // Sort by name
            const aName = a.name.toLowerCase();
            const bName = b.name.toLowerCase();
            return aName.localeCompare(bName);
        });

        return sortedItems;
    }, [searchResults, currentDirData]);

    // Initialize preview functionality
    const getFocusedItem = () => {
        // For search results, use the focused item from search
        if (searchResults && searchResults.length > 0) {
            // This would need to be implemented in search components
            return null; // Placeholder for now
        }
        
        // For regular file list, use the focused item from FileSystemProvider
        return focusedItem;
    };

    // 2D Grid navigation functions using sorted data
    const navigateUp = useCallback(() => {
        const sortedItems = getSortedData();
        if (!sortedItems.length) return;
        
        const currentIndex = focusedItem ? sortedItems.findIndex(item => item.path === focusedItem.path) : -1;
        const newIndex = currentIndex - columnsPerRow;
        
        if (newIndex >= 0) {
            setFocusedItem(sortedItems[newIndex]);
        } else {
            // Wrap to bottom row
            const remainder = currentIndex % columnsPerRow;
            const totalRows = Math.ceil(sortedItems.length / columnsPerRow);
            const lastRowStartIndex = (totalRows - 1) * columnsPerRow;
            const targetIndex = Math.min(lastRowStartIndex + remainder, sortedItems.length - 1);
            setFocusedItem(sortedItems[targetIndex]);
        }
    }, [getSortedData, focusedItem, setFocusedItem, columnsPerRow]);

    const navigateDown = useCallback(() => {
        const sortedItems = getSortedData();
        if (!sortedItems.length) return;
        
        const currentIndex = focusedItem ? sortedItems.findIndex(item => item.path === focusedItem.path) : -1;
        const newIndex = currentIndex + columnsPerRow;
        
        if (newIndex < sortedItems.length) {
            setFocusedItem(sortedItems[newIndex]);
        } else {
            // Wrap to top row
            const remainder = currentIndex % columnsPerRow;
            setFocusedItem(sortedItems[remainder]);
        }
    }, [getSortedData, focusedItem, setFocusedItem, columnsPerRow]);

    const navigateLeft = useCallback(() => {
        const sortedItems = getSortedData();
        if (!sortedItems.length) return;
        
        const currentIndex = focusedItem ? sortedItems.findIndex(item => item.path === focusedItem.path) : -1;
        
        if (currentIndex % columnsPerRow === 0) {
            // At leftmost column, wrap to rightmost of same row or previous row
            const currentRow = Math.floor(currentIndex / columnsPerRow);
            const nextRowLastIndex = Math.min((currentRow + 1) * columnsPerRow - 1, sortedItems.length - 1);
            setFocusedItem(sortedItems[nextRowLastIndex]);
        } else {
            setFocusedItem(sortedItems[currentIndex - 1]);
        }
    }, [getSortedData, focusedItem, setFocusedItem, columnsPerRow]);

    const navigateRight = useCallback(() => {
        const sortedItems = getSortedData();
        if (!sortedItems.length) return;
        
        const currentIndex = focusedItem ? sortedItems.findIndex(item => item.path === focusedItem.path) : -1;
        
        if ((currentIndex + 1) % columnsPerRow === 0 || currentIndex === sortedItems.length - 1) {
            // At rightmost column or last item, wrap to leftmost of same row
            const currentRow = Math.floor(currentIndex / columnsPerRow);
            const rowStartIndex = currentRow * columnsPerRow;
            setFocusedItem(sortedItems[rowStartIndex]);
        } else {
            setFocusedItem(sortedItems[currentIndex + 1]);
        }
    }, [getSortedData, focusedItem, setFocusedItem, columnsPerRow]);

    const { 
        open: isPreviewOpen, 
        payload: previewPayload, 
        isLoading: isPreviewLoading,
        closePreview 
    } = usePreview(getFocusedItem, navigateUp, navigateDown, navigateLeft, navigateRight);

    /**
     * Effect to update UI state when settings change
     */
    useEffect(() => {
        if (settings.show_details_panel !== undefined) {
            setIsDetailsPanelOpen(settings.show_details_panel);
        }
    }, [settings.show_details_panel]);

    /**
     * Effect to update view mode when settings change
     */
    useEffect(() => {
        if (settings.default_view) {
            setViewMode(settings.default_view);
        }
    }, [settings.default_view]);

    /**
     * Effect to load default location on first render
     */
    useEffect(() => {
        if (volumes.length > 0 && !currentDirData && !currentPath) {
            // Show This PC view by default
            setCurrentView('this-pc');
        }
    }, [volumes, currentDirData, currentPath]);

    /**
     * Effect to auto-start indexing when app loads
     */
    useEffect(() => {
        const initializeSearchEngine = async () => {
            if (volumes.length > 0) {
                try {
                    console.log('MainLayout: Checking search engine status for auto-indexing...');
                    
                    // Check if search engine has indexed files
                    const searchEngineInfo = await invoke('get_search_engine_info');
                    const hasNoIndexedFiles = !searchEngineInfo.stats?.trie_size || searchEngineInfo.stats.trie_size === 0;
                    
                    console.log('MainLayout: Search engine info:', searchEngineInfo);
                    console.log('MainLayout: Has no indexed files:', hasNoIndexedFiles);

                    if (hasNoIndexedFiles) {
                        console.log('MainLayout: Starting auto-indexing of home directory...');
                        
                        // Get system info to get the proper home directory
                        const metaDataJson = await invoke('get_meta_data_as_json');
                        const metaData = JSON.parse(metaDataJson);
                        
                        if (!metaData.user_home_dir) {
                            console.error('MainLayout: User home directory not available');
                            return;
                        }

                        console.log('MainLayout: Using home directory:', metaData.user_home_dir);
                        
                        // Auto-index home directory on app startup
                        const result = await invoke('add_paths_recursive_async', {
                            folder: metaData.user_home_dir
                        });
                        
                        console.log('MainLayout: Auto-indexing initiated:', result);
                        showSuccess('Background indexing started for your home directory');
                    } else {
                        console.log('MainLayout: Search engine already has indexed files, skipping auto-indexing');
                    }
                } catch (error) {
                    console.error('MainLayout: Auto-indexing failed:', error);
                    // Don't show error to user as this is background operation
                }
            }
        };

        // Only run once when volumes are first loaded
        if (volumes.length > 0) {
            initializeSearchEngine();
        }
    }, [volumes.length]); // Only depend on volumes.length to avoid re-running

    /**
     * Effect to listen for custom events
     * Improved with debug information
     */
    useEffect(() => {
        /**
         * Handler for opening templates view
         */
        const handleOpenTemplates = () => {
            setCurrentView('templates');
            setIsTemplatesOpen(true);
        };

        /**
         * Handler for showing properties panel
         */
        const handleShowProperties = (e) => {
            setIsDetailsPanelOpen(true);
        };

        /**
         * Handler for opening This PC view
         */
        const handleOpenThisPC = () => {
            setCurrentView('this-pc');
        };

        /**
         * Handler for opening settings panel
         */
        const handleOpenSettings = () => {
            setIsSettingsOpen(true);
        };

        /**
         * Handler for toggling terminal visibility
         */
        const handleToggleTerminal = () => {
            setIsTerminalOpen(prev => !prev);
        };

        /**
         * Handler for opening rename modal
         * @param {CustomEvent} e - Event with item details
         */
        const handleOpenRenameModal = (e) => {
            if (e.detail && e.detail.item) {
                // Close any existing modal first to prevent duplicates
                setIsRenameModalOpen(false);
                setItemToRename(null);
                
                // Small delay to ensure cleanup, then open new modal
                setTimeout(() => {
                    setItemToRename(e.detail.item);
                    setIsRenameModalOpen(true);
                }, 10);
            }
        };

        // Improved hash event handlers with debug information
        /**
         * Handler for opening hash file modal
         * @param {CustomEvent} e - Event with item details
         */
        const handleOpenHashFileModal = (e) => {
            console.log('MainLayout: Received open-hash-file-modal event:', e.detail);
            if (e.detail && e.detail.item) {
                console.log('Opening Hash File Modal for:', e.detail.item.name);
                setHashModalItem(e.detail.item);
                setIsHashFileModalOpen(true);
            } else {
                console.log('Invalid event detail:', e.detail);
            }
        };

        /**
         * Handler for opening hash compare modal
         * @param {CustomEvent} e - Event with item details
         */
        const handleOpenHashCompareModal = (e) => {
            console.log('MainLayout: Received open-hash-compare-modal event:', e.detail);
            if (e.detail && e.detail.item) {
                console.log('Opening Hash Compare Modal for:', e.detail.item.name);
                setHashModalItem(e.detail.item);
                setIsHashCompareModalOpen(true);
            } else {
                console.log('Invalid event detail:', e.detail);
            }
        };

        // Hash Display Modal Handler
        const handleOpenHashDisplayModal = (e) => {
            console.log('Opening Hash Display Modal:', e.detail);
            if (e.detail?.hash && e.detail?.fileName) {
                setHashDisplayData({ hash: e.detail.hash, fileName: e.detail.fileName });
                setIsHashDisplayModalOpen(true);
            } else {
                console.log('Invalid hash display event detail:', e.detail);
            }
        };

        // Register event listeners
        document.addEventListener('open-templates', handleOpenTemplates);
        document.addEventListener('show-properties', handleShowProperties);
        document.addEventListener('open-this-pc', handleOpenThisPC);
        document.addEventListener('open-settings', handleOpenSettings);
        document.addEventListener('toggle-terminal', handleToggleTerminal);
        document.addEventListener('open-rename-modal', handleOpenRenameModal);
        document.addEventListener('open-hash-file-modal', handleOpenHashFileModal);
        document.addEventListener('open-hash-compare-modal', handleOpenHashCompareModal);
        document.addEventListener('open-hash-display-modal', handleOpenHashDisplayModal);

        console.log('MainLayout: All event listeners registered');

        return () => {
            document.removeEventListener('open-templates', handleOpenTemplates);
            document.removeEventListener('show-properties', handleShowProperties);
            document.removeEventListener('open-this-pc', handleOpenThisPC);
            document.removeEventListener('open-settings', handleOpenSettings);
            document.removeEventListener('toggle-terminal', handleToggleTerminal);
            document.removeEventListener('open-rename-modal', handleOpenRenameModal);
            document.removeEventListener('open-hash-file-modal', handleOpenHashFileModal);
            document.removeEventListener('open-hash-compare-modal', handleOpenHashCompareModal);
            document.removeEventListener('open-hash-display-modal', handleOpenHashDisplayModal);
            console.log('MainLayout: All event listeners removed');
        };
    }, []);

    /**
     * Effect to switch to explorer view when navigating to a directory
     */
    useEffect(() => {
        if (currentPath && currentView !== 'explorer') {
            setCurrentView('explorer');
        }
    }, [currentPath]);

    /**
     * Handles search functionality
     * @param {string} value - The search query
     */
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

    /**
     * Effect to handle keyboard shortcuts
     */
    useEffect(() => {
        /**
         * Keyboard event handler for shortcuts
         * @param {KeyboardEvent} e - The keyboard event
         */
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

    /**
     * Copies current path to clipboard
     */
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

    /**
     * Handles renaming a file or directory
     * @param {Object} item - The item to rename
     * @param {string} newName - The new name
     */
    const handleRename = async (item, newName) => {
        if (!newName || newName === item.name) return;

        console.log(`Renaming "${replaceFileName(item.path, newName)}"`);

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

    /**
     * Handles view mode change with settings persistence
     * @param {string} newMode - The new view mode
     */
    const handleViewModeChange = useCallback(async (newMode) => {
        setViewMode(newMode);

        // Save to settings
        try {
            await invoke('update_settings_field', { key: 'default_view', value: newMode });
        } catch (error) {
            console.error('Failed to save view mode setting:', error);
        }
    }, []);

    /**
     * Handles details panel toggle with settings persistence
     */
    const handleDetailsPanelToggle = useCallback(async () => {
        const newState = !isDetailsPanelOpen;
        setIsDetailsPanelOpen(newState);

        // Save to settings
        try {
            await invoke('update_settings_field', { key: 'show_details_panel', value: newState });
        } catch (error) {
            console.error('Failed to save details panel setting:', error);
        }
    }, [isDetailsPanelOpen]);

    /**
     * Handles hidden files visibility toggle with settings persistence
     */
    const handleHiddenFilesToggle = useCallback(async () => {
        const newState = !settings.show_hidden_files_and_folders;
        
        try {
            await updateSetting('show_hidden_files_and_folders', newState);
            // Reload the current directory to reflect the change
            if (currentPath) {
                await loadDirectory(currentPath);
            }
        } catch (error) {
            console.error('Failed to toggle hidden files setting:', error);
        }
    }, [settings.show_hidden_files_and_folders, updateSetting, currentPath, loadDirectory]);

    /**
     * File operation handlers - use the existing context menu functionality
     */
    const handleCut = useCallback(() => {
        if (selectedItems.length === 0) return;
        cutToClipboard(selectedItems);
    }, [selectedItems, cutToClipboard]);

    const handleCopy = useCallback(() => {
        if (selectedItems.length === 0) return;
        copyToClipboard(selectedItems);
    }, [selectedItems, copyToClipboard]);

    const handlePaste = useCallback(() => {
        if (!clipboard.items || clipboard.items.length === 0) return;
        pasteFromClipboard();
    }, [clipboard.items, pasteFromClipboard]);

    const handleRenameToolbar = useCallback(() => {
        if (selectedItems.length !== 1) return;
        // Directly dispatch the event instead of calling renameItem to avoid conflicts
        document.dispatchEvent(new CustomEvent('open-rename-modal', {
            detail: { item: selectedItems[0] }
        }));
    }, [selectedItems]);

    const handleDelete = useCallback(() => {
        if (selectedItems.length === 0) return;
        deleteItems(selectedItems);
    }, [selectedItems, deleteItems]);

    const handleProperties = useCallback(() => {
        if (selectedItems.length === 0) return;
        showProperties(selectedItems[0]);
    }, [selectedItems, showProperties]);

    /**
     * Effect to clear search when changing directory
     */
    useEffect(() => {
        setSearchValue('');
        setSearchResults(null);
    }, [currentDirData]);

    // Get the data to display
    const displayData = searchResults || currentDirData;

    /**
     * Renders the main content based on current view
     * @returns {JSX.Element} The main content component
     */
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
                            <div className="action-bar-left">
                                <CreateFileButton />
                                
                                <div className="action-divider"></div>
                                
                                <button
                                    className="icon-button"
                                    onClick={handleCut}
                                    disabled={selectedItems.length === 0}
                                    title="Cut (Ctrl+X)"
                                    aria-label="Cut selected items"
                                >
                                    <span className="icon icon-cut"></span>
                                </button>
                                
                                <button
                                    className="icon-button"
                                    onClick={handleCopy}
                                    disabled={selectedItems.length === 0}
                                    title="Copy (Ctrl+C)"
                                    aria-label="Copy selected items"
                                >
                                    <span className="icon icon-copy"></span>
                                </button>
                                
                                <button
                                    className="icon-button"
                                    onClick={handlePaste}
                                    disabled={!clipboard.items || clipboard.items.length === 0}
                                    title="Paste (Ctrl+V)"
                                    aria-label="Paste items"
                                >
                                    <span className="icon icon-paste"></span>
                                </button>
                                
                                <button
                                    className="icon-button"
                                    onClick={handleRenameToolbar}
                                    disabled={selectedItems.length !== 1}
                                    title="Rename (F2)"
                                    aria-label="Rename selected item"
                                >
                                    <span className="icon icon-rename"></span>
                                </button>
                                
                                <button
                                    className="icon-button"
                                    onClick={handleDelete}
                                    disabled={selectedItems.length === 0}
                                    title="Delete (Del)"
                                    aria-label="Delete selected items"
                                >
                                    <span className="icon icon-trash"></span>
                                </button>
                                
                                <button
                                    className="icon-button"
                                    onClick={handleProperties}
                                    disabled={selectedItems.length === 0}
                                    title="Properties (Alt+Enter)"
                                    aria-label="Show properties"
                                >
                                    <span className="icon icon-properties"></span>
                                </button>
                            </div>

                            <div className="action-bar-right">
                                {currentView === 'explorer' && (
                                    <ViewModes
                                        currentMode={viewMode}
                                        onChange={handleViewModeChange}
                                    />
                                )}
                                
                                <button
                                    className="icon-button"
                                    onClick={toggleTheme}
                                    title={theme === 'light' ? 'Switch to dark theme' : 'Switch to light theme'}
                                    aria-label={`Switch to ${theme === 'light' ? 'dark' : 'light'} theme`}
                                >
                                    <span className={`icon ${theme === 'light' ? 'icon-moon' : 'icon-sun'}`}></span>
                                </button>
                                
                                <button
                                    className="icon-button toggle-hidden-files"
                                    onClick={handleHiddenFilesToggle}
                                    title={`${settings.show_hidden_files_and_folders ? 'Hide' : 'Show'} hidden files and folders`}
                                    aria-label="Toggle hidden files visibility"
                                >
                                    <span className={`icon ${settings.show_hidden_files_and_folders ? 'icon-eye' : 'icon-eye-off'}`}></span>
                                </button>
                                
                                <button
                                    className={`icon-button ${isDetailsPanelOpen ? 'active' : ''}`}
                                    onClick={handleDetailsPanelToggle}
                                    title="Details Panel"
                                    aria-label="Toggle details panel"
                                >
                                    <span className="icon icon-panel-right"></span>
                                </button>
                            </div>
                        </div>

                        <FileList
                            data={displayData}
                            isLoading={isLoading}
                            viewMode={viewMode}
                            isSearching={!!searchValue}
                            searchTerm={searchValue}
                            disableArrowKeys={isPreviewOpen}
                            onColumnsChange={setColumnsPerRow}
                        />
                    </div>
                );
        }
    };

    return (
        <div className={`main-layout ${isTerminalOpen ? 'with-terminal' : ''}`}>
            {/* Settings Applier - applies settings to DOM */}
            <SettingsApplier />

            {/* Full-width tabs at the top */}
            <TabManager>
                {/* Full-width toolbar */}
                <div className="toolbar">
                        <div className="toolbar-left">
                            <NavigationButtons />
                            <PathBreadcrumb
                                onCopyPath={copyCurrentPath}
                                isVisible={currentView === 'explorer'}
                                onSearch={handleSearch}
                            />
                        </div>
                        <div className="toolbar-center">
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



                        </div>
                </div>

                <div className="layout-content">
                    {/* Sidebar */}
                    <Sidebar
                        onTerminalToggle={() => setIsTerminalOpen(!isTerminalOpen)}
                        isTerminalOpen={isTerminalOpen}
                        currentView={currentView}
                    />

                    {/* Main content area */}
                    <div className="content-area">
                        {/* Main content with file list and optional details panel */}
                    <div
                        className="main-content"
                        style={isTerminalOpen ? { paddingBottom: `${terminalHeight}px` } : {}}
                    >
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

                    {/* Terminal positioned absolutely at the bottom */}
                    <div 
                        className="terminal-wrapper"
                        style={{
                            position: isTerminalOpen ? 'absolute' : 'static',
                            bottom: isTerminalOpen ? 0 : 'auto',
                            left: isTerminalOpen ? 0 : 'auto',
                            right: isTerminalOpen ? 0 : 'auto',
                            height: isTerminalOpen ? `${terminalHeight}px` : 0,
                            overflow: isTerminalOpen ? 'visible' : 'hidden'
                        }}
                    >
                        {isTerminalOpen && (
                            <Terminal
                                isOpen={isTerminalOpen}
                                onToggle={() => setIsTerminalOpen(!isTerminalOpen)}
                            />
                        )}
                    </div>
                    </div>
                </div>
            </TabManager>

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
                item={itemToRename}
                onRename={handleRename}
            />

            {/* Hash Modals - with debug info */}
            <HashFileModal
                isOpen={isHashFileModalOpen}
                onClose={() => {
                    console.log('Closing Hash File Modal');
                    setIsHashFileModalOpen(false);
                    setHashModalItem(null);
                }}
                item={hashModalItem}
            />

            <HashCompareModal
                isOpen={isHashCompareModalOpen}
                onClose={() => {
                    console.log('Closing Hash Compare Modal');
                    setIsHashCompareModalOpen(false);
                    setHashModalItem(null);
                }}
                item={hashModalItem}
            />

            <HashDisplayModal
                isOpen={isHashDisplayModalOpen}
                onClose={() => {
                    console.log('Closing Hash Display Modal');
                    setIsHashDisplayModalOpen(false);
                    setHashDisplayData({ hash: '', fileName: '' });
                }}
                hash={hashDisplayData.hash}
                fileName={hashDisplayData.fileName}
            />

            {/* Preview Modal */}
            {(isPreviewOpen || isPreviewLoading) && (
                <PreviewModal
                    payload={previewPayload}
                    onClose={closePreview}
                    isLoading={isPreviewLoading}
                />
            )}
        </div>
    );
};

export default MainLayout;