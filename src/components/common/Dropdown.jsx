import React, { useState, useRef, useEffect } from 'react';
import Icon from './Icon';
import './common.css';

/**
 * Dropdown component
 * @param {Object} props - Component props
 * @param {React.ReactNode} props.trigger - Element that triggers the dropdown
 * @param {Array} props.items - Array of dropdown items
 * @param {string} [props.align='left'] - Dropdown alignment (left, right)
 * @param {string} [props.width] - Custom width for the dropdown
 * @param {boolean} [props.closeOnClick=true] - Whether to close dropdown when an item is clicked
 * @param {Function} [props.onOpen] - Callback when dropdown opens
 * @param {Function} [props.onClose] - Callback when dropdown closes
 * @param {string} [props.className] - Additional CSS class names
 * @returns {React.ReactElement} Dropdown component
 */
const Dropdown = ({
                      trigger,
                      items = [],
                      align = 'left',
                      width,
                      closeOnClick = true,
                      onOpen,
                      onClose,
                      className = '',
                      ...rest
                  }) => {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef(null);

    // Toggle dropdown open/closed
    const toggleDropdown = () => {
        if (!isOpen && onOpen) onOpen();
        if (isOpen && onClose) onClose();
        setIsOpen(!isOpen);
    };

    // Close dropdown when clicking outside
    useEffect(() => {
        const handleClickOutside = (event) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target)) {
                if (isOpen && onClose) onClose();
                setIsOpen(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isOpen, onClose]);

    // Close dropdown when Escape key is pressed
    useEffect(() => {
        const handleKeyDown = (event) => {
            if (event.key === 'Escape' && isOpen) {
                if (onClose) onClose();
                setIsOpen(false);
            }
        };

        document.addEventListener('keydown', handleKeyDown);

        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [isOpen, onClose]);

    // Handle item click
    const handleItemClick = (item) => {
        if (item.onClick) {
            item.onClick();
        }

        if (closeOnClick) {
            if (onClose) onClose();
            setIsOpen(false);
        }
    };

    // Build class name based on props
    const dropdownClasses = [
        'dropdown',
        className
    ].filter(Boolean).join(' ');

    const menuClasses = [
        'dropdown-menu',
        `dropdown-align-${align}`,
        isOpen ? 'dropdown-open' : ''
    ].filter(Boolean).join(' ');

    const menuStyle = width ? { width } : {};

    return (
        <div className={dropdownClasses} ref={dropdownRef} {...rest}>
            <div className="dropdown-trigger" onClick={toggleDropdown}>
                {trigger}
            </div>

            {isOpen && (
                <ul className={menuClasses} style={menuStyle}>
                    {items.map((item, index) => {
                        // Handle separator
                        if (item.type === 'separator') {
                            return <li key={`separator-${index}`} className="dropdown-separator" />;
                        }

                        // Handle header
                        if (item.type === 'header') {
                            return (
                                <li key={`header-${index}`} className="dropdown-header">
                                    {item.label}
                                </li>
                            );
                        }

                        // Regular dropdown item
                        return (
                            <li key={item.id || `item-${index}`}>
                                <button
                                    className={`dropdown-item ${item.className || ''} ${item.disabled ? 'disabled' : ''}`}
                                    onClick={() => handleItemClick(item)}
                                    disabled={item.disabled}
                                >
                                    {item.icon && (
                                        <span className="dropdown-item-icon">
                      <Icon name={item.icon} size="small" />
                    </span>
                                    )}

                                    <span className="dropdown-item-label">{item.label}</span>

                                    {item.shortcut && (
                                        <span className="dropdown-item-shortcut">{item.shortcut}</span>
                                    )}

                                    {item.submenu && (
                                        <span className="dropdown-item-caret">
                      <Icon name="chevron-right" size="small" />
                    </span>
                                    )}
                                </button>

                                {/* Render submenu if exists and is open */}
                                {item.submenu && item.isOpen && (
                                    <ul className="dropdown-submenu">
                                        {item.submenu.map((subItem, subIndex) => (
                                            <li key={subItem.id || `subitem-${subIndex}`}>
                                                <button
                                                    className={`dropdown-item ${subItem.className || ''} ${subItem.disabled ? 'disabled' : ''}`}
                                                    onClick={() => handleItemClick(subItem)}
                                                    disabled={subItem.disabled}
                                                >
                                                    {subItem.icon && (
                                                        <span className="dropdown-item-icon">
                              <Icon name={subItem.icon} size="small" />
                            </span>
                                                    )}

                                                    <span className="dropdown-item-label">{subItem.label}</span>
                                                </button>
                                            </li>
                                        ))}
                                    </ul>
                                )}
                            </li>
                        );
                    })}
                </ul>
            )}
        </div>
    );
};

export default Dropdown;