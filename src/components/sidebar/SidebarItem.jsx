import React from 'react';
import { useHistory } from '../../providers/HistoryProvider';

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
    // Handle context menu
    const handleContextMenu = (e) => {
        e.preventDefault();
        if (onContextMenu) {
            onContextMenu(e, { name, path, icon, info });
        }
    };

    return (
        <li
            className={`sidebar-item ${isActive ? 'active' : ''}`}
            onClick={onClick}
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