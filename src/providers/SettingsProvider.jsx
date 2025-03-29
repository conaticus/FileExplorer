import React, { createContext, useContext, useState, useEffect } from 'react';
import Settings from '../components/settings/Settings';

// Create the context
export const SettingsContext = createContext();

// Settings Provider Component
export const SettingsProvider = ({ children }) => {
    const [isSettingsOpen, setIsSettingsOpen] = useState(false);

    // Function to open settings modal
    const openSettings = () => {
        setIsSettingsOpen(true);
    };

    // Function to close settings modal
    const closeSettings = () => {
        setIsSettingsOpen(false);
    };

    // Context value
    const contextValue = {
        openSettings,
        closeSettings,
        isSettingsOpen
    };

    return (
        <SettingsContext.Provider value={contextValue}>
            {children}
            <Settings isOpen={isSettingsOpen} onClose={closeSettings} />
        </SettingsContext.Provider>
    );
};

// Custom Hook to use the settings context
export const useSettings = () => {
    const context = useContext(SettingsContext);
    if (context === undefined) {
        throw new Error('useSettings must be used within a SettingsProvider');
    }
    return context;
};

export default SettingsProvider;