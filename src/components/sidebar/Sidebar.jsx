import React, {useEffect, useRef, useState} from 'react';
import {useFileSystem} from '../../providers/FileSystemProvider';
import {useHistory} from '../../providers/HistoryProvider';
import {useContextMenu} from '../../providers/ContextMenuProvider';
import {useSftp} from '../../providers/SftpProvider';
import SidebarItem from './SidebarItem';
import Favorites from './Favorites';
//import QuickAccess from './QuickAccess';
import Modal from '../common/Modal';
import Button from '../common/Button';
import AddSftpConnectionView from './AddSftpConnectionView';
import PermissionHelper from '../common/PermissionHelper';
import {ask, message, open} from '@tauri-apps/plugin-dialog';
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
    const { navigateToSftpConnection, createSftpUrl, isSftpPath, parseSftpPath, createSftpPath } = useSftp();

    const [systemInfo, setSystemInfo] = useState(null);
    const [isAddSourceModalOpen, setIsAddSourceModalOpen] = useState(false);
    const [newSourcePath, setNewSourcePath] = useState('');
    const addSourceInputRef = useRef(null);

    // SFTP Connections state
    const [sftpConnections, setSftpConnections] = useState([]);
    const [isAddSftpModalOpen, setIsAddSftpModalOpen] = useState(false);
    
    // Permission helper state
    const [isPermissionHelperOpen, setIsPermissionHelperOpen] = useState(false);
    const [permissionDirectory, setPermissionDirectory] = useState(null);

    // Quick browse to protected folder
    const browseToProtectedFolder = async (folderName, expectedPath) => {
        try {
            const selectedPath = await open({
                directory: true,
                title: `Browse to your ${folderName} folder`,
                defaultPath: expectedPath ? expectedPath.substring(0, expectedPath.lastIndexOf('/')) : undefined
            });
            
            if (selectedPath) {
                await handleItemClick(selectedPath, folderName);
            }
        } catch (error) {
            console.error(`Failed to browse to ${folderName}:`, error);
        }
    };
    // Load SFTP connections from localStorage
    const loadSftpConnections = React.useCallback(() => {
        try {
            const saved = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
            setSftpConnections(saved);
        } catch (err) {
            setSftpConnections([]);
        }
    }, []);

    // Load on mount and on custom event
    React.useEffect(() => {
        loadSftpConnections();
        const handler = () => loadSftpConnections();
        window.addEventListener('sftp-connections-updated', handler);
        window.addEventListener('storage', (e) => {
            if (e.key === 'fileExplorerSftpConnections') loadSftpConnections();
        });
        return () => {
            window.removeEventListener('sftp-connections-updated', handler);
        };
    }, [loadSftpConnections]);

    // Add SFTP connection
    const addSftpConnection = (conn) => {
        try {
            const existing = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
            const newConnections = [...existing, conn];
            localStorage.setItem('fileExplorerSftpConnections', JSON.stringify(newConnections));
            window.dispatchEvent(new CustomEvent('sftp-connections-updated'));
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerSftpConnections',
                newValue: JSON.stringify(newConnections)
            }));
        } catch (err) {
            // ignore
        }
        setIsAddSftpModalOpen(false);
    };

    // Remove SFTP connection with confirmation
    const removeSftpConnection = async (name) => {
        const confirmRemove = await ask(`Are you sure you want to remove the SFTP connection "${name}"?`);
        if (!confirmRemove) return;
        try {
            const existing = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
            const newConnections = existing.filter(c => c.name !== name);
            localStorage.setItem('fileExplorerSftpConnections', JSON.stringify(newConnections));
            window.dispatchEvent(new CustomEvent('sftp-connections-updated'));
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerSftpConnections',
                newValue: JSON.stringify(newConnections)
            }));
        } catch (err) {
            // ignore
        }
    };

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
     * Handles clicking on a sidebar item with navigation history update and permission handling
     * @param {string} path - Path to navigate to
     * @param {string} [name] - Display name of the directory (for permission helper)
     */
    const handleItemClick = async (path, name = null) => {
        let targetPath = path;
        
        // Handle SFTP files - navigate to parent directory instead of trying to open as directory
        if (isSftpPath(path)) {
            // Check if this is a favorite item and if it looks like a file
            // For SFTP favorites, check if the path ends with a file extension or doesn't look like a directory
            const favorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');
            const favoriteItem = favorites.find(fav => fav.path === path);
            
            if (favoriteItem && favoriteItem.icon === 'file') {
                // This is an SFTP file favorite, navigate to its parent directory
                const parsed = parseSftpPath(path);
                if (parsed && parsed.connection) {
                    const pathParts = parsed.remotePath.split('/').filter(part => part && part !== '.');
                    pathParts.pop(); // Remove the file name
                    const parentPath = pathParts.length > 0 ? pathParts.join('/') : '.';
                    targetPath = createSftpPath(parsed.connection, parentPath);
                    console.log('SFTP file favorite detected, navigating to parent:', targetPath);
                }
            }
        }
        
        // Always update navigation history and reload directory, even if path is the same
        try {
            const existingHistory = JSON.parse(sessionStorage.getItem('fileExplorerHistory') || '[]');
            const updatedHistory = [targetPath, ...existingHistory.filter(p => p !== targetPath)].slice(0, 10);
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
        
        const success = await loadDirectory(targetPath);
        
        // If loading failed and it's a user directory, offer permission helper
        if (!success && name) {
            const isUserDir = ['Desktop', 'Documents', 'Downloads', 'Pictures', 'Movies', 'Music'].some(dir => 
                targetPath.toLowerCase().includes(dir.toLowerCase())
            );
            
            if (isUserDir) {
                setPermissionDirectory({ path: targetPath, name });
                setIsPermissionHelperOpen(true);
            }
        }
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
        return volumes.find(vol =>
            homeDir.startsWith(vol.mount_point) && vol.mount_point !== '/'
        );
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
                                {userDirectories.map((dir) => {
                                    const isProtectedDir = ['Desktop', 'Documents', 'Downloads'].includes(dir.name);
                                    return (
                                        <SidebarItem
                                            key={dir.path}
                                            icon={dir.icon}
                                            name={dir.name}
                                            path={dir.path}
                                            isActive={currentView === 'explorer' && currentPath === dir.path}
                                            onClick={() => handleItemClick(dir.path, dir.name)}
                                            actions={isProtectedDir ? [{
                                                icon: 'folder-open',
                                                tooltip: `Browse to ${dir.name} folder`,
                                                onClick: () => browseToProtectedFolder(dir.name, dir.path)
                                            }] : []}
                                        />
                                    );
                                })}
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
                    {/* Network section */}
                    <section>
                        <div className="sidebar-section-header">
                            <h3 className="sidebar-section-title">Network</h3>
                            <div className="sidebar-section-actions">
                                <button
                                    className="section-add-button"
                                    onClick={() => setIsAddSftpModalOpen(true)}
                                    title="Add SFTP Connection"
                                >
                                    <span className="icon icon-plus-small"></span>
                                </button>
                                <button
                                    className="section-collapse-button"
                                    onClick={() => toggleSectionCollapse('network')}
                                    aria-label={sectionCollapsed.network ? 'Expand Network' : 'Collapse Network'}
                                >
                                    <span className={`icon icon-chevron-${sectionCollapsed.network ? 'up' : 'down'}`}></span>
                                </button>
                            </div>
                        </div>
                        {!sectionCollapsed.network && (
                            <ul className="sidebar-list">
                                {sftpConnections.length === 0 ? (
                                    <div className="sidebar-empty-state">
                                        <div className="empty-state-icon">
                                            <span className="icon icon-network"></span>
                                        </div>
                                        <div className="empty-state-text">
                                            <p>No SFTP connections</p>
                                            <span>Add a connection to get started</span>
                                        </div>
                                    </div>
                                ) : (
                                    sftpConnections.map((conn) => (
                                        <SidebarItem
                                            key={conn.name}
                                            icon="network"
                                            name={conn.name}
                                            path={`sftp://${conn.username}@${conn.host}:${conn.port}`}
                                            isActive={currentView === 'network' && currentPath === conn.name}
                                            onClick={async () => {
                                                // Navigate to SFTP connection using the new provider
                                                try {
                                                    const sftpData = await navigateToSftpConnection(conn);
                                                    if (sftpData) {
                                                        const sftpPath = createSftpUrl(conn, '.');
                                                        await loadDirectory(sftpPath);
                                                        navigateTo(sftpPath);
                                                    }
                                                } catch (error) {
                                                    console.error('Failed to connect to SFTP:', error);
                                                }
                                            }}
                                            actions={[{
                                                icon: 'x',
                                                tooltip: 'Remove SFTP Connection',
                                                onClick: () => removeSftpConnection(conn.name)
                                            }]}
                                        />
                                    ))
                                )}
                            </ul>
                        )}
                    </section>
            {/* Modal for adding SFTP connection */}
            <AddSftpConnectionView
                isOpen={isAddSftpModalOpen}
                onClose={() => setIsAddSftpModalOpen(false)}
                onAdd={addSftpConnection}
            />

            {/* Permission Helper Modal */}
            <PermissionHelper
                isOpen={isPermissionHelperOpen}
                onClose={() => {
                    setIsPermissionHelperOpen(false);
                    setPermissionDirectory(null);
                }}
                directoryPath={permissionDirectory?.path}
                directoryName={permissionDirectory?.name}
                onDirectorySelected={(selectedPath) => {
                    // Navigate to the selected directory
                    handleItemClick(selectedPath, permissionDirectory?.name);
                }}
            />
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

