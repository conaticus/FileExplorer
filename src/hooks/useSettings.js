import { useContext } from 'react';
import { SettingsContext } from '../providers/SettingsProvider';

/**
 * Hook for accessing and modifying application settings.
 * @returns {Object} Settings state and functions.
 */
const useSettings = () => {
    const context = useContext(SettingsContext);

    if (!context) {
        throw new Error('useSettings must be used within a SettingsProvider');
    }

    return context;
};

export default useSettings;