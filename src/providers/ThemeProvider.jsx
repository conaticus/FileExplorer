import React, { createContext, useContext, useState, useEffect } from 'react';

// Erstellen eines Kontexts für das Theme
export const ThemeContext = createContext();

// Verfügbare Themes
export const themes = {
    light: 'light',
    dark: 'dark',
    system: 'system',
};

// Farbpaletten für die Themes
export const themeColors = {
    light: {
        background: '#ffffff',
        surface: '#f5f5f5',
        surfaceAlt: '#e8e8e8',
        primary: '#0078d4',
        primaryHover: '#106ebe',
        secondary: '#2b88d8',
        text: '#202020',
        textSecondary: '#505050',
        border: '#e0e0e0',
        divider: '#d0d0d0',
        hover: 'rgba(0, 120, 212, 0.1)',
        selected: 'rgba(0, 120, 212, 0.2)',
        shadow: 'rgba(0, 0, 0, 0.1)',
    },
    dark: {
        background: '#1e1e1e',
        surface: '#252525',
        surfaceAlt: '#333333',
        primary: '#3a96dd',
        primaryHover: '#2a7ab9',
        secondary: '#60cdff',
        text: '#e0e0e0',
        textSecondary: '#a0a0a0',
        border: '#444444',
        divider: '#555555',
        hover: 'rgba(58, 150, 221, 0.15)',
        selected: 'rgba(58, 150, 221, 0.3)',
        shadow: 'rgba(0, 0, 0, 0.3)',
    },
};

// Benutzerdefinierte Theme-Einstellungen
export const defaultThemeSettings = {
    fontSize: 'medium', // small, medium, large
    defaultView: 'list', // list, grid, details
    iconSize: 'medium', // small, medium, large
    showHiddenFiles: false,
    accentColor: '#0078d4', // Kann vom Benutzer angepasst werden
};

export const ThemeProvider = ({ children }) => {
    // Theme-Zustand: "light", "dark" oder "system"
    const [theme, setTheme] = useState(() => {
        // Gespeichertes Theme aus localStorage abrufen oder Standardwert verwenden
        const savedTheme = localStorage.getItem('theme') || themes.system;
        return savedTheme;
    });

    // Theme-Einstellungen
    const [themeSettings, setThemeSettings] = useState(() => {
        try {
            // Gespeicherte Einstellungen aus localStorage abrufen oder Standardwerte verwenden
            const savedSettings = JSON.parse(localStorage.getItem('themeSettings'));
            return savedSettings || defaultThemeSettings;
        } catch (error) {
            console.error('Error parsing theme settings:', error);
            return defaultThemeSettings;
        }
    });

    // Aktives Theme basierend auf dem Theme-Zustand und dem System-Theme
    const [activeTheme, setActiveTheme] = useState(theme === themes.system
        ? window.matchMedia('(prefers-color-scheme: dark)').matches ? themes.dark : themes.light
        : theme);

    // Aktualisiere das aktive Theme, wenn sich das ausgewählte Theme ändert
    useEffect(() => {
        if (theme === themes.system) {
            // Überprüfe das System-Theme
            const isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
            setActiveTheme(isDarkMode ? themes.dark : themes.light);

            // Listener für Änderungen des System-Themes
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            const handleChange = (e) => {
                setActiveTheme(e.matches ? themes.dark : themes.light);
            };

            mediaQuery.addEventListener('change', handleChange);
            return () => mediaQuery.removeEventListener('change', handleChange);
        } else {
            setActiveTheme(theme);
        }
    }, [theme]);

    // Speichere das Theme in localStorage, wenn es sich ändert
    useEffect(() => {
        localStorage.setItem('theme', theme);
    }, [theme]);

    // Speichere die Theme-Einstellungen in localStorage, wenn sie sich ändern
    useEffect(() => {
        localStorage.setItem('themeSettings', JSON.stringify(themeSettings));
    }, [themeSettings]);

    // Anwenden des Themes auf das HTML-Element
    useEffect(() => {
        document.documentElement.setAttribute('data-theme', activeTheme);

        // Anwenden der Schriftgröße
        document.documentElement.setAttribute('data-font-size', themeSettings.fontSize);
    }, [activeTheme, themeSettings]);

    // Theme wechseln
    const toggleTheme = () => {
        setTheme(prev => prev === themes.light ? themes.dark : themes.light);
    };

    // Spezifisches Theme festlegen
    const setSpecificTheme = (newTheme) => {
        if (Object.values(themes).includes(newTheme)) {
            setTheme(newTheme);
        }
    };

    // Theme-Einstellungen aktualisieren
    const updateThemeSettings = (newSettings) => {
        setThemeSettings(prev => ({ ...prev, ...newSettings }));
    };

    // Aktuelle Theme-Farben basierend auf dem aktiven Theme
    const colors = themeColors[activeTheme];

    // Kontextwert
    const contextValue = {
        theme,
        activeTheme,
        colors,
        themeSettings,
        setTheme: setSpecificTheme,
        toggleTheme,
        updateThemeSettings,
    };

    return (
        <ThemeContext.Provider value={contextValue}>
            {children}
        </ThemeContext.Provider>
    );
};

// Custom Hook für einfachen Zugriff auf den Theme-Kontext
export const useTheme = () => {
    const context = useContext(ThemeContext);
    if (context === undefined) {
        throw new Error('useTheme must be used within a ThemeProvider');
    }
    return context;
};

export default ThemeProvider;