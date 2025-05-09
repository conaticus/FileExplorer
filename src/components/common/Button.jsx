import React from 'react';
import './common.css';

/**
 * Button component
 * @param {Object} props - Component props
 * @param {string} [props.variant='primary'] - Button variant (primary, secondary, ghost, danger)
 * @param {string} [props.size='md'] - Button size (sm, md, lg)
 * @param {boolean} [props.fullWidth=false] - Whether button should take full width
 * @param {boolean} [props.disabled=false] - Whether button is disabled
 * @param {Function} props.onClick - Click handler
 * @param {React.ReactNode} props.children - Button content
 * @param {string} [props.className] - Additional CSS class names
 * @param {Object} [props.rest] - Additional props to pass to the button element
 * @returns {React.ReactElement} Button component
 */
const Button = ({
                    variant = 'primary',
                    size = 'md',
                    fullWidth = false,
                    disabled = false,
                    onClick,
                    children,
                    className = '',
                    ...rest
                }) => {
    // Build class name based on props
    const buttonClasses = [
        'btn',
        `btn-${variant}`,
        size !== 'md' ? `btn-${size}` : '',
        fullWidth ? 'btn-full-width' : '',
        className
    ].filter(Boolean).join(' ');

    return (
        <button
            className={buttonClasses}
            onClick={onClick}
            disabled={disabled}
            {...rest}
        >
            {children}
        </button>
    );
};

export default Button;