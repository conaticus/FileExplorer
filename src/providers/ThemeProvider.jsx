import React, { createContext, useContext, useState, useEffect } from 'react';

// Create the context
export const ThemeContext = createContext();

// Available themes
export const themes = {
    light: 'light',
    dark: 'dark',
    midnight: 'midnight',
    sunset: 'sunset',
    nord: 'nord',
    forest: 'forest',
    ocean: 'ocean',
    neon: 'neon',
    lightGlass: 'lightGlass',
    darkGlass: 'darkGlass',
    system: 'system',
};

// Color palettes for each theme
export const themePalettes = {
    light: {
        background: '#ffffff',
        surface: '#f5f5f7',
        surfaceAlt: '#e8e8ec',
        primary: '#0078d4',
        primaryHover: '#106ebe',
        secondary: '#2b88d8',
        text: '#202020',
        textSecondary: '#505050',
        textOnPrimary: '#ffffff',
        border: '#e0e0e0',
        divider: '#d0d0d0',
        hover: 'rgba(0, 120, 212, 0.1)',
        selected: 'rgba(0, 120, 212, 0.2)',
        shadow: 'rgba(0, 0, 0, 0.1)',
        shadowDark: 'rgba(0, 0, 0, 0.2)',
        folderColor: '#ffc107',
        documentColor: '#2b579a',
        spreadsheetColor: '#217346',
        presentationColor: '#d24726',
        imageColor: '#ff9800',
        audioColor: '#8bc34a',
        videoColor: '#e91e63',
        archiveColor: '#795548',
        codeColor: '#00bcd4',
        pdfColor: '#f44336',
        jsonColor: '#4caf50',
        textFileColor: '#607d8b',
        successColor: '#107c10',
        warningColor: '#ffb900',
        errorColor: '#d83b01',
        infoColor: '#0078d4',
    },
    dark: {
        background: '#1e1e1e',
        surface: '#252526',
        surfaceAlt: '#333333',
        primary: '#3a96dd',
        primaryHover: '#2a7ab9',
        secondary: '#60cdff',
        text: '#e0e0e0',
        textSecondary: '#a0a0a0',
        textOnPrimary: '#ffffff',
        border: '#444444',
        divider: '#555555',
        hover: 'rgba(58, 150, 221, 0.15)',
        selected: 'rgba(58, 150, 221, 0.3)',
        shadow: 'rgba(0, 0, 0, 0.3)',
        shadowDark: 'rgba(0, 0, 0, 0.5)',
        folderColor: '#ffd54f',
        documentColor: '#4a88cf',
        spreadsheetColor: '#4caf50',
        presentationColor: '#ff7043',
        imageColor: '#ffb74d',
        audioColor: '#aed581',
        videoColor: '#f06292',
        archiveColor: '#a1887f',
        codeColor: '#4dd0e1',
        pdfColor: '#ef5350',
        jsonColor: '#66bb6a',
        textFileColor: '#90a4ae',
        successColor: '#4caf50',
        warningColor: '#ffc107',
        errorColor: '#f44336',
        infoColor: '#2196f3',
    },
    midnight: {
        background: '#0d1117',
        surface: '#161b22',
        surfaceAlt: '#21262d',
        primary: '#58a6ff',
        primaryHover: '#79b8ff',
        secondary: '#79b8ff',
        text: '#c9d1d9',
        textSecondary: '#8b949e',
        textOnPrimary: '#ffffff',
        border: '#30363d',
        divider: '#30363d',
        hover: 'rgba(88, 166, 255, 0.1)',
        selected: 'rgba(88, 166, 255, 0.2)',
        shadow: 'rgba(0, 0, 0, 0.4)',
        shadowDark: 'rgba(0, 0, 0, 0.6)',
        folderColor: '#ffd54f',
        documentColor: '#4a88cf',
        spreadsheetColor: '#4caf50',
        presentationColor: '#ff7043',
        imageColor: '#ffb74d',
        audioColor: '#aed581',
        videoColor: '#f06292',
        archiveColor: '#a1887f',
        codeColor: '#4dd0e1',
        pdfColor: '#ef5350',
        jsonColor: '#66bb6a',
        textFileColor: '#90a4ae',
        successColor: '#3fb950',
        warningColor: '#f0883e',
        errorColor: '#f85149',
        infoColor: '#58a6ff',
    },
    sunset: {
        background: '#2d1b2a',
        surface: '#3b2438',
        surfaceAlt: '#472a45',
        primary: '#ff8f66',
        primaryHover: '#ff7c4d',
        secondary: '#fe5e78',
        text: '#e8e0e8',
        textSecondary: '#b9a6b7',
        textOnPrimary: '#2d1b2a',
        border: '#5a384d',
        divider: '#5a384d',
        hover: 'rgba(255, 143, 102, 0.15)',
        selected: 'rgba(255, 143, 102, 0.25)',
        shadow: 'rgba(0, 0, 0, 0.4)',
        shadowDark: 'rgba(0, 0, 0, 0.6)',
        folderColor: '#fed49a',
        documentColor: '#c79fff',
        spreadsheetColor: '#7ed8a8',
        presentationColor: '#fe5e78',
        imageColor: '#ff8f66',
        audioColor: '#a4d9ff',
        videoColor: '#fe5e78',
        archiveColor: '#b8a89a',
        codeColor: '#7edeff',
        pdfColor: '#fe5e78',
        jsonColor: '#7ed8a8',
        textFileColor: '#c5e1f0',
        successColor: '#7ed8a8',
        warningColor: '#fed49a',
        errorColor: '#fe5e78',
        infoColor: '#a4d9ff',
    },
    nord: {
        background: '#2e3440',
        surface: '#3b4252',
        surfaceAlt: '#434c5e',
        primary: '#88c0d0',
        primaryHover: '#8fbcbb',
        secondary: '#81a1c1',
        text: '#eceff4',
        textSecondary: '#d8dee9',
        textOnPrimary: '#2e3440',
        border: '#4c566a',
        divider: '#4c566a',
        hover: 'rgba(136, 192, 208, 0.15)',
        selected: 'rgba(136, 192, 208, 0.25)',
        shadow: 'rgba(0, 0, 0, 0.3)',
        shadowDark: 'rgba(0, 0, 0, 0.5)',
        folderColor: '#ebcb8b',
        documentColor: '#81a1c1',
        spreadsheetColor: '#a3be8c',
        presentationColor: '#bf616a',
        imageColor: '#ebcb8b',
        audioColor: '#a3be8c',
        videoColor: '#bf616a',
        archiveColor: '#d08770',
        codeColor: '#88c0d0',
        pdfColor: '#bf616a',
        jsonColor: '#a3be8c',
        textFileColor: '#d8dee9',
        successColor: '#a3be8c',
        warningColor: '#ebcb8b',
        errorColor: '#bf616a',
        infoColor: '#81a1c1',
    },
    forest: {
        background: '#1b2921',
        surface: '#243329',
        surfaceAlt: '#2d3f36',
        primary: '#7cc082',
        primaryHover: '#8fd297',
        secondary: '#5ba989',
        text: '#e0ebe0',
        textSecondary: '#a7c0b0',
        textOnPrimary: '#1b2921',
        border: '#3a4940',
        divider: '#3a4940',
        hover: 'rgba(124, 192, 130, 0.15)',
        selected: 'rgba(124, 192, 130, 0.25)',
        shadow: 'rgba(0, 0, 0, 0.3)',
        shadowDark: 'rgba(0, 0, 0, 0.5)',
        folderColor: '#d9c872',
        documentColor: '#7caac5',
        spreadsheetColor: '#7cc082',
        presentationColor: '#c57979',
        imageColor: '#d9c872',
        audioColor: '#7cc082',
        videoColor: '#c57979',
        archiveColor: '#c59667',
        codeColor: '#7caac5',
        pdfColor: '#c57979',
        jsonColor: '#7cc082',
        textFileColor: '#b4c5bf',
        successColor: '#7cc082',
        warningColor: '#d9c872',
        errorColor: '#c57979',
        infoColor: '#7caac5',
    },
    ocean: {
        background: '#0f2231',
        surface: '#1a3040',
        surfaceAlt: '#25404f',
        primary: '#60c5fa',
        primaryHover: '#83d3fb',
        secondary: '#3dd9d6',
        text: '#e4f4ff',
        textSecondary: '#a9c7d7',
        textOnPrimary: '#0f2231',
        border: '#2c4a5e',
        divider: '#2c4a5e',
        hover: 'rgba(96, 197, 250, 0.15)',
        selected: 'rgba(96, 197, 250, 0.25)',
        shadow: 'rgba(0, 0, 0, 0.3)',
        shadowDark: 'rgba(0, 0, 0, 0.5)',
        folderColor: '#ffd580',
        documentColor: '#60c5fa',
        spreadsheetColor: '#3dd9d6',
        presentationColor: '#fa607a',
        imageColor: '#ffd580',
        audioColor: '#3dd9d6',
        videoColor: '#fa607a',
        archiveColor: '#faa360',
        codeColor: '#60c5fa',
        pdfColor: '#fa607a',
        jsonColor: '#3dd9d6',
        textFileColor: '#a9c7d7',
        successColor: '#3dd9d6',
        warningColor: '#ffd580',
        errorColor: '#fa607a',
        infoColor: '#60c5fa',
    },
    neon: {
        background: '#030014',
        surface: '#0c0628',
        surfaceAlt: '#14093f',
        primary: '#fa55d6',
        primaryHover: '#fb83e1',
        secondary: '#c676ff',
        text: '#f0f0ff',
        textSecondary: '#b0a2d5',
        textOnPrimary: '#030014',
        border: '#2c0d75',
        divider: '#2c0d75',
        hover: 'rgba(250, 85, 214, 0.2)',
        selected: 'rgba(250, 85, 214, 0.3)',
        shadow: 'rgba(250, 85, 214, 0.3)',
        shadowDark: 'rgba(250, 85, 214, 0.5)',
        folderColor: '#faea5d',
        documentColor: '#54ccff',
        spreadsheetColor: '#00ff95',
        presentationColor: '#ff5566',
        imageColor: '#faea5d',
        audioColor: '#00ff95',
        videoColor: '#ff5566',
        archiveColor: '#ff8d42',
        codeColor: '#54ccff',
        pdfColor: '#ff5566',
        jsonColor: '#00ff95',
        textFileColor: '#b0a2d5',
        successColor: '#00ff95',
        warningColor: '#faea5d',
        errorColor: '#ff5566',
        infoColor: '#54ccff',
    },
    lightGlass: {
        background: 'rgba(255, 255, 255, 0.85)',
        surface: 'rgba(245, 245, 247, 0.8)',
        surfaceAlt: 'rgba(232, 232, 236, 0.75)',
        primary: '#0078d4',
        primaryHover: '#106ebe',
        secondary: '#2b88d8',
        text: '#202020',
        textSecondary: '#505050',
        textOnPrimary: '#ffffff',
        border: 'rgba(224, 224, 224, 0.5)',
        divider: 'rgba(208, 208, 208, 0.5)',
        hover: 'rgba(0, 120, 212, 0.1)',
        selected: 'rgba(0, 120, 212, 0.2)',
        shadow: 'rgba(0, 0, 0, 0.1)',
        shadowDark: 'rgba(0, 0, 0, 0.2)',
        folderColor: '#ffc107',
        documentColor: '#2b579a',
        spreadsheetColor: '#217346',
        presentationColor: '#d24726',
        imageColor: '#ff9800',
        audioColor: '#8bc34a',
        videoColor: '#e91e63',
        archiveColor: '#795548',
        codeColor: '#00bcd4',
        pdfColor: '#f44336',
        jsonColor: '#4caf50',
        textFileColor: '#607d8b',
        successColor: '#107c10',
        warningColor: '#ffb900',
        errorColor: '#d83b01',
        infoColor: '#0078d4',
        glassEnabled: true,
    },
    darkGlass: {
        background: 'rgba(30, 30, 30, 0.75)',
        surface: 'rgba(37, 37, 38, 0.75)',
        surfaceAlt: 'rgba(51, 51, 51, 0.7)',
        primary: '#3a96dd',
        primaryHover: '#2a7ab9',
        secondary: '#60cdff',
        text: '#e0e0e0',
        textSecondary: '#a0a0a0',
        textOnPrimary: '#ffffff',
        border: 'rgba(68, 68, 68, 0.5)',
        divider: 'rgba(85, 85, 85, 0.5)',
        hover: 'rgba(58, 150, 221, 0.15)',
        selected: 'rgba(58, 150, 221, 0.3)',
        shadow: 'rgba(0, 0, 0, 0.3)',
        shadowDark: 'rgba(0, 0, 0, 0.5)',
        folderColor: '#ffd54f',
        documentColor: '#4a88cf',
        spreadsheetColor: '#4caf50',
        presentationColor: '#ff7043',
        imageColor: '#ffb74d',
        audioColor: '#aed581',
        videoColor: '#f06292',
        archiveColor: '#a1887f',
        codeColor: '#4dd0e1',
        pdfColor: '#ef5350',
        jsonColor: '#66bb6a',
        textFileColor: '#90a4ae',
        successColor: '#4caf50',
        warningColor: '#ffc107',
        errorColor: '#f44336',
        infoColor: '#2196f3',
        glassEnabled: true,
    },
};

