import React from 'react';

/**
 * Wiederverwendbare Button-Komponente mit verschiedenen Varianten
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {string} [props.variant='primary'] - Variante des Buttons (primary, secondary, tertiary, icon)
 * @param {string} [props.size='medium'] - Größe des Buttons (small, medium, large)
 * @param {boolean} [props.isFullWidth=false] - Ob der Button die volle Breite ausfüllen soll
 * @param {boolean} [props.isDisabled=false] - Ob der Button deaktiviert sein soll
 * @param {string} [props.icon] - Optionales Icon (SVG-Pfad)
 * @param {string} [props.iconPosition='left'] - Position des Icons (left, right)
 * @param {Function} props.onClick - Klick-Handler
 * @param {string} [props.className] - Zusätzliche CSS-Klassen
 * @param {string} [props.type='button'] - Typ des Buttons (button, submit, reset)
 * @param {React.ReactNode} props.children - Kindelemente des Buttons
 */
const Button = ({
                    variant = 'primary',
                    size = 'medium',
                    isFullWidth = false,
                    isDisabled = false,
                    icon,
                    iconPosition = 'left',
                    onClick,
                    className = '',
                    type = 'button',
                    children,
                    ...rest
                }) => {
    // CSS-Klassen basierend auf Eigenschaften
    const buttonClasses = [
        'btn',
        `btn-${variant}`,
        `btn-${size}`,
        isFullWidth ? 'btn-full-width' : '',
        isDisabled ? 'btn-disabled' : '',
        icon && !children ? 'btn-icon-only' : '',
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
                width={size === 'small' ? '14' : size === 'large' ? '20' : '16'}
                height={size === 'small' ? '14' : size === 'large' ? '20' : '16'}
                className="btn-icon"
            >
                <path d={icon} />
            </svg>
        );
    };

    return (
        <button
            type={type}
            className={buttonClasses}
            onClick={onClick}
            disabled={isDisabled}
            {...rest}
        >
            {icon && iconPosition === 'left' && renderIcon()}
            {children && <span className="btn-text">{children}</span>}
            {icon && iconPosition === 'right' && renderIcon()}
        </button>
    );
};

export default Button;