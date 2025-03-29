import { useCallback } from 'react';
import { useAppState } from '../providers/AppStateProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import useFilePath from './useFilePath';

/**
 * Hook für die Navigation im Dateisystem
 *
 * @returns {Object} Navigationsfunktionen und -zustand
 */
const useNavigation = () => {
    const { state, actions } = useAppState();
    const { listDirectory } = useFileSystem();
    const { navigateTo, goBack, goForward, goUp, canGoBack, canGoForward } = useFilePath();

    // Lade ein Verzeichnis
    const loadDirectory = useCallback(async (path) => {
        if (!path) return [];

        try {
            actions.setLoading(true);
            const items = await listDirectory(path);
            actions.setLoading(false);
            return items;
        } catch (error) {
            console.error(`Error loading directory ${path}:`, error);
            actions.setError(`Fehler beim Laden des Verzeichnisses: ${error.message}`);
            actions.setLoading(false);
            return [];
        }
    }, [listDirectory, actions]);

    // Navigiere zu einem Verzeichnis
    const navigateToDirectory = useCallback(async (path) => {
        try {
            // Setze den aktuellen Pfad
            navigateTo(path);

            // Lade das Verzeichnis
            const items = await loadDirectory(path);
            return items;
        } catch (error) {
            console.error(`Error navigating to ${path}:`, error);
            actions.setError(`Fehler beim Navigieren zu ${path}: ${error.message}`);
            return [];
        }
    }, [navigateTo, loadDirectory, actions]);

    // Öffne ein Element (Datei oder Verzeichnis)
    const openItem = useCallback(async (item) => {
        if (!item) return;

        try {
            if (item.type === 'directory') {
                // Wenn es ein Verzeichnis ist, navigiere dorthin
                return await navigateToDirectory(item.path);
            } else {
                // Wenn es eine Datei ist, öffne sie mit der Standardanwendung
                // [Backend Integration] - Datei mit Standardanwendung öffnen
                // /* BACKEND_INTEGRATION: Datei öffnen */
                console.log(`Opening file: ${item.path}`);

                // Füge die Datei zu den letzten Pfaden hinzu
                actions.addRecent(item.path);

                return true;
            }
        } catch (error) {
            console.error(`Error opening item ${item.path}:`, error);
            actions.setError(`Fehler beim Öffnen von ${item.name}: ${error.message}`);
            return false;
        }
    }, [navigateToDirectory, actions]);

    // Navigiere zum Stammverzeichnis (Home)
    const navigateToHome = useCallback(async () => {
        try {
            // [Backend Integration] - Stammverzeichnis vom Backend abrufen
            // /* BACKEND_INTEGRATION: Stammverzeichnis abrufen */

            // Beispiel für Windows
            const homePath = 'C:\\Users\\User';

            return await navigateToDirectory(homePath);
        } catch (error) {
            console.error('Error navigating to home:', error);
            actions.setError(`Fehler beim Navigieren zum Stammverzeichnis: ${error.message}`);
            return [];
        }
    }, [navigateToDirectory, actions]);

    // Navigiere zu "Dieser PC" (alle Laufwerke)
    const navigateToThisPC = useCallback(async () => {
        try {
            // [Backend Integration] - "Dieser PC" vom Backend abrufen
            // /* BACKEND_INTEGRATION: "Dieser PC" abrufen */

            // In einer echten Implementierung würde hier eine spezielle Ansicht
            // für "Dieser PC" oder alle Laufwerke angezeigt werden
            console.log('Navigating to This PC');

            // Als Beispiel setzen wir einen speziellen Pfad
            const thisPC = 'this-pc://';
            navigateTo(thisPC);

            return await loadDirectory(thisPC);
        } catch (error) {
            console.error('Error navigating to This PC:', error);
            actions.setError(`Fehler beim Navigieren zu "Dieser PC": ${error.message}`);
            return [];
        }
    }, [navigateTo, loadDirectory, actions]);

    // Navigiere zum übergeordneten Verzeichnis
    const navigateUp = useCallback(async () => {
        try {
            goUp();
            return true;
        } catch (error) {
            console.error('Error navigating up:', error);
            actions.setError(`Fehler beim Navigieren zum übergeordneten Verzeichnis: ${error.message}`);
            return false;
        }
    }, [goUp, actions]);

    // Aktualisiere das aktuelle Verzeichnis
    const refreshCurrentDirectory = useCallback(async () => {
        try {
            return await loadDirectory(state.currentPath);
        } catch (error) {
            console.error('Error refreshing directory:', error);
            actions.setError(`Fehler beim Aktualisieren des Verzeichnisses: ${error.message}`);
            return [];
        }
    }, [loadDirectory, state.currentPath, actions]);

    return {
        currentPath: state.currentPath,
        isLoading: state.isLoading,
        error: state.error,
        canGoBack,
        canGoForward,
        navigateTo,
        navigateToDirectory,
        openItem,
        navigateToHome,
        navigateToThisPC,
        goBack,
        goForward,
        goUp,
        refreshDirectory: refreshCurrentDirectory,
        loadDirectory
    };
};

export default useNavigation;