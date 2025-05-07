import React, { useState, useEffect } from 'react';
import SidebarItem from './SidebarItem';

const QuickAccess = ({ isCollapsed = false, onItemClick }) => {
    const [recentItems, setRecentItems] = useState([]);

    // Load recent locations from session storage
    useEffect(() => {
        try {
            const savedHistory = JSON.parse(sessionStorage.getItem('fileExplorerHistory') || '[]');

            // Get unique locations (remove duplicates)
            const uniqueLocations = savedHistory.filter((path, index, self) =>
                self.indexOf(path) === index
            );

            // Take the most recent 5 locations
            const recentLocations = uniqueLocations.slice(-5).reverse().map(path => {
                // Extract the folder name from the path
                const parts = path.split('/');
                const name = parts[parts.length - 1] || path;

                return {
                    name,
                    path,
                    icon: 'folder',
                };
            });

            setRecentItems(recentLocations);
        } catch (err) {
            console.error('Failed to load recent locations:', err);
        }
    }, []);

    // If there are no recent items, display a message
    if (recentItems.length === 0) {
        if (isCollapsed) return null;

        return (
            <div className="sidebar-empty-state">
                <p>No recent locations</p>
            </div>
        );
    }

    return (
        <ul className="sidebar-list">
            {recentItems.map((item) => (
                <SidebarItem
                    key={item.path}
                    icon={item.icon}
                    name={item.name}
                    path={item.path}
                    isCollapsed={isCollapsed}
                    onClick={() => onItemClick(item.path)}
                />
            ))}
        </ul>
    );
};

export default QuickAccess;