import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';

// Default settings
const defaultSettings = {
    theme: 'light',
    defaultView: 'grid',
    showHiddenFiles: false,
    sortBy: 'name',
    sortDirection: 'asc',
    showDetailsPanel: false,
    terminalHeight: 240,
};

// Create context
const SettingsContext = createContext({
    settings: defaultSettings,
    isLoading: true,
    updateSetting: () => {},
    updateMultipleSettings: () => {},
    resetSettings: () => {},
});

// Provider component
export default function SettingsProvider({ children }) {
    const [settings, setSettings] = useState(defaultSettings);
    const [isLoading, setIsLoading] = useState(true);

    // Load settings on mount
    useEffect(() => {
        const loadSettings = async () => {
            setIsLoading(true);

            try {
                // Try to get settings from backend
                const settingsJson = await invoke('get_settings_as_json');
                const loadedSettings = JSON.parse(settingsJson);

                // Merge with default settings to ensure all fields exist
                setSettings({
                    ...defaultSettings,
                    ...loadedSettings,
                });
            } catch (error) {
                console.error('Failed to load settings:', error);

                // Use default settings if loading fails
                setSettings(defaultSettings);

                // Try to save default settings
                try {
                    await invoke('update_multiple_settings_command', {
                        updates: defaultSettings,
                    });
                } catch (saveError) {
                    console.error('Failed to save default settings:', saveError);
                }
            } finally {
                setIsLoading(false);
            }
        };

        loadSettings();
    }, []);

    // Update a single setting
    const updateSetting = async (key, value) => {
        try {
            // Update in backend
            await invoke('update_settings_field', { key, value });

            // Update local state
            setSettings(prev => ({
                ...prev,
                [key]: value,
            }));
        } catch (error) {
            console.error(`Failed to update setting ${key}:`, error);

            // Update local state anyway for better UX
            setSettings(prev => ({
                ...prev,
                [key]: value,
            }));
        }
    };

    // Update multiple settings at once
    const updateMultipleSettings = async (updates) => {
        try {
            // Update in backend
            await invoke('update_multiple_settings_command', { updates });

            // Update local state
            setSettings(prev => ({
                ...prev,
                ...updates,
            }));
        } catch (error) {
            console.error('Failed to update multiple settings:', error);

            // Update local state anyway for better UX
            setSettings(prev => ({
                ...prev,
                ...updates,
            }));
        }
    };

    // Reset settings to defaults
    const resetSettings = async () => {
        try {
            // Reset in backend
            await invoke('reset_settings');

            // Update local state
            setSettings(defaultSettings);
        } catch (error) {
            console.error('Failed to reset settings:', error);

            // Update local state anyway for better UX
            setSettings(defaultSettings);
        }
    };

    const contextValue = {
        settings,
        isLoading,
        updateSetting,
        updateMultipleSettings,
        resetSettings,
    };

    // Show loading state if settings are not loaded yet
    if (isLoading) {
        return (
            <div style={{
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                height: '100%'
            }}>
                <div style={{
                    width: '40px',
                    height: '40px',
                    border: '3px solid var(--background-tertiary)',
                    borderTopColor: 'var(--accent)',
                    borderRadius: '50%',
                    animation: 'spin 1s linear infinite'
                }} />
            </div>
        );
    }

    return (
        <SettingsContext.Provider value={contextValue}>
            {children}
        </SettingsContext.Provider>
    );
}

// Custom hook for using the settings context
export const useSettings = () => useContext(SettingsContext);