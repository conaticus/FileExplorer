import React, { useState, useRef, useEffect } from 'react';

/**
 * Tooltip-Komponente zur Anzeige von Hilfetexten bei Hover
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {React.ReactNode} props.children - Element, das den Tooltip auslöst
 * @param {string} props.content - Inhalt des Tooltips
 * @param {string} [props.position='top'] - Position des Tooltips (top, right, bottom, left)
 * @param {number} [props.delay=300] - Verzögerung in ms, bevor der Tooltip angezeigt wird
 * @param {string} [props.className] - Zusätzliche CSS-Klassen
 */
const Tooltip = ({
                     children,
                     content,
                     position = 'top',
                     delay = 300,
                     className = '',
                 }) => {
    const [isVisible, setIsVisible] = useState(false);
    const tooltipRef = useRef(null);
    const triggerRef = useRef(null);
    const timeoutRef = useRef(null);

    // Behandle den Mauszeiger über dem Element
    const handleMouseEnter = () => {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = setTimeout(() => {
            setIsVisible(true);
        }, delay);
    };

    // Behandle den Mauszeiger, der das Element verlässt
    const handleMouseLeave = () => {
        clearTimeout(timeoutRef.current);
        setIsVisible(false);
    };

    // Positioniere den Tooltip basierend auf dem Trigger-Element
    useEffect(() => {
        if (isVisible && tooltipRef.current && triggerRef.current) {
            const triggerRect = triggerRef.current.getBoundingClientRect();
            const tooltipRect = tooltipRef.current.getBoundingClientRect();

            // Standardversatz
            const offset = 8;

            // Positioniere den Tooltip basierend auf der ausgewählten Position
            let tooltipStyle = {};

            switch (position) {
                case 'top':
                    tooltipStyle = {
                        top: `${triggerRect.top - tooltipRect.height - offset}px`,
                        left: `${triggerRect.left + (triggerRect.width / 2) - (tooltipRect.width / 2)}px`,
                    };
                    break;
                case 'right':
                    tooltipStyle = {
                        top: `${triggerRect.top + (triggerRect.height / 2) - (tooltipRect.height / 2)}px`,
                        left: `${triggerRect.right + offset}px`,
                    };
                    break;
                case 'bottom':
                    tooltipStyle = {
                        top: `${triggerRect.bottom + offset}px`,
                        left: `${triggerRect.left + (triggerRect.width / 2) - (tooltipRect.width / 2)}px`,
                    };
                    break;
                case 'left':
                    tooltipStyle = {
                        top: `${triggerRect.top + (triggerRect.height / 2) - (tooltipRect.height / 2)}px`,
                        left: `${triggerRect.left - tooltipRect.width - offset}px`,
                    };
                    break;
                default:
                    break;
            }

            // Stelle sicher, dass der Tooltip im sichtbaren Bereich bleibt
            const viewport = {
                width: window.innerWidth,
                height: window.innerHeight,
            };

            // Horizontale Ausrichtung anpassen
            if (tooltipStyle.left < 0) {
                tooltipStyle.left = offset;
            } else if (tooltipStyle.left + tooltipRect.width > viewport.width) {
                tooltipStyle.left = viewport.width - tooltipRect.width - offset;
            }

            // Vertikale Ausrichtung anpassen
            if (tooltipStyle.top < 0) {
                tooltipStyle.top = offset;
            } else if (tooltipStyle.top + tooltipRect.height > viewport.height) {
                tooltipStyle.top = viewport.height - tooltipRect.height - offset;
            }

            // Wende die Stile auf das Tooltip-Element an
            Object.assign(tooltipRef.current.style, tooltipStyle);
        }
    }, [isVisible, position]);

    // Bereinige den Timeout beim Unmount
    useEffect(() => {
        return () => {
            clearTimeout(timeoutRef.current);
        };
    }, []);

    // CSS-Klassen für den Tooltip
    const tooltipClasses = [
        'tooltip',
        `tooltip-${position}`,
        isVisible ? 'tooltip-visible' : '',
        className,
    ].filter(Boolean).join(' ');

    return (
        <div
            className="tooltip-container"
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
            ref={triggerRef}
        >
            {children}

            {isVisible && (
                <div
                    className={tooltipClasses}
                    ref={tooltipRef}
                    role="tooltip"
                >
                    {content}
                </div>
            )}
        </div>
    );
};

export default Tooltip;