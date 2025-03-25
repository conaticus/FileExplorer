import { useState, useCallback } from 'react';
import { useFileSystem } from '../providers/FileSystemProvider';
import { Modal, createConfirmDialog } from '../components/common';

/**
 * Hook für Datei- und Ordneroperationen
 *
 * @param {Function} [onSuccess] - Callback nach erfolgreicher Operation
 * @param {Function} [onError] - Callback bei Fehler
 * @returns {Object} Dateioperationen und Status
 */
const useFileOperations = (onSuccess, onError) => {
    const {
        createFile,
        createDirectory,
        deleteItem,
        renameItem,
        copyItem,
        moveItem,
        getItemProperties
    } = useFileSystem();

    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);

    // Neue Datei erstellen
    const handleCreateFile = useCallback(async (path, name, content = '') => {
        setIsLoading(true);
        setError(null);

        try {
            const result = await createFile(`${path}/${name}`, content);

            if (result.success) {
                if (onSuccess) onSuccess({ type: 'create-file', path: result.path });
                return result;
            } else {
                const errorMsg = `Fehler beim Erstellen der Datei: ${result.error || 'Unbekannter Fehler'}`;
                setError(errorMsg);
                if (onError) onError(errorMsg);
                return result;
            }
        } catch (err) {
            const errorMsg = `Fehler beim Erstellen der Datei: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [createFile, onSuccess, onError]);

    // Neuer Ordner erstellen
    const handleCreateDirectory = useCallback(async (path, name) => {
        setIsLoading(true);
        setError(null);

        try {
            const result = await createDirectory(`${path}/${name}`);

            if (result.success) {
                if (onSuccess) onSuccess({ type: 'create-directory', path: result.path });
                return result;
            } else {
                const errorMsg = `Fehler beim Erstellen des Ordners: ${result.error || 'Unbekannter Fehler'}`;
                setError(errorMsg);
                if (onError) onError(errorMsg);
                return result;
            }
        } catch (err) {
            const errorMsg = `Fehler beim Erstellen des Ordners: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [createDirectory, onSuccess, onError]);

    // Element löschen
    const handleDeleteItem = useCallback(async (path, skipConfirm = false) => {
        if (!skipConfirm) {
            // Zeige einen Bestätigungsdialog an
            createConfirmDialog({
                title: 'Element löschen',
                message: `Möchtest du "${path.split('/').pop()}" wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden.`,
                confirmText: 'Löschen',
                cancelText: 'Abbrechen',
                confirmVariant: 'danger',
                onConfirm: () => handleDeleteItem(path, true),
            });
            return;
        }

        setIsLoading(true);
        setError(null);

        try {
            const result = await deleteItem(path);

            if (result.success) {
                if (onSuccess) onSuccess({ type: 'delete', path });
                return result;
            } else {
                const errorMsg = `Fehler beim Löschen: ${result.error || 'Unbekannter Fehler'}`;
                setError(errorMsg);
                if (onError) onError(errorMsg);
                return result;
            }
        } catch (err) {
            const errorMsg = `Fehler beim Löschen: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [deleteItem, onSuccess, onError]);

    // Element umbenennen
    const handleRenameItem = useCallback(async (path, newName) => {
        setIsLoading(true);
        setError(null);

        try {
            const result = await renameItem(path, newName);

            if (result.success) {
                if (onSuccess) onSuccess({ type: 'rename', oldPath: path, newPath: result.newPath });
                return result;
            } else {
                const errorMsg = `Fehler beim Umbenennen: ${result.error || 'Unbekannter Fehler'}`;
                setError(errorMsg);
                if (onError) onError(errorMsg);
                return result;
            }
        } catch (err) {
            const errorMsg = `Fehler beim Umbenennen: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [renameItem, onSuccess, onError]);

    // Element kopieren
    const handleCopyItem = useCallback(async (sourcePath, destinationPath) => {
        setIsLoading(true);
        setError(null);

        try {
            const result = await copyItem(sourcePath, destinationPath);

            if (result.success) {
                if (onSuccess) onSuccess({ type: 'copy', sourcePath, destinationPath });
                return result;
            } else {
                const errorMsg = `Fehler beim Kopieren: ${result.error || 'Unbekannter Fehler'}`;
                setError(errorMsg);
                if (onError) onError(errorMsg);
                return result;
            }
        } catch (err) {
            const errorMsg = `Fehler beim Kopieren: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [copyItem, onSuccess, onError]);

    // Element verschieben
    const handleMoveItem = useCallback(async (sourcePath, destinationPath) => {
        setIsLoading(true);
        setError(null);

        try {
            const result = await moveItem(sourcePath, destinationPath);

            if (result.success) {
                if (onSuccess) onSuccess({ type: 'move', sourcePath, destinationPath });
                return result;
            } else {
                const errorMsg = `Fehler beim Verschieben: ${result.error || 'Unbekannter Fehler'}`;
                setError(errorMsg);
                if (onError) onError(errorMsg);
                return result;
            }
        } catch (err) {
            const errorMsg = `Fehler beim Verschieben: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [moveItem, onSuccess, onError]);

    // Eigenschaften eines Elements abrufen
    const handleGetProperties = useCallback(async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            const properties = await getItemProperties(path);
            return properties;
        } catch (err) {
            const errorMsg = `Fehler beim Abrufen der Eigenschaften: ${err.message}`;
            setError(errorMsg);
            if (onError) onError(errorMsg);
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, [getItemProperties, onError]);

    return {
        createFile: handleCreateFile,
        createDirectory: handleCreateDirectory,
        deleteItem: handleDeleteItem,
        renameItem: handleRenameItem,
        copyItem: handleCopyItem,
        moveItem: handleMoveItem,
        getProperties: handleGetProperties,
        isLoading,
        error,
        clearError: () => setError(null)
    };
};

export default useFileOperations;