// Default theme settings
export const defaultThemeSettings = {
    fontSize: 'medium', // small, medium, large
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif',
    defaultView: 'list', // list, grid, details
    iconSize: 'medium', // small, medium, large
    accentColor: '#0078d4', // Can be overridden by user
    showHiddenFiles: false,
    enableGlassEffect: false,
    enableAnimations: true,
    density: 'normal', // compact, normal, comfortable
    borderRadius: 'medium', // none, small, medium, large
    confirmDeletion: true,
    showThumbnails: true,
};

// Border radius values for each setting
const borderRadiusValues = {
    none: {
        sm: '0',
        md: '0',
        lg: '0',
    },
    small: {
        sm: '2px',
        md: '4px',
        lg: '6px',
    },
    medium: {
        sm: '4px',
        md: '6px',
        lg: '8px',
    },
    large: {
        sm: '6px',
        md: '10px',
        lg: '16px',
    },
};

export const ThemeProvider = ({ children }) => {
    // Theme state: "light", "dark", "system", etc.
    const [theme, setTheme] = useState(() => {
        // Get saved theme from localStorage or use default
        const savedTheme = localStorage.getItem('theme') || themes.system;
        return savedTheme;
    });

    // Theme settings
    const [themeSettings, setThemeSettings] = useState(() => {
        try {
            // Get saved settings from localStorage or use defaults
            const savedSettings = JSON.parse(localStorage.getItem('themeSettings'));
            return savedSettings || defaultThemeSettings;
        } catch (error) {
            console.error('Error parsing theme settings:', error);
            return defaultThemeSettings;
        }
    });

    // Active theme based on theme state and system preference
    const [activeTheme, setActiveTheme] = useState(theme === themes.system
        ? window.matchMedia('(prefers-color-scheme: dark)').matches ? themes.dark : themes.light
        : theme);

    // Apply glass effect to active theme if enabled
    const [processedTheme, setProcessedTheme] = useState(() => {
        const baseTheme = activeTheme;

        // If glass effect is enabled and the base theme doesn't already have glass
        if (themeSettings.enableGlassEffect && !baseTheme.includes('Glass')) {
            if (baseTheme === themes.light || baseTheme === themes.system) {
                return themes.lightGlass;
            } else if (baseTheme === themes.dark) {
                return themes.darkGlass;
            }
        }

        return baseTheme;
    });

    // Update active theme when theme changes
    useEffect(() => {
        if (theme === themes.system) {
            // Check system preference
            const isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
            setActiveTheme(isDarkMode ? themes.dark : themes.light);

            // Listen for system theme changes
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

    // Update processed theme when active theme or glass settings change
    useEffect(() => {
        let newTheme = activeTheme;

        // If glass effect is enabled and the active theme isn't already a glass theme
        if (themeSettings.enableGlassEffect && !activeTheme.includes('Glass')) {
            if (activeTheme === themes.light) {
                newTheme = themes.lightGlass;
            } else if (activeTheme === themes.dark || activeTheme === themes.midnight) {
                newTheme = themes.darkGlass;
            }
        } else if (!themeSettings.enableGlassEffect && activeTheme.includes('Glass')) {
            // If glass effect is disabled but we're using a glass theme
            newTheme = activeTheme === themes.lightGlass ? themes.light : themes.dark;
        }

        setProcessedTheme(newTheme);
    }, [activeTheme, themeSettings.enableGlassEffect]);

    // Save theme to localStorage when it changes
    useEffect(() => {
        localStorage.setItem('theme', theme);
    }, [theme]);

    // Save theme settings to localStorage when they change
    useEffect(() => {
        localStorage.setItem('themeSettings', JSON.stringify(themeSettings));
    }, [themeSettings]);

    // Apply theme to the HTML element
    useEffect(() => {
        // Set theme attribute
        document.documentElement.setAttribute('data-theme', processedTheme);

        // Set font size
        document.documentElement.setAttribute('data-font-size', themeSettings.fontSize);

        // Set icon size
        document.documentElement.setAttribute('data-icon-size', themeSettings.iconSize);

        // Set UI density
        document.documentElement.setAttribute('data-density', themeSettings.density);

        // Apply border radius values
        const radiusSettings = themeSettings.borderRadius || 'medium';
        const radiusValues = borderRadiusValues[radiusSettings];
        document.documentElement.style.setProperty('--border-radius-sm', radiusValues.sm);
        document.documentElement.style.setProperty('--border-radius-md', radiusValues.md);
        document.documentElement.style.setProperty('--border-radius-lg', radiusValues.lg);

        // Apply accent color
        document.documentElement.style.setProperty('--color-primary', themeSettings.accentColor);

        // Apply font family
        document.documentElement.style.setProperty('--font-family', themeSettings.fontFamily);

        // Apply glass effect variables
        const palette = themePalettes[processedTheme];
        if (palette && palette.glassEnabled) {
            // Extract RGB components from background color (assuming rgba format)
            let bgColorParts = palette.background.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*[\d.]+)?\)/);

            // If not in rgba format, use default values
            const bgColorRGB = bgColorParts
                ? `${bgColorParts[1]}, ${bgColorParts[2]}, ${bgColorParts[3]}`
                : processedTheme.includes('light') ? '255, 255, 255' : '30, 30, 30';

            document.documentElement.style.setProperty('--glass-bg-color', bgColorRGB);
            document.documentElement.style.setProperty('--glass-opacity', '0.75');
            document.documentElement.style.setProperty('--glass-blur', '12px');
            document.documentElement.style.setProperty('--glass-border-color', '255, 255, 255');
            document.documentElement.style.setProperty('--glass-border-opacity', processedTheme.includes('light') ? '0.4' : '0.1');
            document.documentElement.style.setProperty('--glass-shadow-color', '0, 0, 0');
            document.documentElement.style.setProperty('--glass-shadow-opacity', processedTheme.includes('light') ? '0.1' : '0.3');

            // Add a class to the body for global glass effect styles
            document.body.classList.add('glass-theme');
        } else {
            document.body.classList.remove('glass-theme');
        }

        // Apply animations setting
        if (themeSettings.enableAnimations) {
            document.body.classList.remove('no-animations');
        } else {
            document.body.classList.add('no-animations');
        }
    }, [processedTheme, themeSettings]);

    // Toggle dark/light theme
    const toggleTheme = () => {
        setTheme(prev => prev === themes.light ? themes.dark : themes.light);
    };

    // Set a specific theme
    const setSpecificTheme = (newTheme) => {
        if (Object.values(themes).includes(newTheme)) {
            setTheme(newTheme);
        }
    };

    // Update theme settings
    const updateThemeSettings = (newSettings) => {
        setThemeSettings(prev => ({ ...prev, ...newSettings }));
    };

    // Get current theme colors
    const colors = themePalettes[processedTheme] || themePalettes[activeTheme] || themePalettes.light;

    // Context value
    const contextValue = {
        theme,
        themes,
        activeTheme,
        colors,
        themeSettings,
        themePalettes,
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

// Custom Hook for easy access to the theme context
export const useTheme = () => {
    const context = useContext(ThemeContext);
    if (context === undefined) {
        throw new Error('useTheme must be used within a ThemeProvider');
    }
    return context;
};

export default ThemeProvider;