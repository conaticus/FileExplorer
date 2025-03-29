import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";

// Erstellen eines Kontexts für den Dateisystemzugriff
export const FileSystemContext = createContext();

export const FileSystemProvider = ({ children }) => {
    const [rootFolders, setRootFolders] = useState([]);
    const [dataSources, setDataSources] = useState([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);

    // Lade die Stammordner beim Start
    useEffect(() => {
        loadRootFolders();
        loadDataSources();
    }, []);

    // Lade die Stammordner
    const loadRootFolders = async () => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Stammordner vom Backend abrufen
            // /* BACKEND_INTEGRATION: Stammordner laden */

            // Beispieldaten für Windows
            const mockRootFolders = [
                { name: 'Desktop', path: 'C:/Users/User/Desktop', type: 'directory', icon: 'desktop' },
                { name: 'Dokumente', path: 'C:/Users/User/Documents', type: 'directory', icon: 'documents' },
                { name: 'Downloads', path: 'C:/Users/User/Downloads', type: 'directory', icon: 'downloads' },
                { name: 'Musik', path: 'C:/Users/User/Music', type: 'directory', icon: 'music' },
                { name: 'Bilder', path: 'C:/Users/User/Pictures', type: 'directory', icon: 'pictures' },
                { name: 'Videos', path: 'C:/Users/User/Videos', type: 'directory', icon: 'videos' },
            ];

            setRootFolders(mockRootFolders);
        } catch (err) {
            console.error('Error loading root folders:', err);
            setError('Fehler beim Laden der Stammordner');
        } finally {
            setIsLoading(false);
        }
    };

    // Lade die Datenquellen
    const loadDataSources = async () => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Datenquellen vom Backend abrufen
            // /* BACKEND_INTEGRATION: Datenquellen laden */

            // Beispieldaten für Windows
            const mockDataSources = [
                { name: 'Lokaler Datenträger (C:)', path: 'C:/', type: 'drive', icon: 'drive', totalSpace: '256 GB', freeSpace: '120 GB' },
                { name: 'Daten (D:)', path: 'D:/', type: 'drive', icon: 'drive', totalSpace: '1 TB', freeSpace: '750 GB' },
            ];

            setDataSources(mockDataSources);
        } catch (err) {
            console.error('Error loading data sources:', err);
            setError('Fehler beim Laden der Datenquellen');
        } finally {
            setIsLoading(false);
        }
    };

    // Füge eine neue Datenquelle hinzu
    const addDataSource = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Datenquelle zum Backend hinzufügen
            // /* BACKEND_INTEGRATION: Datenquelle hinzufügen */

            // Beispieldaten
            const newDataSource = {
                name: path.split('/').pop(),
                path: path,
                type: 'custom',
                icon: 'folder',
            };

            setDataSources([...dataSources, newDataSource]);

            return { success: true, dataSource: newDataSource };
        } catch (err) {
            console.error('Error adding data source:', err);
            setError('Fehler beim Hinzufügen der Datenquelle');
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Entferne eine Datenquelle
    const removeDataSource = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Datenquelle aus dem Backend entfernen
            // /* BACKEND_INTEGRATION: Datenquelle entfernen */

            setDataSources(dataSources.filter(ds => ds.path !== path));

            return { success: true };
        } catch (err) {
            console.error('Error removing data source:', err);
            setError('Fehler beim Entfernen der Datenquelle');
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Abrufen der Elemente in einem Verzeichnis
    const listDirectory = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Verzeichnisinhalt vom Backend abrufen
            // /* BACKEND_INTEGRATION: Verzeichnisinhalt laden */

            // Beispieldaten
            const mockItems = [
                { name: 'Dokument1.docx', path: `${path}/Dokument1.docx`, type: 'file', size: '25 KB', modified: '2023-01-15T10:30:00Z' },
                { name: 'Tabelle.xlsx', path: `${path}/Tabelle.xlsx`, type: 'file', size: '156 KB', modified: '2023-02-20T14:45:00Z' },
                { name: 'Präsentation.pptx', path: `${path}/Präsentation.pptx`, type: 'file', size: '2.3 MB', modified: '2023-03-10T09:15:00Z' },
                { name: 'Ordner1', path: `${path}/Ordner1`, type: 'directory', modified: '2023-01-05T11:20:00Z' },
                { name: 'Ordner2', path: `${path}/Ordner2`, type: 'directory', modified: '2023-02-12T16:40:00Z' },
                { name: 'Bilder', path: `${path}/Bilder`, type: 'directory', modified: '2023-03-05T13:10:00Z' },
                { name: 'test.txt', path: `${path}/test.txt`, type: 'file', size: '2 KB', modified: '2023-04-01T08:00:00Z' },
                { name: 'config.json', path: `${path}/config.json`, type: 'file', size: '4 KB', modified: '2023-04-02T09:30:00Z' },
            ];

            return mockItems;
        } catch (err) {
            console.error(`Error listing directory ${path}:`, err);
            setError(`Fehler beim Laden des Verzeichnisses: ${err.message}`);
            throw err;
        } finally {
            setIsLoading(false);
        }
    };

    // Lesen einer Datei
    const readFile = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Datei vom Backend lesen
            // /* BACKEND_INTEGRATION: Datei lesen */

            // Beispielinhalt
            const content = `Dies ist der Inhalt der Datei ${path}`;

            return content;
        } catch (err) {
            console.error(`Error reading file ${path}:`, err);
            setError(`Fehler beim Lesen der Datei: ${err.message}`);
            throw err;
        } finally {
            setIsLoading(false);
        }
    };

    // Erstellen einer Datei
    const createFile = async (path, content = '') => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Datei im Backend erstellen
            // /* BACKEND_INTEGRATION: Datei erstellen */

            return { success: true, path };
        } catch (err) {
            console.error(`Error creating file ${path}:`, err);
            setError(`Fehler beim Erstellen der Datei: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Erstellen eines Ordners
    const createDirectory = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Ordner im Backend erstellen
            // /* BACKEND_INTEGRATION: Ordner erstellen */

            return { success: true, path };
        } catch (err) {
            console.error(`Error creating directory ${path}:`, err);
            setError(`Fehler beim Erstellen des Ordners: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Löschen einer Datei oder eines Ordners
    const deleteItem = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Element im Backend löschen
            // /* BACKEND_INTEGRATION: Element löschen */

            return { success: true };
        } catch (err) {
            console.error(`Error deleting item ${path}:`, err);
            setError(`Fehler beim Löschen: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Umbenennen einer Datei oder eines Ordners
    const renameItem = async (oldPath, newName) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Element im Backend umbenennen
            // /* BACKEND_INTEGRATION: Element umbenennen */

            // Neuen Pfad erstellen
            const pathParts = oldPath.split('/');
            pathParts.pop();
            const newPath = [...pathParts, newName].join('/');

            return { success: true, oldPath, newPath };
        } catch (err) {
            console.error(`Error renaming item ${oldPath} to ${newName}:`, err);
            setError(`Fehler beim Umbenennen: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Kopieren einer Datei oder eines Ordners
    const copyItem = async (sourcePath, destinationPath) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Element im Backend kopieren
            // /* BACKEND_INTEGRATION: Element kopieren */

            return { success: true, sourcePath, destinationPath };
        } catch (err) {
            console.error(`Error copying item ${sourcePath} to ${destinationPath}:`, err);
            setError(`Fehler beim Kopieren: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Verschieben einer Datei oder eines Ordners
    const moveItem = async (sourcePath, destinationPath) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Element im Backend verschieben
            // /* BACKEND_INTEGRATION: Element verschieben */

            return { success: true, sourcePath, destinationPath };
        } catch (err) {
            console.error(`Error moving item ${sourcePath} to ${destinationPath}:`, err);
            setError(`Fehler beim Verschieben: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    // Eigenschaften einer Datei oder eines Ordners abrufen
    const getItemProperties = async (path) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Eigenschaften vom Backend abrufen
            // /* BACKEND_INTEGRATION: Eigenschaften abrufen */

            // Beispieldaten
            const properties = {
                name: path.split('/').pop(),
                path: path,
                type: path.includes('.') ? 'file' : 'directory',
                size: '1.2 MB',
                created: new Date().toISOString(),
                modified: new Date().toISOString(),
                accessed: new Date().toISOString(),
                attributes: {
                    readonly: false,
                    hidden: false,
                    system: false,
                }
            };

            return properties;
        } catch (err) {
            console.error(`Error getting properties for ${path}:`, err);
            setError(`Fehler beim Abrufen der Eigenschaften: ${err.message}`);
            throw err;
        } finally {
            setIsLoading(false);
        }
    };

    // Suche nach Dateien und Ordnern
    const searchFiles = async (query, path = null, recursive = true) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Suche im Backend durchführen
            // /* BACKEND_INTEGRATION: Suche durchführen */

            // Beispieldaten
            const searchResults = [
                { name: `Dokument-${query}.docx`, path: `/path/to/Dokument-${query}.docx`, type: 'file', size: '25 KB', modified: '2023-01-15T10:30:00Z' },
                { name: `Tabelle-${query}.xlsx`, path: `/path/to/Tabelle-${query}.xlsx`, type: 'file', size: '156 KB', modified: '2023-02-20T14:45:00Z' },
                { name: query, path: `/path/to/${query}`, type: 'directory', modified: '2023-01-05T11:20:00Z' },
                { name: `${query}-test.txt`, path: `/path/to/${query}-test.txt`, type: 'file', size: '2 KB', modified: '2023-04-01T08:00:00Z' },
            ];

            return searchResults;
        } catch (err) {
            console.error(`Error searching for ${query}:`, err);
            setError(`Fehler bei der Suche: ${err.message}`);
            throw err;
        } finally {
            setIsLoading(false);
        }
    };

    // Template-Funktionen
    const saveAsTemplate = async (path, templateName) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Template im Backend speichern
            // /* BACKEND_INTEGRATION: Template speichern */

            return { success: true, templateName };
        } catch (err) {
            console.error(`Error saving template for ${path}:`, err);
            setError(`Fehler beim Speichern des Templates: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    const getTemplates = async () => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Templates vom Backend abrufen
            // /* BACKEND_INTEGRATION: Templates laden */

            // Beispieldaten
            const templates = [
                { name: 'Dokumentvorlage', path: '/templates/document-template', type: 'file', created: '2023-01-15T10:30:00Z' },
                { name: 'Projektordner', path: '/templates/project-folder', type: 'directory', created: '2023-02-20T14:45:00Z' },
            ];

            return templates;
        } catch (err) {
            console.error('Error getting templates:', err);
            setError('Fehler beim Abrufen der Templates');
            throw err;
        } finally {
            setIsLoading(false);
        }
    };

    const applyTemplate = async (templatePath, destinationPath) => {
        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Template im Backend anwenden
            // /* BACKEND_INTEGRATION: Template anwenden */

            return { success: true, destinationPath };
        } catch (err) {
            console.error(`Error applying template ${templatePath} to ${destinationPath}:`, err);
            setError(`Fehler beim Anwenden des Templates: ${err.message}`);
            return { success: false, error: err.message };
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <FileSystemContext.Provider
            value={{
                rootFolders,
                dataSources,
                isLoading,
                error,
                loadRootFolders,
                loadDataSources,
                addDataSource,
                removeDataSource,
                listDirectory,
                readFile,
                createFile,
                createDirectory,
                deleteItem,
                renameItem,
                copyItem,
                moveItem,
                getItemProperties,
                searchFiles,
                saveAsTemplate,
                getTemplates,
                applyTemplate,
            }}
        >
            {children}
        </FileSystemContext.Provider>
    );
};

// Custom Hook für einfachen Zugriff auf den FileSystem-Kontext
export const useFileSystem = () => {
    const context = useContext(FileSystemContext);
    if (context === undefined) {
        throw new Error('useFileSystem must be used within a FileSystemProvider');
    }
    return context;
};

export default FileSystemProvider;