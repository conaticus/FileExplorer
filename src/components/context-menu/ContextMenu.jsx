import React, { useEffect, useRef } from 'react';
import ContextMenuItem from './ContextMenuItem';

const ContextMenu = ({
                         position = { x: 0, y: 0 },
                         items = [],
                         onClose,
                         className = '',
                     }) => {
    const menuRef = useRef(null);

    // Sicherstellen, dass das Menü nicht außerhalb des Fensters angezeigt wird
    useEffect(() => {
        const adjustPosition = () => {
            if (!menuRef.current) return;

            const menu = menuRef.current;
            const rect = menu.getBoundingClientRect();
            const windowWidth = window.innerWidth;
            const windowHeight = window.innerHeight;

            // Horizontale Anpassung
            if (position.x + rect.width > windowWidth) {
                menu.style.left = `${windowWidth - rect.width}px`;
            } else {
                menu.style.left = `${position.x}px`;
            }

            // Vertikale Anpassung
            if (position.y + rect.height > windowHeight) {
                menu.style.top = `${windowHeight - rect.height}px`;
            } else {
                menu.style.top = `${position.y}px`;
            }
        };

        adjustPosition();
    }, [position]);

    // Klicks außerhalb des Menüs schließen es
    useEffect(() => {
        const handleClickOutside = (e) => {
            if (menuRef.current && !menuRef.current.contains(e.target)) {
                onClose();
            }
        };

        // ESC-Taste schließt das Menü
        const handleKeyDown = (e) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleKeyDown);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [onClose]);

    // Gruppiere Elemente nach Gruppen für Trennlinien
    const groupedItems = items.reduce((acc, item) => {
        const lastGroup = acc[acc.length - 1];

        if (!lastGroup || item.group !== lastGroup[0].group) {
            acc.push([item]);
        } else {
            lastGroup.push(item);
        }

        return acc;
    }, []);

    return (
        <div
            className={`context-menu ${className}`}
            style={{ top: position.y, left: position.x }}
            ref={menuRef}
        >
            {groupedItems.map((group, groupIndex) => (
                <div key={`group-${groupIndex}`} className="context-menu-group">
                    {groupIndex > 0 && (
                        <div className="context-menu-separator"></div>
                    )}

                    {group.map((item) => (
                        <ContextMenuItem
                            key={item.id}
                            id={item.id}
                            label={item.label}
                            icon={item.icon}
                            disabled={item.disabled}
                            onClick={() => {
                                if (!item.disabled && item.onClick) {
                                    item.onClick();
                                    onClose();
                                }
                            }}
                        />
                    ))}
                </div>
            ))}
        </div>
    );
};

export default ContextMenu;