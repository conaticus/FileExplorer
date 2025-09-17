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
    const [submenuPosition, setSubmenuPosition] = useState({ x: 0, y: 0, alignLeft: false });
    const itemRef = useRef(null);

    /**
     * Calculates the submenu position based on the parent element position and available space
     */
    useEffect(() => {
        if (isSubmenuOpen && itemRef.current && item.submenu) {
            const itemRect = itemRef.current.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const submenuWidth = 180; // min-width from CSS
            const padding = 8; // safety padding

            let x = itemRect.right; // default to right side
            let alignLeft = false;

            // Check if submenu would overflow the right edge of the viewport
            if (itemRect.right + submenuWidth + padding > viewportWidth) {
                // Not enough space on the right, try left side
                if (itemRect.left - submenuWidth - padding >= 0) {
                    // Enough space on the left, position on left
                    x = itemRect.left - submenuWidth;
                    alignLeft = true;
                } else {
                    // Not enough space on either side, keep on right but adjust to fit
                    x = viewportWidth - submenuWidth - padding;
                    alignLeft = false;
                }
            }

            setSubmenuPosition({
                x: x,
                y: itemRect.top,
                alignLeft: alignLeft
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
          <span className={`icon ${submenuPosition.alignLeft ? 'icon-chevron-left' : 'icon-chevron-right'}`}></span>
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

