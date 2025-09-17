import React from 'react';
import './viewModes.css';

/**
 * ViewModes component - Displays and handles switching between different file view modes
 * @param {Object} props - Component props
 * @param {string} [props.currentMode='grid'] - Currently active view mode
 * @param {Function} [props.onChange] - Callback function when view mode changes
 * @returns {React.ReactElement} ViewModes component
 */
const ViewModes = ({ currentMode = 'grid', onChange }) => {
    /**
     * Available view modes configuration
     * @type {Array<{id: string, label: string, icon: string}>}
     */
    const viewModes = [
        { id: 'grid', label: 'Grid View', icon: 'grid' },
        { id: 'list', label: 'List View', icon: 'list' },
        { id: 'details', label: 'Details View', icon: 'details' },
    ];

    /**
     * Handles view mode change and invokes the onChange callback
     * @param {string} mode - The selected view mode ID
     */
    const handleViewModeChange = (mode) => {
        if (onChange) {
            onChange(mode);
        }
    };

    return (
        <div className="view-modes">
            {viewModes.map((mode) => (
                <button
                    key={mode.id}
                    className={`view-mode-button ${currentMode === mode.id ? 'active' : ''}`}
                    onClick={() => handleViewModeChange(mode.id)}
                    aria-label={mode.label}
                    title={mode.label}
                >
                    <span className={`icon icon-${mode.icon}`}></span>
                </button>
            ))}
        </div>
    );
};

export default ViewModes;