import React from 'react';

const ContextMenuItem = ({
                             id,
                             label,
                             icon,
                             disabled = false,
                             onClick,
                             className = '',
                         }) => {
    // Handle icon path
    const getIconPath = (icon) => {
        if (!icon) return null;

        // Wenn es ein SVG-Pfad ist
        if (typeof icon === 'string' && icon.includes('M')) {
            return icon;
        }

        // Vordefinierte Icons
        const icons = {
            open: 'M3 12h18M12 3v18',
            copy: 'M16 4h2a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-9a2 2 0 0 1-2-2v-2M8 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2v-2',
            cut: 'M6 9l6 6m0-6l-6 6m12-6a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm0 6a2 2 0 1 0 0-4 2 2 0 0 0 0 4z',
            paste: 'M16 4h2a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2 M14 2h-4a2 2 0 0 0-2 2v2h8V4a2 2 0 0 0-2-2z',
            rename: 'M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z',
            delete: 'M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2',
            properties: 'M4 21v-13a3 3 0 0 1 3-3h10a3 3 0 0 1 3 3v13M9 5h6M4 10h16M4 15h16M4 21h16',
            new: 'M12 5v14M5 12h14',
            edit: 'M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7M18.5 2.5a2.12 2.12 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z',
            download: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3',
            upload: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12',
            template: 'M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z M3 6l18 0 M16 10a4 4 0 0 1-8 0',
            pin: 'M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z M12 7a3 3 0 1 0 0 6 3 3 0 0 0 0-6z',
            folder: 'M10 3H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h16a1 1 0 0 0 1-1V7a1 1 0 0 0-1-1h-8l-2-2z',
            file: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6',
        };

        return icons[icon] || null;
    };

    const iconPath = getIconPath(icon);

    return (
        <div
            className={`context-menu-item ${disabled ? 'disabled' : ''} ${className}`}
            onClick={disabled ? undefined : onClick}
            role="menuitem"
            tabIndex={disabled ? -1 : 0}
            aria-disabled={disabled}
        >
            {iconPath && (
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
                    className="context-menu-item-icon"
                >
                    <path d={iconPath} />
                </svg>
            )}
            <span className="context-menu-item-label">{label}</span>
        </div>
    );
};

export default ContextMenuItem;