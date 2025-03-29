import React from 'react';

const ViewModeSelector = ({ currentMode, onChange }) => {
    // View-Modi
    const viewModes = [
        { id: 'list', icon: 'M3 4h18M3 8h18M3 12h18M3 16h18M3 20h18', label: 'Liste' },
        { id: 'grid', icon: 'M3 3h7v7H3V3zm11 0h7v7h-7V3zm0 11h7v7h-7v-7zM3 14h7v7H3v-7z', label: 'Raster' },
        { id: 'details', icon: 'M3 4h18M3 8h12M3 12h18M3 16h12M3 20h18', label: 'Details' },
    ];

    return (
        <div className="view-mode-selector">
            {viewModes.map((mode) => (
                <button
                    key={mode.id}
                    className={`view-mode-button ${currentMode === mode.id ? 'active' : ''}`}
                    onClick={() => onChange(mode.id)}
                    title={mode.label}
                    aria-label={`Ansichtsmodus: ${mode.label}`}
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        width="18"
                        height="18"
                    >
                        <path d={mode.icon} />
                    </svg>
                </button>
            ))}
        </div>
    );
};

export default ViewModeSelector;