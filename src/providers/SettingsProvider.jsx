import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Default settings - using exact backend keys and supported values
const defaultSettings = {
    // Core UI settings
    darkmode: false,
    custom_themes: [],
    default_theme: "",
    default_themes_path: "",
    default_folder_path_on_opening: "",
    default_view: "grid", // Grid, List, Details
    font_size: "Medium", // Small, Medium, Large
    show_hidden_files_and_folders: false,
    show_details_panel: false,
    accent_color: "#0672ef",

    // Behavior settings
    confirm_delete: true,
    auto_refresh_dir: true,
    sort_direction: "Ascending", // Ascending, Descending
    sort_by: "Name", // Name, Size, Modified, Type
    double_click: "OpenFilesAndFolders", // OpenFilesAndFolders, SelectFilesAndFolders
    show_file_extensions: true,

    // Interface settings
    terminal_height: 240,
    enable_animations_and_transitions: true,
    enable_virtual_scroll_for_large_directories: false,

    // Search settings
    enable_suggestions: true,
    highlight_matches: true,

    // Backend settings that are nested but we'll manage them
    search_engine_enabled: true,
    case_sensitive_search: false,
    index_hidden_files: false,
    fuzzy_search_enabled: true,
    default_checksum_hash: "SHA256",
};

// Create context
const SettingsContext = createContext({
    settings: defaultSettings,
    isLoading: true,
    error: null,
    updateSetting: () => {},
    updateMultipleSettings: () => {},
    resetSettings: () => {},
    reloadSettings: () => {},
});

// Provider component
export default function SettingsProvider({ children }) {
    const [settings, setSettings] = useState(defaultSettings);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);

    // Load settings from backend
    const loadSettings = async () => {
        setIsLoading(true);
        setError(null);

        try {
            console.log('Loading settings from backend...');
            const settingsJson = await invoke('get_settings_as_json');
            const loadedSettings = JSON.parse(settingsJson);

            console.log('Loaded settings:', loadedSettings);

            // Extract nested backend settings for easier access
            const flattenedSettings = {
                ...loadedSettings,
                // Extract search engine settings from nested structure
                ...(loadedSettings.backend_settings?.search_engine_config && {
                    search_engine_enabled: loadedSettings.backend_settings.search_engine_config.search_engine_enabled,
                    case_sensitive_search: loadedSettings.backend_settings.search_engine_config.case_sensitive_search,
                    index_hidden_files: loadedSettings.backend_settings.search_engine_config.index_hidden_files,
                    fuzzy_search_enabled: loadedSettings.backend_settings.search_engine_config.fuzzy_search_enabled,
                }),
                // Extract other backend settings
                ...(loadedSettings.backend_settings && {
                    default_checksum_hash: loadedSettings.backend_settings.default_checksum_hash,
                }),
            };

            // Merge with default settings to ensure all fields exist
            const mergedSettings = {
                ...defaultSettings,
                ...flattenedSettings,
            };

            setSettings(mergedSettings);
            console.log('Settings loaded successfully');
        } catch (error) {
            console.error('Failed to load settings:', error);
            setError('Failed to load settings from backend');

            // Use default settings if loading fails
            setSettings(defaultSettings);

            // Try to save default settings to backend
            try {
                console.log('Saving default settings to backend...');
                await invoke('update_multiple_settings_command', {
                    updates: defaultSettings,
                });
                console.log('Default settings saved successfully');
            } catch (saveError) {
                console.error('Failed to save default settings:', saveError);
                setError('Failed to load and save default settings');
            }
        } finally {
            setIsLoading(false);
        }
    };

    // Load settings on mount
    useEffect(() => {
        loadSettings();
    }, []);

    // Update a single setting
    const updateSetting = async (key, value) => {
        console.log(`Updating setting: ${key} = ${value}`);

        try {
            // Update in backend
            const updatedSettingsJson = await invoke('update_settings_field', {
                key,
                value
            });

            // Parse the response to get the updated settings
            if (updatedSettingsJson) {
                const updatedSettings = JSON.parse(updatedSettingsJson);
                setSettings(updatedSettings);
            } else {
                // Fallback: just update the local state
                setSettings(prev => ({
                    ...prev,
                    [key]: value,
                }));
            }

            console.log(`Setting ${key} updated successfully`);
        } catch (error) {
            console.error(`Failed to update setting ${key}:`, error);
            setError(`Failed to update ${key}: ${error.message || error}`);

            // Update local state anyway for better UX, but show the error
            setSettings(prev => ({
                ...prev,
                [key]: value,
            }));

            // Clear error after a few seconds
            setTimeout(() => setError(null), 5000);
        }
    };

    // Update multiple settings at once
    const updateMultipleSettings = async (updates) => {
        console.log('Updating multiple settings:', updates);

        try {
            // Update in backend
            const updatedSettingsJson = await invoke('update_multiple_settings_command', {
                updates
            });

            // Parse the response to get the updated settings
            if (updatedSettingsJson) {
                const updatedSettings = JSON.parse(updatedSettingsJson);
                setSettings(updatedSettings);
            } else {
                // Fallback: just update the local state
                setSettings(prev => ({
                    ...prev,
                    ...updates,
                }));
            }

            console.log('Multiple settings updated successfully');
        } catch (error) {
            console.error('Failed to update multiple settings:', error);
            setError(`Failed to update settings: ${error.message || error}`);

            // Update local state anyway for better UX, but show the error
            setSettings(prev => ({
                ...prev,
                ...updates,
            }));

            // Clear error after a few seconds
            setTimeout(() => setError(null), 5000);
        }
    };

    // Reset settings to defaults
    const resetSettings = async () => {
        console.log('Resetting settings to defaults');

        try {
            // Reset in backend
            await invoke('reset_settings');

            // Update local state
            setSettings(defaultSettings);

            console.log('Settings reset successfully');
        } catch (error) {
            console.error('Failed to reset settings:', error);
            setError(`Failed to reset settings: ${error.message || error}`);

            // Update local state anyway for better UX, but show the error
            setSettings(defaultSettings);

            // Clear error after a few seconds
            setTimeout(() => setError(null), 5000);
        }
    };

    // Reload settings from backend
    const reloadSettings = () => {
        loadSettings();
    };

    const contextValue = {
        settings,
        isLoading,
        error,
        updateSetting,
        updateMultipleSettings,
        resetSettings,
        reloadSettings,
    };

    // Show loading state if settings are not loaded yet
    if (isLoading) {
        return (
            <div style={{
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                height: '100vh',
                flexDirection: 'column',
                gap: '16px'
            }}>
                <div style={{
                    width: '40px',
                    height: '40px',
                    border: '3px solid #f3f4f6',
                    borderTopColor: '#3b82f6',
                    borderRadius: '50%',
                    animation: 'spin 1s linear infinite'
                }} />
                <div style={{
                    color: '#6b7280',
                    fontSize: '14px'
                }}>
                    Loading settings...
                </div>
                <style>{`
                    @keyframes spin {
                        to { transform: rotate(360deg); }
                    }
                `}</style>
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