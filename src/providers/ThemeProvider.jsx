import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';

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

    // Function to set theme and update settings
    const setTheme = async (newTheme) => {
        if (newTheme === theme) return;

        // Update DOM
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

        // Update state
        setThemeState(newTheme);

        try {
            // Save to settings
            await invoke('update_settings_field', { key: 'theme', value: newTheme });
        } catch (error) {
            console.error('Failed to save theme setting:', error);
        }
    };

    // Toggle between light and dark theme
    const toggleTheme = () => {
        setTheme(theme === 'light' ? 'dark' : 'light');
    };

    // Load theme from settings on mount
    useEffect(() => {
        const loadTheme = async () => {
            try {
                // Try to get theme from settings
                const savedTheme = await invoke('get_setting_field', { key: 'theme' });
                setTheme(savedTheme || 'light');
            } catch (error) {
                console.warn('Could not load theme from settings:', error);

                // Try to match system preference
                if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
                    setTheme('dark');
                } else {
                    setTheme('light');
                }
            } finally {
                setIsLoading(false);
            }
        };

        loadTheme();

        // Listen for system theme changes
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

        const handleChange = (e) => {
            setTheme(e.matches ? 'dark' : 'light');
        };

        mediaQuery.addEventListener('change', handleChange);

        return () => {
            mediaQuery.removeEventListener('change', handleChange);
        };
    }, []);

    // Don't render children until theme is loaded
    if (isLoading) {
        return (
            <div className="theme-loading flex-center" style={{ height: '100vh', width: '100vw' }}>
                {/* Simple loading spinner */}
                <div
                    style={{
                        width: '48px',
                        height: '48px',
                        border: '5px solid var(--light-background-tertiary)',
                        borderTopColor: 'var(--light-accent)',
                        borderRadius: '50%',
                        animation: 'spin 1s linear infinite',
                    }}
                />
            </div>
        );
    }

    return (
        <ThemeContext.Provider value={{ theme, toggleTheme, setTheme }}>
            <style jsx global>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
            {children}
        </ThemeContext.Provider>
    );
}

// Custom hook for using the theme context
export const useTheme = () => useContext(ThemeContext);