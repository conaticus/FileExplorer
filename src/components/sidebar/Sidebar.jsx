import React, { useState, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import SidebarItem from './SidebarItem';
import Favorites from './Favorites';
import QuickAccess from './QuickAccess';
import './sidebar.css';

const Sidebar = () => {
    const { volumes, loadDirectory } = useFileSystem();
    const [isCollapsed, setIsCollapsed] = useState(false);
    const [favorites, setFavorites] = useState([]);

    // Load favorites from localStorage
    useEffect(() => {
        try {
            const savedFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');
            setFavorites(savedFavorites);
        } catch (err) {
            console.error('Failed to load favorites:', err);
        }
    }, []);

    // Toggle sidebar collapse
    const toggleCollapse = () => {
        setIsCollapsed(prev => !prev);
    };

    // Add location to favorites
    const addToFavorites = (location) => {
        const newFavorites = [...favorites, location];
        setFavorites(newFavorites);
        localStorage.setItem('fileExplorerFavorites', JSON.stringify(newFavorites));
    };

    // Remove location from favorites
    const removeFromFavorites = (path) => {
        const newFavorites = favorites.filter(fav => fav.path !== path);
        setFavorites(newFavorites);
        localStorage.setItem('fileExplorerFavorites', JSON.stringify(newFavorites));
    };

    // Handle clicking on a sidebar item
    const handleItemClick = (path) => {
        loadDirectory(path);
    };

    return (
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

                {/* Favorites section */}
                <section className="sidebar-section">
                    {!isCollapsed && <h3 className="sidebar-section-title">Favorites</h3>}
                    <Favorites
                        favorites={favorites}
                        isCollapsed={isCollapsed}
                        onItemClick={handleItemClick}
                        onRemove={removeFromFavorites}
                    />
                </section>

                {/* Drives/Volumes section */}
                <section className="sidebar-section">
                    {!isCollapsed && <h3 className="sidebar-section-title">Drives</h3>}
                    <ul className="sidebar-list">
                        {volumes.map((volume) => (
                            <SidebarItem
                                key={volume.mount_point}
                                icon="drive"
                                name={volume.volume_name || volume.mount_point}
                                path={volume.mount_point}
                                isCollapsed={isCollapsed}
                                onClick={() => handleItemClick(volume.mount_point)}
                                info={`${(volume.available_space / 1024 / 1024 / 1024).toFixed(1)}GB free of ${(volume.size / 1024 / 1024 / 1024).toFixed(1)}GB`}
                            />
                        ))}
                    </ul>
                </section>

                {/* This PC / Computer section */}
                <section className="sidebar-section">
                    {!isCollapsed && <h3 className="sidebar-section-title">This PC</h3>}
                    <ul className="sidebar-list">
                        <SidebarItem
                            icon="desktop"
                            name="Desktop"
                            path="/home/user/Desktop" // This will need to be dynamically determined
                            isCollapsed={isCollapsed}
                            onClick={() => handleItemClick('/home/user/Desktop')}
                        />
                        <SidebarItem
                            icon="documents"
                            name="Documents"
                            path="/home/user/Documents" // This will need to be dynamically determined
                            isCollapsed={isCollapsed}
                            onClick={() => handleItemClick('/home/user/Documents')}
                        />
                        <SidebarItem
                            icon="downloads"
                            name="Downloads"
                            path="/home/user/Downloads" // This will need to be dynamically determined
                            isCollapsed={isCollapsed}
                            onClick={() => handleItemClick('/home/user/Downloads')}
                        />
                        <SidebarItem
                            icon="pictures"
                            name="Pictures"
                            path="/home/user/Pictures" // This will need to be dynamically determined
                            isCollapsed={isCollapsed}
                            onClick={() => handleItemClick('/home/user/Pictures')}
                        />
                        <SidebarItem
                            icon="music"
                            name="Music"
                            path="/home/user/Music" // This will need to be dynamically determined
                            isCollapsed={isCollapsed}
                            onClick={() => handleItemClick('/home/user/Music')}
                        />
                        <SidebarItem
                            icon="videos"
                            name="Videos"
                            path="/home/user/Videos" // This will need to be dynamically determined
                            isCollapsed={isCollapsed}
                            onClick={() => handleItemClick('/home/user/Videos')}
                        />
                    </ul>
                </section>
            </div>

            {/* Bottom actions */}
            <div className="sidebar-footer">
                <button
                    className="sidebar-action-button"
                    aria-label="Settings"
                    title="Settings"
                >
                    <span className="icon icon-settings"></span>
                    {!isCollapsed && <span>Settings</span>}
                </button>

                <button
                    className="sidebar-action-button"
                    aria-label="Add Datasource"
                    title="Add Datasource"
                >
                    <span className="icon icon-add-source"></span>
                    {!isCollapsed && <span>Add Datasource</span>}
                </button>
            </div>
        </aside>
    );
};

export default Sidebar;