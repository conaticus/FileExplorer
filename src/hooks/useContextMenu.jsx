import { useState, useCallback, useEffect } from 'react';

/**
 * Hook zur Verwaltung eines Kontextmenüs
 *
 * @returns {Object} Kontextmenü-Funktionen und -Zustand
 */
const useContextMenu = () => {
    const [isOpen, setIsOpen] = useState(false);
    const [position, setPosition] = useState({ x: 0, y: 0 });
    const [target, setTarget] = useState(null);

    // Öffne das Kontextmenü an der angegebenen Position
    const openContextMenu = useCallback((event, targetItem = null) => {
        event.preventDefault();
        event.stopPropagation();

        setPosition({
            x: event.clientX,
            y: event.clientY
        });
        setTarget(targetItem);
        setIsOpen(true);
    }, []);

    // Schließe das Kontextmenü
    const closeContextMenu = useCallback(() => {
        setIsOpen(false);
        setTarget(null);
    }, []);

    // Schließe das Kontextmenü bei Klick außerhalb oder Escape-Taste
    useEffect(() => {
        const handleClickOutside = () => {
            if (isOpen) {
                closeContextMenu();
            }
        };

        const handleEscapeKey = (event) => {
            if (isOpen && event.key === 'Escape') {
                closeContextMenu();
            }
        };

        // Verzögere das Hinzufügen der Event-Listener, um sofortige Schließung zu verhindern
        let timeoutId;
        if (isOpen) {
            timeoutId = setTimeout(() => {
                document.addEventListener('click', handleClickOutside);
                document.addEventListener('keydown', handleEscapeKey);
            }, 100);
        }

        return () => {
            clearTimeout(timeoutId);
            document.removeEventListener('click', handleClickOutside);
            document.removeEventListener('keydown', handleEscapeKey);
        };
    }, [isOpen, closeContextMenu]);

    return {
        isContextMenuOpen: isOpen,
        contextMenuPosition: position,
        contextMenuTarget: target,
        openContextMenu,
        closeContextMenu
    };
};

export default useContextMenu;