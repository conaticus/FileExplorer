import React, { useState, useEffect, useCallback } from 'react';
import SidebarItem from './SidebarItem';

/**
 * QuickAccess component - Displays recent locations for quick navigation
 *
 * @param {Object} props - Component props
 * @param {boolean} [props.isCollapsed=false] - Whether the sidebar is collapsed
 * @param {Function} props.onItemClick - Callback for item click
 * @returns {React.ReactElement} QuickAccess component
 */
const QuickAccess = ({ isCollapsed = false, onItemClick }) => {
    const [recentItems, setRecentItems] = useState([]);

    /**
     * Load recent locations from session storage
     * @returns {void}
     */
    const loadRecentItems = useCallback(() => {
        try {
            const savedHistory = JSON.parse(sessionStorage.getItem('fileExplorerHistory') || '[]');

            // Get unique locations (remove duplicates)
            const uniqueLocations = savedHistory.filter((path, index, self) =>
                self.indexOf(path) === index
            );

            // Take the most recent 5 locations
            const recentLocations = uniqueLocations.slice(-5).reverse().map(path => {
                // Extract the folder name from the path
                const parts = path.split(/[/\\]/);
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
            setRecentItems([]);
        }
    }, []);

    /**
     * Load recent items on mount
     */
    useEffect(() => {
        loadRecentItems();
    }, [loadRecentItems]);

    /**
     * Listen for navigation changes to update quick access
     */
    useEffect(() => {
        const handleNavigationChange = () => {
            loadRecentItems();
        };

        const handleQuickAccessUpdate = () => {
            loadRecentItems();
        };

        // Listen for custom events
        window.addEventListener('navigation-changed', handleNavigationChange);
        window.addEventListener('quick-access-updated', handleQuickAccessUpdate);

        // Also listen for storage events in case history is updated from elsewhere
        const handleStorageChange = (e) => {
            if (e.key === 'fileExplorerHistory') {
                loadRecentItems();
            }
        };

        window.addEventListener('storage', handleStorageChange);

        return () => {
            window.removeEventListener('navigation-changed', handleNavigationChange);
            window.removeEventListener('quick-access-updated', handleQuickAccessUpdate);
            window.removeEventListener('storage', handleStorageChange);
        };
    }, [loadRecentItems]);

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