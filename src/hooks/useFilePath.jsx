import { useState, useCallback, useEffect } from 'react';
import { useAppState } from '../providers/AppStateProvider';

/**
 * Hook zur Verwaltung von Dateipfaden und -verlauf
 *
 * @returns {Object} Pfadverwaltungsfunktionen und -zustand
 */
const useFilePath = () => {
    const { state, actions } = useAppState();
    const [pathHistory, setPathHistory] = useState([]);
    const [historyIndex, setHistoryIndex] = useState(-1);

    // Initialisiere den Pfadverlauf aus dem AppState, wenn er sich ändert
    useEffect(() => {
        setPathHistory(state.history || []);
        setHistoryIndex(state.historyIndex || -1);
    }, [state.history, state.historyIndex]);

    // Navigiere zu einem bestimmten Pfad
    const navigateTo = useCallback((path) => {
        if (!path) return;

        actions.setCurrentPath(path);
    }, [actions]);

    // Gehe im Verlauf zurück
    const goBack = useCallback(() => {
        actions.goBack();
    }, [actions]);

    // Gehe im Verlauf vorwärts
    const goForward = useCallback(() => {
        actions.goForward();
    }, [actions]);

    // Gehe zum übergeordneten Verzeichnis
    const goUp = useCallback(() => {
        if (!state.currentPath) return;

        // Trenne den Pfad in Segmente auf
        const isWindowsPath = state.currentPath.includes('\\');
        const separator = isWindowsPath ? '\\' : '/';
        const normalizedPath = state.currentPath.replace(/\\/g, '/');

        // Entferne das letzte Segment
        const segments = normalizedPath.split('/').filter(Boolean);

        // Wenn keine Segmente mehr übrig sind, sind wir am Stamm
        if (segments.length === 0) {
            return;
        }

        // Windows-Laufwerke behandeln (z.B. C:\)
        if (isWindowsPath && segments.length === 1 && segments[0].endsWith(':')) {
            // Bei Windows-Laufwerken gibt es kein "höheres" Verzeichnis
            return;
        }

        // Entferne das letzte Segment
        segments.pop();

        // Erstelle den neuen Pfad
        let parentPath;
        if (isWindowsPath) {
            parentPath = segments.length > 0
                ? segments.join('\\')
                : segments[0] + '\\'; // Falls es nur das Laufwerk gibt
        } else {
            parentPath = '/' + segments.join('/');
        }

        // Navigiere zum übergeordneten Verzeichnis
        navigateTo(parentPath);
    }, [state.currentPath, navigateTo]);

    // Parsieren und normalisieren eines Pfades
    const parsePath = useCallback((path) => {
        if (!path) return '';

        // Prüfe, ob es ein Windows-Pfad ist
        const isWindowsPath = path.includes('\\') || /^[A-Z]:/.test(path);

        // Normalisiere den Pfad
        let normalizedPath = path;

        if (isWindowsPath) {
            // Windows-Pfad normalisieren
            normalizedPath = path.replace(/\//g, '\\'); // Ersetze alle Schrägstriche durch Backslashes

            // Stelle sicher, dass das Laufwerk großgeschrieben ist
            if (/^[a-z]:/.test(normalizedPath)) {
                normalizedPath = normalizedPath.charAt(0).toUpperCase() + normalizedPath.slice(1);
            }
        } else {
            // Unix-Pfad normalisieren
            normalizedPath = path.replace(/\\/g, '/'); // Ersetze alle Backslashes durch Schrägstriche

            // Stelle sicher, dass der Pfad mit einem Schrägstrich beginnt
            if (!normalizedPath.startsWith('/')) {
                normalizedPath = '/' + normalizedPath;
            }
        }

        // Entferne doppelte Separatoren
        const separator = isWindowsPath ? '\\' : '/';
        normalizedPath = normalizedPath.replace(new RegExp(`${separator}+`, 'g'), separator);

        // Entferne den Separator am Ende, es sei denn, es ist ein Stammverzeichnis oder Laufwerk
        if (isWindowsPath) {
            if (normalizedPath.length > 3 && normalizedPath.endsWith('\\')) {
                normalizedPath = normalizedPath.slice(0, -1);
            }
        } else {
            if (normalizedPath.length > 1 && normalizedPath.endsWith('/')) {
                normalizedPath = normalizedPath.slice(0, -1);
            }
        }

        return normalizedPath;
    }, []);

    // Teile einen Pfad in Segmente auf
    const getPathSegments = useCallback((path) => {
        if (!path) return [];

        const isWindowsPath = path.includes('\\') || /^[A-Z]:/.test(path);
        const separator = isWindowsPath ? '\\' : '/';
        const normalizedPath = path.replace(/\\/g, '/');

        // Teile den Pfad in Segmente auf
        const segments = normalizedPath.split('/').filter(Boolean);

        // Behandle Windows-Laufwerke
        if (isWindowsPath && segments.length > 0 && segments[0].endsWith(':')) {
            segments[0] += '\\'; // Füge den Backslash zum Laufwerk hinzu
        }

        // Erstelle Pfade für jedes Segment
        const result = [];
        let currentPath = isWindowsPath ? '' : '/';

        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];

            // Aktualisiere den aktuellen Pfad
            if (i === 0 && isWindowsPath) {
                currentPath = segment;
            } else {
                currentPath += (isWindowsPath ? '\\' : '/') + segment;
            }

            // Füge das Segment zum Ergebnis hinzu
            result.push({
                name: segment,
                path: currentPath,
                isLast: i === segments.length - 1
            });
        }

        return result;
    }, []);

    // Verbinde Pfadsegmente zu einem vollständigen Pfad
    const joinPaths = useCallback((...paths) => {
        if (paths.length === 0) return '';

        // Prüfe, ob es ein Windows-Pfad ist
        const isWindowsPath = paths.some(path => path.includes('\\') || /^[A-Z]:/.test(path));
        const separator = isWindowsPath ? '\\' : '/';

        // Normalisiere alle Pfade
        const normalizedPaths = paths.map(path =>
            path.replace(/[\\/]+/g, separator).replace(new RegExp(`${separator}$`), '')
        );

        // Verbinde die Pfade
        let result = normalizedPaths[0];

        for (let i = 1; i < normalizedPaths.length; i++) {
            const path = normalizedPaths[i];

            if (!path) continue;

            // Wenn der Pfad absolut ist, ersetze den bisherigen Pfad
            if (path.startsWith(separator) || /^[A-Z]:/.test(path)) {
                result = path;
            } else {
                // Andernfalls füge den Pfad an
                result += separator + path;
            }
        }

        return result;
    }, []);

    return {
        currentPath: state.currentPath,
        pathHistory,
        historyIndex,
        canGoBack: historyIndex > 0,
        canGoForward: historyIndex < pathHistory.length - 1,
        navigateTo,
        goBack,
        goForward,
        goUp,
        parsePath,
        getPathSegments,
        joinPaths
    };
};

export default useFilePath;