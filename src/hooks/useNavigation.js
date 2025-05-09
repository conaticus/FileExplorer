import { useCallback } from 'react';
import { useHistory } from '../providers/HistoryProvider';
import { useFileSystem } from '../providers/FileSystemProvider';

/**
 * Hook for handling navigation within the file explorer.
 * @returns {Object} Navigation state and functions.
 */
const useNavigation = () => {
    const {
        currentPath,
        canGoBack,
        canGoForward,
        goBack,
        goForward,
        navigateTo
    } = useHistory();

    const { loadDirectory } = useFileSystem();

    // Navigate to a specific path
    const navigateToPath = useCallback((path) => {
        if (path === currentPath) return;

        loadDirectory(path)
            .then(() => navigateTo(path))
            .catch(error => {
                console.error(`Failed to navigate to path: ${path}`, error);
            });
    }, [currentPath, loadDirectory, navigateTo]);

    // Navigate to parent directory
    const navigateToParent = useCallback(() => {
        if (!currentPath) return;

        // For Windows paths (e.g., C:\path\to\folder)
        if (currentPath.includes(':\\')) {
            // If at root of drive (e.g., C:\), don't navigate up
            if (currentPath.match(/^[A-Z]:\\$/)) {
                return;
            }

            const parentPath = currentPath.substring(0, currentPath.lastIndexOf('\\'));
            // If at drive letter, add trailing slash (e.g., C:\ instead of C:)
            const fixedParentPath = parentPath.match(/^[A-Z]:$/)
                ? `${parentPath}\\`
                : parentPath;

            navigateToPath(fixedParentPath);
        }
        // For Unix paths (e.g., /path/to/folder)
        else {
            // If at root directory, don't navigate up
            if (currentPath === '/') {
                return;
            }

            const parentPath = currentPath.substring(0, currentPath.lastIndexOf('/'));
            // If empty string, we were in a first-level directory, navigate to root
            const fixedParentPath = parentPath || '/';

            navigateToPath(fixedParentPath);
        }
    }, [currentPath, navigateToPath]);

    // Navigate to home directory
    const navigateToHome = useCallback(() => {
        // This would typically depend on the OS
        // For demo purposes, we'll use a placeholder
        const homePath = '/home/user';
        navigateToPath(homePath);
    }, [navigateToPath]);

    // Refresh current directory
    const refreshCurrentDirectory = useCallback(() => {
        if (currentPath) {
            loadDirectory(currentPath)
                .catch(error => {
                    console.error(`Failed to refresh directory: ${currentPath}`, error);
                });
        }
    }, [currentPath, loadDirectory]);

    // Parse a path string to get segment objects
    const parsePathSegments = useCallback((path) => {
        if (!path) return [];

        const segments = [];
        let currentSegment = '';

        // Handle Windows paths
        if (path.includes(':\\')) {
            const parts = path.split('\\');

            // Add the drive letter (e.g., C:)
            segments.push({
                name: parts[0],
                path: parts[0] + '\\',
                type: 'drive',
            });

            // Add the rest of the path
            for (let i = 1; i < parts.length; i++) {
                if (!parts[i]) continue;

                currentSegment += '\\' + parts[i];
                segments.push({
                    name: parts[i],
                    path: parts[0] + currentSegment,
                    type: 'folder',
                });
            }
        }
        // Handle Unix paths
        else {
            const parts = path.split('/');

            // Handle root directory
            if (parts[0] === '') {
                segments.push({
                    name: '/',
                    path: '/',
                    type: 'root',
                });
                parts.shift(); // Remove empty string from beginning
            }

            // Add the rest of the path
            for (let i = 0; i < parts.length; i++) {
                if (!parts[i]) continue;

                currentSegment += '/' + parts[i];
                segments.push({
                    name: parts[i],
                    path: currentSegment,
                    type: 'folder',
                });
            }
        }

        return segments;
    }, []);

    return {
        currentPath,
        canGoBack,
        canGoForward,
        goBack,
        goForward,
        navigateToPath,
        navigateToParent,
        navigateToHome,
        refreshCurrentDirectory,
        parsePathSegments,
    };
};

export default useNavigation;