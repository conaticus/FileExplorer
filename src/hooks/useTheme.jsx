import { useContext } from 'react';
import { ThemeContext } from '../providers/ThemeProvider';

/**
 * Hook für den Zugriff auf das Theme und seine Funktionen
 *
 * @returns {Object} Theme-Funktionen und -Zustand
 */
const useTheme = () => {
    const context = useContext(ThemeContext);

    if (!context) {
        throw new Error('useTheme must be used within a ThemeProvider');
    }

    return context;
};

export default useTheme;