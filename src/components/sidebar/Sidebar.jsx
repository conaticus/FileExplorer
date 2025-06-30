import React, { useState, useEffect, useRef } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { useContextMenu } from '../../providers/ContextMenuProvider';
import SidebarItem from './SidebarItem';
import Favorites from './Favorites';
import QuickAccess from './QuickAccess';
import Modal from '../common/Modal';
import Button from '../common/Button';
import './sidebar.css';

const Sidebar = ({ onTerminalToggle, isTerminalOpen, currentView }) => {
    const { volumes, loadDirectory } = useFileSystem();
    const { currentPath } = useHistory();
    const { removeFromFavorites } = useContextMenu();
    const [isCollapsed, setIsCollapsed] = useState(() => {
        return localStorage.getItem('sidebarCollapsed') === 'true';
    });
    const [systemInfo, setSystemInfo] = useState(null);
    const [isAddSourceModalOpen, setIsAddSourceModalOpen] = useState(false);
    const [newSourcePath, setNewSourcePath] = useState('');
    const addSourceInputRef = useRef(null);

    // Load system info to get proper user directories
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

    // Toggle sidebar collapse
    const toggleCollapse = () => {
        const newCollapsed = !isCollapsed;
        setIsCollapsed(newCollapsed);
        localStorage.setItem('sidebarCollapsed', newCollapsed.toString());
    };

    // Handle clicking on a sidebar item with navigation history update
    const handleItemClick = (path) => {
        // Update navigation history immediately
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

        loadDirectory(path);
    };

    // Add location to favorites
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

    // Remove location from favorites (using the one from context menu provider)
    const handleRemoveFromFavorites = (path) => {
        removeFromFavorites(path);
    };

    // Handle add source modal
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

    // Save new source
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

    // Handle form submission for add source
    const handleAddSourceSubmit = (e) => {
        e.preventDefault();
        saveNewSource();
    };

    // Handle input change for add source
    const handleAddSourceInputChange = (e) => {
        setNewSourcePath(e.target.value);
    };

    // Handle key down for add source input
    const handleAddSourceKeyDown = (e) => {
        if (e.key === 'Escape') {
            setIsAddSourceModalOpen(false);
        }
    };

    // Get user directories based on OS
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

    // Get user volume (for macOS dual mount handling)
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
            <aside className={`sidebar ${isCollapsed ? 'sidebar-collapsed' : ''}`}>
                <div className="sidebar-header">
                    <h2 className={isCollapsed ? 'visually-hidden' : ''}>File Explorer</h2>
                    <button
                        className="collapse-button"
                        onClick={toggleCollapse}
                        aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
                    >
                        <span className={`icon icon-${isCollapsed ? 'chevron-right' : 'chevron-left'}`}></span>
                    </button>
                </div>

                <div className="sidebar-content">
                    {/* Quick Access section */}
                    <section className="sidebar-section">
                        {!isCollapsed && <h3 className="sidebar-section-title">Quick Access</h3>}
                        <QuickAccess
                            isCollapsed={isCollapsed}
                            onItemClick={handleItemClick}
                        />
                    </section>

                    {/* This PC section */}
                    <section className="sidebar-section">
                        {!isCollapsed && <h3 className="sidebar-section-title">This PC</h3>}
                        <ul className="sidebar-list">
                            <SidebarItem
                                icon="computer"
                                name="This PC"
                                path="this-pc"
                                isCollapsed={isCollapsed}
                                isActive={currentView === 'this-pc'}
                                onClick={() => document.dispatchEvent(new CustomEvent('open-this-pc'))}
                            />

                            {/* User volume (for macOS/Windows user directory) */}
                            {userVolume && (
                                <SidebarItem
                                    icon="user"
                                    name={`User (${userVolume.volume_name || 'User Disk'})`}
                                    path={userVolume.mount_point}
                                    isCollapsed={isCollapsed}
                                    isActive={currentPath === userVolume.mount_point}
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
                                    isCollapsed={isCollapsed}
                                    isActive={currentPath === dir.path}
                                    onClick={() => handleItemClick(dir.path)}
                                />
                            ))}
                        </ul>
                    </section>

                    {/* Favorites section */}
                    <section className="sidebar-section">
                        {!isCollapsed && (
                            <div className="sidebar-section-header">
                                <h3 className="sidebar-section-title">Favorites</h3>
                                <button
                                    className="section-add-button"
                                    onClick={handleAddSource}
                                    title="Add to Favorites"
                                >
                                    <span className="icon icon-plus-small"></span>
                                </button>
                            </div>
                        )}
                        <Favorites
                            isCollapsed={isCollapsed}
                            onItemClick={handleItemClick}
                            onRemove={handleRemoveFromFavorites}
                            onAdd={addToFavorites}
                        />
                    </section>

                    {/* Drives/Volumes section */}
                    <section className="sidebar-section">
                        {!isCollapsed && <h3 className="sidebar-section-title">Drives</h3>}
                        <ul className="sidebar-list">
                            {volumes.map((volume) => (
                                <SidebarItem
                                    key={volume.mount_point}
                                    icon={volume.is_removable ? 'usb' : 'drive'}
                                    name={volume.volume_name || volume.mount_point}
                                    path={volume.mount_point}
                                    isCollapsed={isCollapsed}
                                    isActive={currentPath === volume.mount_point}
                                    onClick={() => handleItemClick(volume.mount_point)}
                                    info={`${(volume.available_space / 1024 / 1024 / 1024).toFixed(1)}GB free of ${(volume.size / 1024 / 1024 / 1024).toFixed(1)}GB`}
                                    actions={volume.is_removable ? [
                                        {
                                            icon: 'eject',
                                            tooltip: 'Safely eject',
                                            onClick: async () => {
                                                const confirmEject = confirm(`Are you sure you want to safely eject ${volume.volume_name}?`);
                                                if (!confirmEject) return;

                                                try {
                                                    const { invoke } = await import('@tauri-apps/api/core');
                                                    const command = systemInfo?.current_running_os === 'windows'
                                                        ? `eject ${volume.mount_point}`
                                                        : `umount ${volume.mount_point}`;

                                                    await invoke('execute_command', { command });
                                                    alert(`${volume.volume_name} has been safely ejected.`);
                                                } catch (error) {
                                                    console.error('Failed to eject volume:', error);
                                                    alert(`Failed to eject ${volume.volume_name}: ${error.message || error}`);
                                                }
                                            }
                                        }
                                    ] : []}
                                />
                            ))}
                        </ul>
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
                        {!isCollapsed && <span>Terminal</span>}
                    </button>

                    <button
                        className="sidebar-action-button"
                        onClick={() => document.dispatchEvent(new CustomEvent('open-settings'))}
                        aria-label="Settings"
                        title="Settings"
                    >
                        <span className="icon icon-settings"></span>
                        {!isCollapsed && <span>Settings</span>}
                    </button>

                    <button
                        className="sidebar-action-button"
                        onClick={() => document.dispatchEvent(new CustomEvent('open-templates'))}
                        aria-label="Templates"
                        title="Templates"
                    >
                        <span className="icon icon-template"></span>
                        {!isCollapsed && <span>Templates</span>}
                    </button>

                    <button
                        className="sidebar-action-button"
                        aria-label="Add Datasource"
                        title="Add Datasource"
                        onClick={handleAddSource}
                    >
                        <span className="icon icon-plus"></span>
                        {!isCollapsed && <span>Add Source</span>}
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