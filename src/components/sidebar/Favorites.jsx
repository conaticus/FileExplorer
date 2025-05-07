import React from 'react';
import SidebarItem from './SidebarItem';

const Favorites = ({
                       favorites = [],
                       isCollapsed = false,
                       onItemClick,
                       onRemove
                   }) => {
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