import { useEffect } from 'react';
import { useSettings } from '../providers/SettingsProvider';

/**
 * Component that applies settings to the DOM/CSS variables
 * This component doesn't render anything, it just applies settings as side effects
 */
const SettingsApplier = () => {
    const { settings } = useSettings();

    useEffect(() => {
        // Apply font size settings
        if (settings.font_size) {
            const fontSizeClass = `font-size-${settings.font_size.toLowerCase()}`;

            // Remove existing font size classes
            document.documentElement.classList.remove('font-size-small', 'font-size-medium', 'font-size-large');

            // Add the current font size class
            if (settings.font_size !== 'Medium') {
                document.documentElement.classList.add(fontSizeClass);
            }
        }

        // Apply accent color
        if (settings.accent_color) {
            // Convert hex to RGB for CSS variables that need RGB values
            const hexToRgb = (hex) => {
                const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
                return result ? {
                    r: parseInt(result[1], 16),
                    g: parseInt(result[2], 16),
                    b: parseInt(result[3], 16)
                } : null;
            };

            const rgb = hexToRgb(settings.accent_color);
            if (rgb) {
                // Create hover and surface variants
                const darkerAccent = `color-mix(in srgb, ${settings.accent_color} 85%, black)`;
                const lighterAccent = `color-mix(in srgb, ${settings.accent_color} 15%, transparent)`;

                // Apply accent color variables
                document.documentElement.style.setProperty('--light-accent', settings.accent_color);
                document.documentElement.style.setProperty('--dark-accent', settings.accent_color);
                document.documentElement.style.setProperty('--light-accent-hover', darkerAccent);
                document.documentElement.style.setProperty('--dark-accent-hover', darkerAccent);
                document.documentElement.style.setProperty('--light-accent-surface', lighterAccent);
                document.documentElement.style.setProperty('--dark-accent-surface', lighterAccent);

                // Update RGB values for transparency effects
                document.documentElement.style.setProperty('--accent-rgb', `${rgb.r}, ${rgb.g}, ${rgb.b}`);

                // Apply to current theme
                document.documentElement.style.setProperty('--accent', settings.accent_color);
                document.documentElement.style.setProperty('--accent-hover', darkerAccent);
                document.documentElement.style.setProperty('--accent-surface', lighterAccent);
            }
        }

        // Apply animation settings
        if (settings.enable_animations_and_transitions === false) {
            document.documentElement.classList.add('reduce-motion');
        } else {
            document.documentElement.classList.remove('reduce-motion');
        }

        // Apply terminal height
        if (settings.terminal_height) {
            document.documentElement.style.setProperty('--terminal-height', `${settings.terminal_height}px`);
        }

        console.log('Settings applied to DOM:', {
            font_size: settings.font_size,
            accent_color: settings.accent_color,
            enable_animations_and_transitions: settings.enable_animations_and_transitions,
            terminal_height: settings.terminal_height
        });

    }, [settings.font_size, settings.accent_color, settings.enable_animations_and_transitions, settings.terminal_height]);

    // This component doesn't render anything
    return null;
};

export default SettingsApplier;