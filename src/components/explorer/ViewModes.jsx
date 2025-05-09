import React from 'react';
import './viewModes.css';

const ViewModes = ({ currentMode = 'grid', onChange }) => {
    // Available view modes
    const viewModes = [
        { id: 'grid', label: 'Grid View', icon: 'grid' },
        { id: 'list', label: 'List View', icon: 'list' },
        { id: 'details', label: 'Details View', icon: 'details' },
    ];

    // Handle view mode change
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