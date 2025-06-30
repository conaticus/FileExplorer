import React, { useRef, useState, useEffect } from 'react';

/**
 * Single context menu item that can render both main menu items and submenu items
 * @param {Object} props - Component properties
 * @param {Object} props.item - The menu item to render
 * @param {string} props.item.label - Label of the menu item
 * @param {string} [props.item.icon] - Icon name for the menu item
 * @param {string} [props.item.shortcut] - Keyboard shortcut for the menu item
 * @param {Array} [props.item.submenu] - Array of submenu items
 * @param {Function} [props.item.action] - Action to execute on click
 * @param {boolean} [props.item.disabled] - Whether the item is disabled
 * @param {boolean} [props.isSubmenuOpen=false] - Whether the submenu is open
 * @param {Function} props.onSubmenuOpen - Callback when submenu opens
 * @param {Function} props.onSubmenuClose - Callback when submenu closes
 * @param {Function} props.onAction - Action to execute on click
 * @returns {React.ReactElement} Context menu item component
 */
const ContextMenuItem = ({
                             item,
                             isSubmenuOpen = false,
                             onSubmenuOpen,
                             onSubmenuClose,
                             onAction,
                         }) => {
    const [submenuPosition, setSubmenuPosition] = useState({ x: 0, y: 0 });
    const itemRef = useRef(null);

    /**
     * Calculates the submenu position based on the parent element position
     */
    useEffect(() => {
        if (isSubmenuOpen && itemRef.current && item.submenu) {
            const itemRect = itemRef.current.getBoundingClientRect();

            setSubmenuPosition({
                x: itemRect.right,
                y: itemRect.top,
            });
        }
    }, [isSubmenuOpen, item.submenu]);

    /**
     * Handles mouse enter - opens submenu
     */
    const handleMouseEnter = () => {
        if (item.submenu) {
            onSubmenuOpen();
        }
    };

    /**
     * Handles mouse leave - closes submenu with a short delay
     */
    const handleMouseLeave = () => {
        if (item.submenu) {
            // Add delay to prevent submenu from closing immediately
            setTimeout(() => {
                onSubmenuClose();
            }, 100);
        }
    };

    /**
     * Handles click events on menu items
     * @param {React.MouseEvent} e - The click event
     */
    const handleClick = (e) => {
        e.stopPropagation();

        console.log('ContextMenuItem clicked:', item.label);

        if (item.disabled) {
            console.log('Item is disabled, ignoring click');
            return;
        }

        if (item.submenu) {
            console.log('Opening submenu for:', item.label);
            onSubmenuOpen();
        } else if (onAction) {
            console.log('Executing action for:', item.label);
            onAction();
        } else {
            console.log('No action defined for:', item.label);
        }
    };

    return (
        <li
            ref={itemRef}
            className={`context-menu-item ${item.disabled ? 'disabled' : ''}`}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
            onClick={handleClick}
        >
            {/* Icon */}
            {item.icon && (
                <span className={`context-menu-icon icon-${item.icon}`}></span>
            )}

            {/* Label */}
            <span className="context-menu-label">{item.label}</span>

            {/* Shortcut */}
            {item.shortcut && (
                <span className="context-menu-shortcut">{item.shortcut}</span>
            )}

            {/* Submenu indicator */}
            {item.submenu && (
                <span className="context-menu-submenu-indicator">
          <span className="icon icon-chevron-right"></span>
        </span>
            )}

            {/* Submenu */}
            {isSubmenuOpen && item.submenu && (
                <div
                    className="context-submenu"
                    style={{
                        left: `${submenuPosition.x}px`,
                        top: `${submenuPosition.y}px`,
                    }}
                >
                    <ul className="context-menu-list">
                        {item.submenu.map((subItem, index) => {
                            // Render separator
                            if (subItem.type === 'separator') {
                                return <li key={`sub-separator-${index}`} className="context-menu-separator"></li>;
                            }

                            // Render submenu item
                            return (
                                <ContextMenuItem
                                    key={subItem.id || `sub-item-${index}`}
                                    item={subItem}
                                    onAction={() => {
                                        console.log('Submenu item clicked:', subItem.label);
                                        if (subItem.action) {
                                            try {
                                                subItem.action();
                                                console.log('Submenu action executed successfully');
                                            } catch (error) {
                                                console.error('Submenu action failed:', error);
                                            }
                                        }
                                        // Propagate onAction to close the main menu
                                        if (onAction) {
                                            setTimeout(() => onAction(), 10);
                                        }
                                    }}
                                />
                            );
                        })}
                    </ul>
                </div>
            )}
        </li>
    );
};

export default ContextMenuItem;

