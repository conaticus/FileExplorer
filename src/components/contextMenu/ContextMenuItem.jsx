import React, { useRef, useState, useEffect } from 'react';

const ContextMenuItem = ({
                             item,
                             isSubmenuOpen = false,
                             onSubmenuOpen,
                             onSubmenuClose,
                             onAction,
                         }) => {
    const [submenuPosition, setSubmenuPosition] = useState({ x: 0, y: 0 });
    const itemRef = useRef(null);

    // Calculate submenu position
    useEffect(() => {
        if (isSubmenuOpen && itemRef.current && item.submenu) {
            const itemRect = itemRef.current.getBoundingClientRect();

            setSubmenuPosition({
                x: itemRect.right,
                y: itemRect.top,
            });
        }
    }, [isSubmenuOpen, item.submenu]);

    // Handle mouse enter - open submenu
    const handleMouseEnter = () => {
        if (item.submenu) {
            onSubmenuOpen();
        }
    };

    // Handle mouse leave - close submenu
    const handleMouseLeave = () => {
        if (item.submenu) {
            // Add delay to prevent submenu from closing immediately
            setTimeout(() => {
                onSubmenuClose();
            }, 100);
        }
    };

    // Handle click
    const handleClick = (e) => {
        e.stopPropagation();

        if (item.disabled) return;

        if (item.submenu) {
            onSubmenuOpen();
        } else if (onAction) {
            onAction();
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
                                    onAction={onAction}
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