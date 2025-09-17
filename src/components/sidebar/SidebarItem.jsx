import React from 'react';
import { useHistory } from '../../providers/HistoryProvider';

/**
 * SidebarItem component - Renders a single item in the sidebar
 *
 * @param {Object} props - Component props
 * @param {string} props.icon - Icon name to display
 * @param {string} props.name - Display name for the item
 * @param {string} props.path - Path for navigation
 * @param {string} [props.info] - Additional information text
 * @param {boolean} [props.isCollapsed=false] - Whether the sidebar is collapsed
 * @param {boolean} [props.isActive=false] - Whether this item is active
 * @param {Function} props.onClick - Click handler for the item
 * @param {Function} [props.onContextMenu] - Context menu handler
 * @param {Array<{icon: string, tooltip: string, onClick: Function}>} [props.actions=[]] - Action buttons to display
 * @returns {React.ReactElement} SidebarItem component
 */
const SidebarItem = ({
                         icon,
                         name,
                         path,
                         info,
                         isCollapsed = false,
                         isActive = false,
                         onClick,
                         onContextMenu,
                         actions = []
                     }) => {
    /**
     * Handle context menu
     * @param {React.MouseEvent} e - Context menu event
     */
    const handleContextMenu = (e) => {
        e.preventDefault();
        if (onContextMenu) {
            onContextMenu(e, { name, path, icon, info });
        }
    };

    /**
     * Handles double-click events to prevent text selection
     * @param {React.MouseEvent} e - The double-click event
     */
    const handleDoubleClick = (e) => {
        // Prevent text selection on double-click
        e.preventDefault();
        e.stopPropagation();
        
        // Clear any existing text selection
        if (window.getSelection) {
            window.getSelection().removeAllRanges();
        }
    };

    /**
     * Handles mouse down events to prevent text selection
     * @param {React.MouseEvent} e - The mouse down event
     */
    const handleMouseDown = (e) => {
        // Prevent text selection on mouse down
        e.preventDefault();
    };

    /**
     * Handles selectstart events to prevent text selection
     * @param {React.SyntheticEvent} e - The selectstart event
     */
    const handleSelectStart = (e) => {
        // Prevent any text selection
        e.preventDefault();
        return false;
    };

    return (
        <li
            className={`sidebar-item ${isActive ? 'active' : ''}`}
            onClick={onClick}
            onDoubleClick={handleDoubleClick}
            onMouseDown={handleMouseDown}
            onSelectStart={handleSelectStart}
            onContextMenu={handleContextMenu}
            title={isCollapsed ? name : undefined}
        >
            <span className={`sidebar-item-icon icon icon-${icon}`}></span>

            <div className="sidebar-item-content">
                <span className="sidebar-item-name">{name}</span>
                {info && <span className="sidebar-item-info">{info}</span>}
            </div>

            {actions.length > 0 && !isCollapsed && (
                <div className="sidebar-item-actions">
                    {actions.map((action, index) => (
                        <button
                            key={`action-${index}`}
                            className="sidebar-item-action"
                            onClick={(e) => {
                                e.stopPropagation();
                                action.onClick();
                            }}
                            title={action.tooltip}
                            aria-label={action.tooltip}
                        >
                            <span className={`icon icon-${action.icon}`}></span>
                        </button>
                    ))}
                </div>
            )}
        </li>
    );
};

export default SidebarItem;