import React, { useEffect } from 'react';
import SidebarItem from './SidebarItem';

const Favorites = ({
                       favorites = [],
                       isCollapsed = false,
                       onItemClick,
                       onRemove,
                       onAdd
                   }) => {

    // Listen for storage events to update favorites
    useEffect(() => {
        const handleStorageChange = (e) => {
            if (e.key === 'fileExplorerFavorites') {
                // Force a re-render by dispatching a custom event
                window.dispatchEvent(new CustomEvent('favorites-updated'));
            }
        };

        window.addEventListener('storage', handleStorageChange);

        return () => {
            window.removeEventListener('storage', handleStorageChange);
        };
    }, []);

    // Handle context menu for favorites
    const handleContextMenu = (e, item) => {
        e.preventDefault();

        const menu = [
            {
                label: 'Open',
                icon: 'open',
                action: () => onItemClick(item.path)
            },
            {
                label: 'Remove from Favorites',
                icon: 'x',
                action: () => onRemove(item.path)
            }
        ];

        // Simple context menu implementation
        // In a full implementation, you'd use the ContextMenuProvider
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
                <p>No favorites added</p>
                <small>Right-click a folder and select "Add to Favorites"</small>
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