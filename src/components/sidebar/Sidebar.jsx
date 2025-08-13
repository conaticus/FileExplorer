import React, { useState, useEffect, useRef } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { useContextMenu } from '../../providers/ContextMenuProvider';
import SidebarItem from './SidebarItem';
import Favorites from './Favorites';
import QuickAccess from './QuickAccess';
import Modal from '../common/Modal';
import Button from '../common/Button';
import { ask, message } from '@tauri-apps/plugin-dialog';
import './sidebar.css';

/**
 * Sidebar component - Provides navigation, favorites, and quick access
 *
 * @param {Object} props - Component props
 * @param {Function} props.onTerminalToggle - Callback to toggle terminal visibility
 * @param {boolean} props.isTerminalOpen - Whether the terminal is currently open
 * @param {string} props.currentView - Current view being displayed (e.g., 'this-pc')
 * @returns {React.ReactElement} Sidebar component
 */
const Sidebar = ({ onTerminalToggle, isTerminalOpen, currentView }) => {
    const { volumes, loadDirectory, loadVolumes } = useFileSystem();
    const { currentPath, navigateTo } = useHistory();
    const { removeFromFavorites } = useContextMenu();

    const [systemInfo, setSystemInfo] = useState(null);
    const [isAddSourceModalOpen, setIsAddSourceModalOpen] = useState(false);
    const [newSourcePath, setNewSourcePath] = useState('');
    const addSourceInputRef = useRef(null);

    // State for collapsible sections
    const [sectionCollapsed, setSectionCollapsed] = useState(() => {
        const saved = localStorage.getItem('sidebarSectionsCollapsed');
        return saved ? JSON.parse(saved) : {
            quickAccess: false,
            thisPC: false,
            favorites: false,
            drives: false
        };
    });

    /**
     * Load system info to get proper user directories
     */
    useEffect(() => {
        const loadSystemInfo = async () => {
            try {
                const { invoke } = await import('@tauri-apps/api/core');
                const metaDataJson = await invoke('get_meta_data_as_json');
                const metaData = JSON.parse(metaDataJson);
                setSystemInfo(metaData);
            } catch (error) {
                console.error('Failed to load system info:', error);
                // Mock data for development
                setSystemInfo({
                    current_running_os: 'windows',
                    user_home_dir: 'C:\\Users\\User'
                });
            }
        };

        loadSystemInfo();
    }, []);



    /**
     * Toggles a specific section's collapse state
     * @param {string} section - Section name to toggle
     */
    const toggleSectionCollapse = (section) => {
        const newSectionCollapsed = {
            ...sectionCollapsed,
            [section]: !sectionCollapsed[section]
        };
        setSectionCollapsed(newSectionCollapsed);
        localStorage.setItem('sidebarSectionsCollapsed', JSON.stringify(newSectionCollapsed));
    };

    /**
     * Handles clicking on a sidebar item with navigation history update
     * @param {string} path - Path to navigate to
     */
    const handleItemClick = async (path) => {
        // Always update navigation history and reload directory, even if path is the same
        try {
            const existingHistory = JSON.parse(sessionStorage.getItem('fileExplorerHistory') || '[]');
            const updatedHistory = [path, ...existingHistory.filter(p => p !== path)].slice(0, 10);
            sessionStorage.setItem('fileExplorerHistory', JSON.stringify(updatedHistory));

            // Dispatch events to update quick access immediately
            window.dispatchEvent(new CustomEvent('navigation-changed'));
            window.dispatchEvent(new CustomEvent('quick-access-updated'));
        } catch (err) {
            console.error('Failed to update navigation history:', err);
        }

        // Always refresh disks when opening a directory
        await loadVolumes();
        // Always reload the directory, even if it's already selected
        window.dispatchEvent(new CustomEvent('force-explorer-view'));
        await loadDirectory(path);
    };
    // Refresh disks when switching to 'this-pc' or 'explorer' view
    React.useEffect(() => {
        if (currentView === 'this-pc' || currentView === 'explorer') {
            loadVolumes();
        }
    }, [currentView, loadVolumes]);

    /**
     * Adds a location to favorites
     * @param {Object} location - Location object to add to favorites
     * @param {string} location.path - Path to the location
     * @param {string} location.name - Display name for the location
     * @param {string} location.icon - Icon to use for the location
     */
    const addToFavorites = (location) => {
        try {
            const existingFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');
            const newFavorites = [...existingFavorites, location];
            localStorage.setItem('fileExplorerFavorites', JSON.stringify(newFavorites));

            // Dispatch events to update favorites immediately
            window.dispatchEvent(new CustomEvent('favorites-updated'));
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerFavorites',
                newValue: JSON.stringify(newFavorites)
            }));
        } catch (error) {
            console.error('Failed to add to favorites:', error);
        }
    };

    /**
     * Removes a location from favorites using the context menu provider
     * @param {string} path - Path to remove from favorites
     */
    const handleRemoveFromFavorites = (path) => {
        removeFromFavorites(path);
    };

    /**
     * Opens the add source modal
     */
    const handleAddSource = () => {
        setNewSourcePath('');
        setIsAddSourceModalOpen(true);

        // Focus input after modal opens
        setTimeout(() => {
            if (addSourceInputRef.current) {
                addSourceInputRef.current.focus();
            }
        }, 100);
    };

    /**
     * Saves a new source to favorites
     */
    const saveNewSource = () => {
        if (!newSourcePath.trim()) return;

        const sourcePath = newSourcePath.trim();
        addToFavorites({
            name: sourcePath.split(/[/\\]/).pop() || 'New Source',
            path: sourcePath,
            icon: 'folder'
        });

        setIsAddSourceModalOpen(false);
        setNewSourcePath('');
    };

    /**
     * Handles form submission for add source
     * @param {React.FormEvent} e - Form submit event
     */
    const handleAddSourceSubmit = (e) => {
        e.preventDefault();
        saveNewSource();
    };

    /**
     * Handles input change for add source
     * @param {React.ChangeEvent<HTMLInputElement>} e - Input change event
     */
    const handleAddSourceInputChange = (e) => {
        setNewSourcePath(e.target.value);
    };

    /**
     * Handles key down for add source input
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleAddSourceKeyDown = (e) => {
        if (e.key === 'Escape') {
            setIsAddSourceModalOpen(false);
        }
    };

    /**
     * Gets user directories based on OS
     * @returns {Array<{name: string, path: string, icon: string}>} Array of user directory objects
     */
    const getUserDirectories = () => {
        if (!systemInfo) return [];

        const dirs = [];
        const homeDir = systemInfo.user_home_dir;
        const os = systemInfo.current_running_os;

        if (os === 'windows') {
            dirs.push(
                { name: 'Desktop', path: `${homeDir}\\Desktop`, icon: 'desktop' },
                { name: 'Documents', path: `${homeDir}\\Documents`, icon: 'documents' },
                { name: 'Downloads', path: `${homeDir}\\Downloads`, icon: 'downloads' },
                { name: 'Pictures', path: `${homeDir}\\Pictures`, icon: 'pictures' },
                { name: 'Music', path: `${homeDir}\\Music`, icon: 'music' },
                { name: 'Videos', path: `${homeDir}\\Videos`, icon: 'videos' }
            );
        } else {
            dirs.push(
                { name: 'Desktop', path: `${homeDir}/Desktop`, icon: 'desktop' },
                { name: 'Documents', path: `${homeDir}/Documents`, icon: 'documents' },
                { name: 'Downloads', path: `${homeDir}/Downloads`, icon: 'downloads' },
                { name: 'Pictures', path: `${homeDir}/Pictures`, icon: 'pictures' },
                { name: 'Music', path: `${homeDir}/Music`, icon: 'music' },
                { name: 'Videos', path: `${homeDir}/Movies`, icon: 'videos' }
            );
        }

        return dirs;
    };

    const userDirectories = getUserDirectories();

    /**
     * Gets user volume (for macOS dual mount handling)
     * @returns {Object|null} User volume object or null if not found
     */
    const getUserVolume = () => {
        if (!systemInfo || !volumes.length) return null;

        const homeDir = systemInfo.user_home_dir;

        // Find volume that contains user directory but is not root
        const userVolume = volumes.find(vol =>
            homeDir.startsWith(vol.mount_point) && vol.mount_point !== '/'
        );

        return userVolume;
    };

    const userVolume = getUserVolume();

    return (
        <>
            <aside className="sidebar">
                <div className="sidebar-content">
                    {/* Quick Access section */}
                    {/*
                    <section className="sidebar-section">
                        <div className="sidebar-section-header">
                            <h3 className="sidebar-section-title">Quick Access</h3>
                            <button
                                className="section-collapse-button"
                                onClick={() => toggleSectionCollapse('quickAccess')}
                                aria-label={sectionCollapsed.quickAccess ? 'Expand Quick Access' : 'Collapse Quick Access'}
                            >
                                <span className={`icon icon-chevron-${sectionCollapsed.quickAccess ? 'up' : 'down'}`}></span>
                            </button>
                        </div>
                        {!sectionCollapsed.quickAccess && (
                            <QuickAccess
                                onItemClick={handleItemClick}
                                currentView={currentView}
                                currentPath={currentPath}
                            />
                        )}
                    </section>
                    */}

                    {/* This PC section */}
                    <section className="sidebar-section">
                        <div className="sidebar-section-header">
                            <h3 className="sidebar-section-title">This PC</h3>
                            <button
                                className="section-collapse-button"
                                onClick={() => toggleSectionCollapse('thisPC')}
                                aria-label={sectionCollapsed.thisPC ? 'Expand This PC' : 'Collapse This PC'}
                            >
                                <span className={`icon icon-chevron-${sectionCollapsed.thisPC ? 'up' : 'down'}`}></span>
                            </button>
                        </div>
                        {!sectionCollapsed.thisPC && (
                            <ul className="sidebar-list">
                                <SidebarItem
                                    icon="computer"
                                    name="This PC"
                                    path="this-pc"
                                    isActive={currentView === 'this-pc'}
                                    onClick={() => {
                                        navigateTo(null); // Clear explorer path
                                        document.dispatchEvent(new CustomEvent('open-this-pc'));
                                    }}
                                />

                                {/* User volume (for macOS/Windows user directory) */}
                                {userVolume && (
                                    <SidebarItem
                                        icon="user"
                                        name={`User (${userVolume.volume_name || 'User Disk'})`}
                                        path={userVolume.mount_point}
                                        isActive={currentView === 'explorer' && currentPath === userVolume.mount_point}
                                        onClick={() => handleItemClick(userVolume.mount_point)}
                                        info={`${(userVolume.available_space / 1024 / 1024 / 1024).toFixed(1)}GB free`}
                                    />
                                )}

                                {/* User directories */}
                                {userDirectories.map((dir) => (
                                    <SidebarItem
                                        key={dir.path}
                                        icon={dir.icon}
                                        name={dir.name}
                                        path={dir.path}
                                        isActive={currentView === 'explorer' && currentPath === dir.path}
                                        onClick={() => handleItemClick(dir.path)}
                                    />
                                ))}
                            </ul>
                        )}
                    </section>

                    {/* Favorites section */}
                    <section className="sidebar-section">
                        <div className="sidebar-section-header">
                            <h3 className="sidebar-section-title">Favorites</h3>
                            <div className="sidebar-section-actions">
                                <button
                                    className="section-add-button"
                                    onClick={handleAddSource}
                                    title="Add to Favorites"
                                >
                                    <span className="icon icon-plus-small"></span>
                                </button>
                                <button
                                    className="section-collapse-button"
                                    onClick={() => toggleSectionCollapse('favorites')}
                                    aria-label={sectionCollapsed.favorites ? 'Expand Favorites' : 'Collapse Favorites'}
                                >
                                    <span className={`icon icon-chevron-${sectionCollapsed.favorites ? 'up' : 'down'}`}></span>
                                </button>
                            </div>
                        </div>
                        {!sectionCollapsed.favorites && (
                            <Favorites
                                onItemClick={handleItemClick}
                                onRemove={handleRemoveFromFavorites}
                                onAdd={addToFavorites}
                                currentView={currentView}
                                currentPath={currentPath}
                            />
                        )}
                    </section>

                    {/* Drives/Volumes section */}
                    <section className="sidebar-section">
                        <div className="sidebar-section-header">
                            <h3 className="sidebar-section-title">Drives</h3>
                            <button
                                className="section-collapse-button"
                                onClick={() => toggleSectionCollapse('drives')}
                                aria-label={sectionCollapsed.drives ? 'Expand Drives' : 'Collapse Drives'}
                            >
                                <span className={`icon icon-chevron-${sectionCollapsed.drives ? 'up' : 'down'}`}></span>
                            </button>
                        </div>
                        {!sectionCollapsed.drives && (
                            <ul className="sidebar-list">
                                {volumes.map((volume) => (
                                    <SidebarItem
                                        key={volume.mount_point}
                                        icon={volume.is_removable ? 'usb' : 'drive'}
                                        name={volume.volume_name || volume.mount_point}
                                        path={volume.mount_point}
                                        isActive={currentView === 'explorer' && currentPath === volume.mount_point}
                                        onClick={() => handleItemClick(volume.mount_point)}
                                        info={`${(volume.available_space / 1024 / 1024 / 1024).toFixed(1)}GB free of ${(volume.size / 1024 / 1024 / 1024).toFixed(1)}GB`}
                                        actions={volume.is_removable ? [
                                            {
                                                icon: 'eject',
                                                tooltip: 'Safely eject',
                                                onClick: async () => {
                                                    const confirmEject = await ask(`Are you sure you want to safely eject ${volume.volume_name}?`);
                                                    if (!confirmEject) return;

                                                    try {
                                                        const { invoke } = await import('@tauri-apps/api/core');
                                                        let command;
                                                        const os = systemInfo?.current_running_os?.toLowerCase();
                                                        
                                                        if (os === 'windows') {
                                                            command = `eject ${volume.mount_point}`;
                                                        } else if (os === 'macos' || os === 'darwin') {
                                                            // Use diskutil for proper ejection on macOS
                                                            command = `diskutil eject "${volume.mount_point}"`;
                                                        } else {
                                                            // Linux and other Unix-like systems
                                                            command = `umount "${volume.mount_point}"`;
                                                        }

                                                        const result = await invoke('execute_command', { command });
                                                        
                                                        // Parse the command result to check for success
                                                        const commandResponse = JSON.parse(result);
                                                        
                                                        if (commandResponse.status === 0) {
                                                            await message(`${volume.volume_name} has been safely ejected.`);
                                                            // Reload volumes to update the UI after ejection
                                                            setTimeout(() => {
                                                                loadVolumes();
                                                            }, 1000);
                                                        } else {
                                                            throw new Error(commandResponse.stderr || commandResponse.stdout || 'Ejection failed');
                                                        }
                                                    } catch (error) {
                                                        console.error('Failed to eject volume:', error);
                                                        let errorMessage = error.message || error;
                                                        
                                                        // Parse error message if it's JSON
                                                        try {
                                                            const parsedError = JSON.parse(errorMessage);
                                                            errorMessage = parsedError.custom_message || parsedError.error_message || errorMessage;
                                                        } catch (e) {
                                                            // If not JSON, use as-is
                                                        }
                                                        
                                                        await message(`Failed to eject ${volume.volume_name}: ${errorMessage}`);
                                                    }
                                                }
                                            }
                                        ] : []}
                                    />
                                ))}
                            </ul>
                        )}
                    </section>
                </div>

                {/* Bottom actions */}
                <div className="sidebar-footer">
                    <button
                        className={`sidebar-action-button ${isTerminalOpen ? 'active' : ''}`}
                        onClick={onTerminalToggle}
                        aria-label="Toggle Terminal"
                        title="Toggle Terminal (Ctrl+`)"
                    >
                        <span className="icon icon-terminal"></span>
                        <span>Terminal</span>
                    </button>

                    <button
                        className="sidebar-action-button"
                        onClick={() => {
                            navigateTo(null); // Clear explorer path
                            document.dispatchEvent(new CustomEvent('open-settings'));
                        }}
                        aria-label="Settings"
                        title="Settings"
                    >
                        <span className="icon icon-settings"></span>
                        <span>Settings</span>
                    </button>

                    <button
                        className="sidebar-action-button"
                        onClick={() => {
                            navigateTo(null); // Clear explorer path
                            document.dispatchEvent(new CustomEvent('open-templates'));
                        }}
                        aria-label="Templates"
                        title="Templates"
                    >
                        <span className="icon icon-template"></span>
                        <span>Templates</span>
                    </button>

                    <button
                        className="sidebar-action-button"
                        aria-label="Add Datasource"
                        title="Add Datasource"
                        onClick={handleAddSource}
                    >
                        <span className="icon icon-plus"></span>
                        <span>Add Source</span>
                    </button>
                </div>
            </aside>

            {/* Modal for adding data source */}
            <Modal
                isOpen={isAddSourceModalOpen}
                onClose={() => setIsAddSourceModalOpen(false)}
                title="Add Data Source"
                size="sm"
                footer={
                    <>
                        <Button
                            variant="ghost"
                            onClick={() => setIsAddSourceModalOpen(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            variant="primary"
                            onClick={saveNewSource}
                            disabled={!newSourcePath.trim()}
                        >
                            Add Source
                        </Button>
                    </>
                }
            >
                <form onSubmit={handleAddSourceSubmit}>
                    <div className="form-group">
                        <label htmlFor="source-path">Data Source Path</label>
                        <input
                            ref={addSourceInputRef}
                            type="text"
                            id="source-path"
                            className="input"
                            value={newSourcePath}
                            onChange={handleAddSourceInputChange}
                            onKeyDown={handleAddSourceKeyDown}
                            placeholder="Enter path to folder to add as data source"
                        />
                        <div className="input-hint">
                            Enter the full path to a folder that you want to add as a data source to favorites.
                        </div>
                    </div>
                </form>
            </Modal>
        </>
    );
};

export default Sidebar;

