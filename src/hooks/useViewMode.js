import { useState, useCallback, useEffect } from 'react';
import { useSettings } from './useSettings';

/**
 * Available view modes for the file explorer.
 */
export const VIEW_MODES = {
    GRID: 'grid',
    LIST: 'list',
    DETAILS: 'details',
};

/**
 * Hook for managing file explorer view modes.
 * @param {string} initialMode - The initial view mode.
 * @returns {Object} View mode state and functions.
 */
const useViewMode = (initialMode = VIEW_MODES.GRID) => {
    const { settings, updateSetting } = useSettings();
    const [viewMode, setViewMode] = useState(initialMode);

    // Initialize from settings
    useEffect(() => {
        if (settings.defaultView) {
            setViewMode(settings.defaultView);
        }
    }, [settings.defaultView]);

    // Update view mode
    const changeViewMode = useCallback(
        (newMode) => {
            if (!Object.values(VIEW_MODES).includes(newMode)) {
                console.error(`Invalid view mode: ${newMode}`);
                return;
            }

            setViewMode(newMode);

            // Save as default if user changes it
            updateSetting('defaultView', newMode);
        },
        [updateSetting]
    );

    // Toggle between view modes
    const toggleViewMode = useCallback(() => {
        const modes = Object.values(VIEW_MODES);
        const currentIndex = modes.indexOf(viewMode);
        const nextIndex = (currentIndex + 1) % modes.length;
        changeViewMode(modes[nextIndex]);
    }, [viewMode, changeViewMode]);

    // Get column count for grid layout
    const getColumnCount = useCallback(() => {
        // This would typically be calculated based on the container width
        // For simplicity, we'll return fixed values based on view mode
        if (viewMode === VIEW_MODES.GRID) {
            return 4; // Default column count for grid view
        } else if (viewMode === VIEW_MODES.LIST) {
            return 1; // List view has one column
        } else if (viewMode === VIEW_MODES.DETAILS) {
            return 1; // Details view has one column
        }

        return 4; // Default fallback
    }, [viewMode]);

    return {
        viewMode,
        changeViewMode,
        toggleViewMode,
        getColumnCount,
        VIEW_MODES,
    };
};

export default useViewMode;