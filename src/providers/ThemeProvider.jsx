import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Create context
const ThemeContext = createContext({
    theme: 'light',
    toggleTheme: () => {},
    setTheme: () => {},
});

// Theme provider component
export default function ThemeProvider({ children }) {
    const [theme, setThemeState] = useState('light');
    const [isLoading, setIsLoading] = useState(true);

    // Function to apply theme to DOM
    const applyThemeToDOM = (newTheme) => {
        // Update DOM class
        document.documentElement.classList.remove(`theme-${theme}`);
        document.documentElement.classList.add(`theme-${newTheme}`);

        // Update CSS variables
        if (newTheme === 'dark') {
            document.body.style.setProperty('--background', 'var(--dark-background)');
            document.body.style.setProperty('--background-secondary', 'var(--dark-background-secondary)');
            document.body.style.setProperty('--background-tertiary', 'var(--dark-background-tertiary)');
            document.body.style.setProperty('--surface', 'var(--dark-surface)');
            document.body.style.setProperty('--surface-hover', 'var(--dark-surface-hover)');
            document.body.style.setProperty('--surface-active', 'var(--dark-surface-active)');
            document.body.style.setProperty('--border', 'var(--dark-border)');
            document.body.style.setProperty('--text-primary', 'var(--dark-text-primary)');
            document.body.style.setProperty('--text-secondary', 'var(--dark-text-secondary)');
            document.body.style.setProperty('--text-tertiary', 'var(--dark-text-tertiary)');
            document.body.style.setProperty('--accent', 'var(--dark-accent)');
            document.body.style.setProperty('--accent-hover', 'var(--dark-accent-hover)');
            document.body.style.setProperty('--accent-surface', 'var(--dark-accent-surface)');
            document.body.style.setProperty('--error', 'var(--dark-error)');
            document.body.style.setProperty('--success', 'var(--dark-success)');
            document.body.style.setProperty('--warning', 'var(--dark-warning)');
            document.body.style.setProperty('--info', 'var(--dark-info)');
        } else {
            document.body.style.setProperty('--background', 'var(--light-background)');
            document.body.style.setProperty('--background-secondary', 'var(--light-background-secondary)');
            document.body.style.setProperty('--background-tertiary', 'var(--light-background-tertiary)');
            document.body.style.setProperty('--surface', 'var(--light-surface)');
            document.body.style.setProperty('--surface-hover', 'var(--light-surface-hover)');
            document.body.style.setProperty('--surface-active', 'var(--light-surface-active)');
            document.body.style.setProperty('--border', 'var(--light-border)');
            document.body.style.setProperty('--text-primary', 'var(--light-text-primary)');
            document.body.style.setProperty('--text-secondary', 'var(--light-text-secondary)');
            document.body.style.setProperty('--text-tertiary', 'var(--light-text-tertiary)');
            document.body.style.setProperty('--accent', 'var(--light-accent)');
            document.body.style.setProperty('--accent-hover', 'var(--light-accent-hover)');
            document.body.style.setProperty('--accent-surface', 'var(--light-accent-surface)');
            document.body.style.setProperty('--error', 'var(--light-error)');
            document.body.style.setProperty('--success', 'var(--light-success)');
            document.body.style.setProperty('--warning', 'var(--light-warning)');
            document.body.style.setProperty('--info', 'var(--light-info)');
        }
    };

    // Function to set theme and update settings
    const setTheme = async (newTheme) => {
        if (newTheme === theme) return;

        console.log(`Setting theme to: ${newTheme}`);

        // Apply theme to DOM immediately for better UX
        applyThemeToDOM(newTheme);

        // Update state
        setThemeState(newTheme);

        try {
            // Save to settings (this will be handled by SettingsProvider)
            await invoke('update_settings_field', { key: 'theme', value: newTheme });
            console.log('Theme saved to settings successfully');
        } catch (error) {
            console.error('Failed to save theme setting:', error);
            // Don't revert the theme change since it's already applied and might work
        }
    };

    // Toggle between light and dark theme
    const toggleTheme = () => {
        const newTheme = theme === 'light' ? 'dark' : 'light';
        setTheme(newTheme);
    };

    // Load theme from settings on mount
    useEffect(() => {
        const loadTheme = async () => {
            setIsLoading(true);

            try {
                // Try to get theme from settings
                console.log('Loading theme from settings...');
                const savedTheme = await invoke('get_setting_field', { key: 'theme' });

                if (savedTheme) {
                    console.log(`Loaded theme from settings: ${savedTheme}`);
                    setThemeState(savedTheme);
                    applyThemeToDOM(savedTheme);
                } else {
                    console.log('No saved theme found, checking system preference');
                    // Try to match system preference
                    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
                        console.log('System prefers dark theme');
                        setThemeState('dark');
                        applyThemeToDOM('dark');
                    } else {
                        console.log('Using default light theme');
                        setThemeState('light');
                        applyThemeToDOM('light');
                    }
                }
            } catch (error) {
                console.warn('Could not load theme from settings:', error);

                // Fallback to system preference
                if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
                    setThemeState('dark');
                    applyThemeToDOM('dark');
                } else {
                    setThemeState('light');
                    applyThemeToDOM('light');
                }
            } finally {
                setIsLoading(false);
            }
        };

        loadTheme();

        // Listen for system theme changes only if no saved theme
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

        const handleSystemThemeChange = async (e) => {
            try {
                // Only auto-update if user hasn't explicitly set a theme
                const savedTheme = await invoke('get_setting_field', { key: 'theme' });
                if (!savedTheme || savedTheme === 'system') {
                    const newTheme = e.matches ? 'dark' : 'light';
                    setThemeState(newTheme);
                    applyThemeToDOM(newTheme);
                }
            } catch (error) {
                // If we can't check saved theme, just update based on system
                const newTheme = e.matches ? 'dark' : 'light';
                setThemeState(newTheme);
                applyThemeToDOM(newTheme);
            }
        };

        mediaQuery.addEventListener('change', handleSystemThemeChange);

        return () => {
            mediaQuery.removeEventListener('change', handleSystemThemeChange);
        };
    }, []);

    // Don't render children until theme is loaded
    if (isLoading) {
        return (
            <div className="theme-loading" style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                height: '100vh',
                width: '100vw',
                backgroundColor: '#ffffff',
                flexDirection: 'column',
                gap: '16px'
            }}>
                {/* Simple loading spinner */}
                <div
                    style={{
                        width: '48px',
                        height: '48px',
                        border: '5px solid #f3f4f6',
                        borderTopColor: '#3b82f6',
                        borderRadius: '50%',
                        animation: 'spin 1s linear infinite',
                    }}
                />
                <div style={{
                    color: '#6b7280',
                    fontSize: '14px'
                }}>
                    Loading theme...
                </div>
                <style jsx global>{`
                    @keyframes spin {
                        to { transform: rotate(360deg); }
                    }
                `}</style>
            </div>
        );
    }

    return (
        <ThemeContext.Provider value={{ theme, toggleTheme, setTheme }}>
            {children}
        </ThemeContext.Provider>
    );
}

// Custom hook for using the theme context
export const useTheme = () => useContext(ThemeContext);