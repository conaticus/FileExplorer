import React from 'react';
import Icon from './Icon';
import Tooltip from './Tooltip';
import './common.css';

/**
 * IconButton component for icon-only buttons
 * @param {Object} props - Component props
 * @param {string} props.icon - Icon name to display
 * @param {string} [props.variant='ghost'] - Button variant (primary, secondary, ghost, danger)
 * @param {string} [props.size='md'] - Button size (sm, md, lg)
 * @param {string} [props.tooltip] - Optional tooltip text
 * @param {string} [props.tooltipPosition='top'] - Tooltip position (top, bottom, left, right)
 * @param {boolean} [props.disabled=false] - Whether button is disabled
 * @param {Function} props.onClick - Click handler
 * @param {string} [props.className] - Additional CSS class names
 * @param {string} [props.label] - Accessible label for the button (required for a11y)
 * @param {Object} [props.rest] - Additional props to pass to the button element
 * @returns {React.ReactElement} IconButton component
 */
const IconButton = ({
                        icon,
                        variant = 'ghost',
                        size = 'md',
                        tooltip,
                        tooltipPosition = 'top',
                        disabled = false,
                        onClick,
                        className = '',
                        label,
                        ...rest
                    }) => {
    /**
     * Erzeugt die CSS-Klassen für den Button basierend auf den Props
     * @type {string}
     */
    const buttonClasses = [
        'btn',
        'btn-icon',
        `btn-${variant}`,
        size !== 'md' ? `btn-${size}` : '',
        className
    ].filter(Boolean).join(' ');

    /**
     * Bestimmt die Icon-Größe basierend auf der Button-Größe
     * @type {string}
     */
    const iconSize = size === 'sm' ? 'small' : size === 'lg' ? 'large' : 'medium';

    const button = (
        <button
            className={buttonClasses}
            onClick={onClick}
            disabled={disabled}
            aria-label={label || tooltip}
            {...rest}
        >
            <Icon name={icon} size={iconSize} />
        </button>
    );

    // Wrap with tooltip if provided
    if (tooltip && !disabled) {
        return (
            <Tooltip text={tooltip} position={tooltipPosition}>
                {button}
            </Tooltip>
        );
    }

    return button;
};

export default IconButton;