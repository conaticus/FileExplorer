import React, { useState, useRef, useEffect } from 'react';

/**
 * Dropdown-Komponente für Auswahllisten und Menüs
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {React.ReactNode} props.trigger - Auslöser-Element für das Dropdown
 * @param {React.ReactNode[]} props.children - Inhalt des Dropdowns
 * @param {string} [props.position='bottom-start'] - Position des Dropdown-Menüs (bottom-start, bottom-end, top-start, top-end)
 * @param {boolean} [props.isFullWidth=false] - Ob das Dropdown die volle Breite ausfüllen soll
 * @param {string} [props.className] - Zusätzliche CSS-Klassen
 * @param {Function} [props.onOpen] - Callback, wenn das Dropdown geöffnet wird
 * @param {Function} [props.onClose] - Callback, wenn das Dropdown geschlossen wird
 */
const Dropdown = ({
                      trigger,
                      children,
                      position = 'bottom-start',
                      isFullWidth = false,
                      className = '',
                      onOpen,
                      onClose,
                  }) => {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef(null);

    // Positioniere das Dropdown-Menü basierend auf der gewählten Position
    const getPositionStyles = () => {
        switch (position) {
            case 'bottom-start':
                return { top: '100%', left: 0 };
            case 'bottom-end':
                return { top: '100%', right: 0 };
            case 'top-start':
                return { bottom: '100%', left: 0 };
            case 'top-end':
                return { bottom: '100%', right: 0 };
            default:
                return { top: '100%', left: 0 };
        }
    };

    // CSS-Klassen für das Dropdown
    const dropdownClasses = [
        'dropdown',
        isOpen ? 'dropdown-open' : '',
        isFullWidth ? 'dropdown-full-width' : '',
        className
    ].filter(Boolean).join(' ');

    // Öffne das Dropdown
    const openDropdown = () => {
        setIsOpen(true);
        if (onOpen) onOpen();
    };

    // Schließe das Dropdown
    const closeDropdown = () => {
        setIsOpen(false);
        if (onClose) onClose();
    };

    // Toggle das Dropdown
    const toggleDropdown = () => {
        if (isOpen) {
            closeDropdown();
        } else {
            openDropdown();
        }
    };

    // Behandle Klicks außerhalb des Dropdowns
    useEffect(() => {
        const handleClickOutside = (event) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target)) {
                closeDropdown();
            }
        };

        // Behandle Escape-Taste zum Schließen
        const handleEscapeKey = (event) => {
            if (event.key === 'Escape') {
                closeDropdown();
            }
        };

        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
            document.addEventListener('keydown', handleEscapeKey);
        }

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleEscapeKey);
        };
    }, [isOpen]);

    return (
        <div className={dropdownClasses} ref={dropdownRef}>
            <div className="dropdown-trigger" onClick={toggleDropdown}>
                {trigger}
            </div>

            {isOpen && (
                <div
                    className="dropdown-menu"
                    style={{
                        ...getPositionStyles(),
                        width: isFullWidth ? '100%' : 'auto'
                    }}
                >
                    {children}
                </div>
            )}
        </div>
    );
};

/**
 * Dropdown-Element-Komponente für einzelne Menüelemente
 */
export const DropdownItem = ({
                                 children,
                                 icon,
                                 onClick,
                                 isDisabled = false,
                                 className = ''
                             }) => {
    const itemClasses = [
        'dropdown-item',
        isDisabled ? 'dropdown-item-disabled' : '',
        className
    ].filter(Boolean).join(' ');

    // Rendere das Icon basierend auf dem SVG-Pfad
    const renderIcon = () => {
        if (!icon) return null;

        return (
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                width="16"
                height="16"
                className="dropdown-item-icon"
            >
                <path d={icon} />
            </svg>
        );
    };

    return (
        <div
            className={itemClasses}
            onClick={isDisabled ? undefined : onClick}
        >
            {icon && renderIcon()}
            <span className="dropdown-item-text">{children}</span>
        </div>
    );
};

/**
 * Dropdown-Separator-Komponente für die visuelle Trennung von Menüelementen
 */
export const DropdownSeparator = () => {
    return <div className="dropdown-separator"></div>;
};

export default Dropdown;