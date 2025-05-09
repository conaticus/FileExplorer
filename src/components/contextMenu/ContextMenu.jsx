import React, { useEffect, useRef, useState } from 'react';
import ContextMenuItem from './ContextMenuItem';
import './contextMenu.css';

const ContextMenu = ({ position, items = [], onClose }) => {
    const menuRef = useRef(null);
    const [adjustedPosition, setAdjustedPosition] = useState(position);
    const [openSubmenu, setOpenSubmenu] = useState(null);

    // Calculate menu position to ensure it stays within viewport
    useEffect(() => {
        if (!menuRef.current) return;

        const menuRect = menuRef.current.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        let adjustedX = position.x;
        let adjustedY = position.y;

        // Adjust horizontal position if menu would overflow right edge
        if (position.x + menuRect.width > viewportWidth) {
            adjustedX = viewportWidth - menuRect.width - 8; // 8px padding
        }

        // Adjust vertical position if menu would overflow bottom edge
        if (position.y + menuRect.height > viewportHeight) {
            adjustedY = viewportHeight - menuRect.height - 8; // 8px padding
        }

        setAdjustedPosition({ x: adjustedX, y: adjustedY });
    }, [position, items]);

    // Close menu when clicking outside
    useEffect(() => {
        const handleOutsideClick = (e) => {
            if (menuRef.current && !menuRef.current.contains(e.target)) {
                if (onClose) onClose();
            }
        };

        // Use capture phase to ensure the click is captured
        // before it bubbles to other elements
        document.addEventListener('click', handleOutsideClick, { capture: true });

        return () => {
            document.removeEventListener('click', handleOutsideClick, { capture: true });
        };
    }, [onClose]);

    // Close menu on Escape key
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.key === 'Escape') {
                if (onClose) onClose();
            }
        };

        document.addEventListener('keydown', handleKeyDown);

        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [onClose]);

    // Handle submenu opening
    const handleSubmenuOpen = (id) => {
        setOpenSubmenu(id);
    };

    // Handle submenu closing
    const handleSubmenuClose = () => {
        setOpenSubmenu(null);
    };

    return (
        <div
            ref={menuRef}
            className="context-menu"
            style={{
                left: `${adjustedPosition.x}px`,
                top: `${adjustedPosition.y}px`,
            }}
            onClick={(e) => e.stopPropagation()}
        >
            <ul className="context-menu-list">
                {items.map((item, index) => {
                    // Render separator
                    if (item.type === 'separator') {
                        return <li key={`separator-${index}`} className="context-menu-separator"></li>;
                    }

                    // Render menu item
                    return (
                        <ContextMenuItem
                            key={item.id || `item-${index}`}
                            item={item}
                            isSubmenuOpen={openSubmenu === item.id}
                            onSubmenuOpen={() => handleSubmenuOpen(item.id)}
                            onSubmenuClose={handleSubmenuClose}
                            onAction={() => {
                                if (onClose) onClose();
                                if (item.action) item.action();
                            }}
                        />
                    );
                })}
            </ul>
        </div>
    );
};

export default ContextMenu;