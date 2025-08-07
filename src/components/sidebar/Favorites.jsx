import React, { useEffect, useState, useCallback } from 'react';
import SidebarItem from './SidebarItem';

/**
 * Favorites component - Displays and manages favorite locations
 *
 * @param {Object} props - Component props
 * @param {boolean} [props.isCollapsed=false] - Whether the sidebar is collapsed
 * @param {Function} props.onItemClick - Callback when an item is clicked
 * @param {Function} props.onRemove - Callback to remove an item from favorites
 * @param {Function} props.onAdd - Callback to add an item to favorites
 * @returns {React.ReactElement} Favorites component
 */
const Favorites = ({
                       isCollapsed = false,
                       onItemClick,
                       onRemove,
                       onAdd
                   }) => {
    const [favorites, setFavorites] = useState([]);

    /**
     * Load favorites from localStorage
     * @returns {void}
     */
    const loadFavorites = useCallback(() => {
        try {
            const savedFavorites = JSON.parse(localStorage.getItem('fileExplorerFavorites') || '[]');
            setFavorites(savedFavorites);
        } catch (err) {
            console.error('Failed to load favorites:', err);
            setFavorites([]);
        }
    }, []);

    /**
     * Load favorites on mount
     */
    useEffect(() => {
        loadFavorites();
    }, [loadFavorites]);

    /**
     * Listen for storage events (from other tabs) and custom events (from current tab)
     */
    useEffect(() => {
        const handleStorageChange = (e) => {
            if (e.key === 'fileExplorerFavorites') {
                loadFavorites();
            }
        };

        const handleFavoritesUpdate = () => {
            loadFavorites();
        };

        // Listen for storage events from other tabs
        window.addEventListener('storage', handleStorageChange);

        // Listen for custom events from the current tab
        window.addEventListener('favorites-updated', handleFavoritesUpdate);

        return () => {
            window.removeEventListener('storage', handleStorageChange);
            window.removeEventListener('favorites-updated', handleFavoritesUpdate);
        };
    }, [loadFavorites]);

    /**
     * Handle context menu for favorites
     * @param {React.MouseEvent} e - Context menu event
     * @param {Object} item - The favorite item
     */
    const handleContextMenu = (e, item) => {
        e.preventDefault();

        const choice = confirm('Remove from favorites?');
        if (choice) {
            onRemove(item.path);
        }
    };

    // If there are no favorites, display a message
    if (favorites.length === 0) {
        if (isCollapsed) return null;

        return (
            <div className="sidebar-empty-state">
                <small className="empty-state-title">No favorites added <br /></small>
                <small className="empty-state-subtitle">Right-click a folder and select<br />"Add to Favorites"</small>
            </div>
        );
    }

    return (
        <ul className="sidebar-list">
            {favorites.map((item) => (
                <SidebarItem
                    key={item.path}
                    icon={item.icon || 'star'}
                    name={item.name}
                    path={item.path}
                    isCollapsed={isCollapsed}
                    onClick={() => onItemClick(item.path)}
                    onContextMenu={(e) => handleContextMenu(e, item)}
                    actions={[
                        {
                            icon: 'x',
                            tooltip: 'Remove from favorites',
                            onClick: () => onRemove(item.path),
                        },
                    ]}
                />
            ))}
        </ul>
    );
};

export default Favorites;