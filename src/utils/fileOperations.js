/**
 * Dienstprogramm für Dateioperationen
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Erstellt eine Datei im angegebenen Pfad mit dem angegebenen Inhalt
 *
 * @param {string} path - Pfad der zu erstellenden Datei
 * @param {string} content - Inhalt der Datei
 * @returns {Promise<{ success: boolean, path: string, error?: string }>} Ergebnis der Operation
 */
export const createFile = async (path, content = '') => {
    try {
        // [Backend Integration] - Datei im Backend erstellen
        // /* BACKEND_INTEGRATION: Datei erstellen */

        await invoke('create_file', { path, content });

        return { success: true, path };
    } catch (error) {
        console.error(`Error creating file ${path}:`, error);
        return { success: false, path, error: error.message || String(error) };
    }
};

/**
 * Erstellt einen Ordner im angegebenen Pfad
 *
 * @param {string} path - Pfad des zu erstellenden Ordners
 * @returns {Promise<{ success: boolean, path: string, error?: string }>} Ergebnis der Operation
 */
export const createDirectory = async (path) => {
    try {
        // [Backend Integration] - Ordner im Backend erstellen
        // /* BACKEND_INTEGRATION: Ordner erstellen */

        await invoke('create_directory', { path });

        return { success: true, path };
    } catch (error) {
        console.error(`Error creating directory ${path}:`, error);
        return { success: false, path, error: error.message || String(error) };
    }
};

/**
 * Löscht eine Datei oder einen Ordner
 *
 * @param {string} path - Pfad des zu löschenden Elements
 * @param {boolean} [recursive=true] - Ob Ordner rekursiv gelöscht werden sollen
 * @returns {Promise<{ success: boolean, error?: string }>} Ergebnis der Operation
 */
export const deleteItem = async (path, recursive = true) => {
    try {
        // [Backend Integration] - Element im Backend löschen
        // /* BACKEND_INTEGRATION: Element löschen */

        await invoke('delete_item', { path, recursive });

        return { success: true };
    } catch (error) {
        console.error(`Error deleting item ${path}:`, error);
        return { success: false, error: error.message || String(error) };
    }
};

/**
 * Benennt eine Datei oder einen Ordner um
 *
 * @param {string} oldPath - Aktueller Pfad des Elements
 * @param {string} newName - Neuer Name des Elements (ohne Pfad)
 * @returns {Promise<{ success: boolean, oldPath: string, newPath: string, error?: string }>} Ergebnis der Operation
 */
export const renameItem = async (oldPath, newName) => {
    try {
        // Erstelle den neuen Pfad
        const separator = oldPath.includes('\\') ? '\\' : '/';
        const pathParts = oldPath.split(separator);
        pathParts.pop(); // Entferne den alten Namen
        const newPath = [...pathParts, newName].join(separator);

        // [Backend Integration] - Element im Backend umbenennen
        // /* BACKEND_INTEGRATION: Element umbenennen */

        await invoke('rename_item', { oldPath, newPath });

        return { success: true, oldPath, newPath };
    } catch (error) {
        console.error(`Error renaming item ${oldPath} to ${newName}:`, error);
        return { success: false, oldPath, newPath: '', error: error.message || String(error) };
    }
};

/**
 * Kopiert eine Datei oder einen Ordner
 *
 * @param {string} sourcePath - Quellpfad des Elements
 * @param {string} destinationPath - Zielpfad des Elements
 * @param {boolean} [overwrite=false] - Ob bestehende Dateien überschrieben werden sollen
 * @returns {Promise<{ success: boolean, sourcePath: string, destinationPath: string, error?: string }>} Ergebnis der Operation
 */
export const copyItem = async (sourcePath, destinationPath, overwrite = false) => {
    try {
        // [Backend Integration] - Element im Backend kopieren
        // /* BACKEND_INTEGRATION: Element kopieren */

        await invoke('copy_item', { sourcePath, destinationPath, overwrite });

        return { success: true, sourcePath, destinationPath };
    } catch (error) {
        console.error(`Error copying item ${sourcePath} to ${destinationPath}:`, error);
        return {
            success: false,
            sourcePath,
            destinationPath,
            error: error.message || String(error)
        };
    }
};

/**
 * Verschiebt eine Datei oder einen Ordner
 *
 * @param {string} sourcePath - Quellpfad des Elements
 * @param {string} destinationPath - Zielpfad des Elements
 * @param {boolean} [overwrite=false] - Ob bestehende Dateien überschrieben werden sollen
 * @returns {Promise<{ success: boolean, sourcePath: string, destinationPath: string, error?: string }>} Ergebnis der Operation
 */
export const moveItem = async (sourcePath, destinationPath, overwrite = false) => {
    try {
        // [Backend Integration] - Element im Backend verschieben
        // /* BACKEND_INTEGRATION: Element verschieben */

        await invoke('move_item', { sourcePath, destinationPath, overwrite });

        return { success: true, sourcePath, destinationPath };
    } catch (error) {
        console.error(`Error moving item ${sourcePath} to ${destinationPath}:`, error);
        return {
            success: false,
            sourcePath,
            destinationPath,
            error: error.message || String(error)
        };
    }
};

/**
 * Liest den Inhalt einer Datei
 *
 * @param {string} path - Pfad der zu lesenden Datei
 * @returns {Promise<{ success: boolean, content: string, error?: string }>} Ergebnis der Operation
 */
export const readFile = async (path) => {
    try {
        // [Backend Integration] - Datei im Backend lesen
        // /* BACKEND_INTEGRATION: Datei lesen */

        const content = await invoke('read_file', { path });

        return { success: true, content };
    } catch (error) {
        console.error(`Error reading file ${path}:`, error);
        return { success: false, content: '', error: error.message || String(error) };
    }
};

/**
 * Schreibt Inhalt in eine Datei
 *
 * @param {string} path - Pfad der zu schreibenden Datei
 * @param {string} content - Zu schreibender Inhalt
 * @returns {Promise<{ success: boolean, error?: string }>} Ergebnis der Operation
 */
export const writeFile = async (path, content) => {
    try {
        // [Backend Integration] - In Datei im Backend schreiben
        // /* BACKEND_INTEGRATION: In Datei schreiben */

        await invoke('write_file', { path, content });

        return { success: true };
    } catch (error) {
        console.error(`Error writing to file ${path}:`, error);
        return { success: false, error: error.message || String(error) };
    }
};

/**
 * Listet den Inhalt eines Verzeichnisses auf
 *
 * @param {string} path - Pfad des Verzeichnisses
 * @param {boolean} [includeHidden=false] - Ob versteckte Dateien einbezogen werden sollen
 * @returns {Promise<Array<Object>>} Liste der Elemente im Verzeichnis
 */
export const listDirectory = async (path, includeHidden = false) => {
    try {
        // [Backend Integration] - Verzeichnisinhalt vom Backend abrufen
        // /* BACKEND_INTEGRATION: Verzeichnisinhalt laden */

        const items = await invoke('list_directory', { path, includeHidden });
        return items;
    } catch (error) {
        console.error(`Error listing directory ${path}:`, error);
        throw error;
    }
};

/**
 * Ruft die Eigenschaften eines Elements ab
 *
 * @param {string} path - Pfad des Elements
 * @returns {Promise<Object>} Eigenschaften des Elements
 */
export const getItemProperties = async (path) => {
    try {
        // [Backend Integration] - Eigenschaften vom Backend abrufen
        // /* BACKEND_INTEGRATION: Eigenschaften abrufen */

        const properties = await invoke('get_item_properties', { path });
        return properties;
    } catch (error) {
        console.error(`Error getting properties for ${path}:`, error);
        throw error;
    }
};

/**
 * Sucht nach Dateien und Ordnern
 *
 * @param {string} query - Suchanfrage
 * @param {string} [path=null] - Pfad, in dem gesucht werden soll (null für überall)
 * @param {boolean} [recursive=true] - Ob rekursiv gesucht werden soll
 * @param {Object} [options] - Weitere Suchoptionen
 * @returns {Promise<Array<Object>>} Liste der gefundenen Elemente
 */
export const searchFiles = async (query, path = null, recursive = true, options = {}) => {
    try {
        // [Backend Integration] - Suche im Backend durchführen
        // /* BACKEND_INTEGRATION: Suche durchführen */

        const results = await invoke('search_files', { query, path, recursive, options });
        return results;
    } catch (error) {
        console.error(`Error searching for ${query}:`, error);
        throw error;
    }
};

/**
 * Speichert ein Element als Template
 *
 * @param {string} path - Pfad des zu speichernden Elements
 * @param {Object} templateData - Daten für das Template
 * @returns {Promise<{ success: boolean, templateName: string, error?: string }>} Ergebnis der Operation
 */
export const saveAsTemplate = async (path, templateData) => {
    try {
        // [Backend Integration] - Template im Backend speichern
        // /* BACKEND_INTEGRATION: Template speichern */

        await invoke('save_as_template', { path, templateData });

        return { success: true, templateName: templateData.name };
    } catch (error) {
        console.error(`Error saving template for ${path}:`, error);
        return { success: false, templateName: '', error: error.message || String(error) };
    }
};

/**
 * Wendet ein Template an
 *
 * @param {string} templatePath - Pfad des Templates
 * @param {string} destinationPath - Zielpfad für die Anwendung
 * @returns {Promise<{ success: boolean, destinationPath: string, error?: string }>} Ergebnis der Operation
 */
export const applyTemplate = async (templatePath, destinationPath) => {
    try {
        // [Backend Integration] - Template im Backend anwenden
        // /* BACKEND_INTEGRATION: Template anwenden */

        await invoke('apply_template', { templatePath, destinationPath });

        return { success: true, destinationPath };
    } catch (error) {
        console.error(`Error applying template ${templatePath} to ${destinationPath}:`, error);
        return { success: false, destinationPath, error: error.message || String(error) };
    }
};

export default {
    createFile,
    createDirectory,
    deleteItem,
    renameItem,
    copyItem,
    moveItem,
    readFile,
    writeFile,
    listDirectory,
    getItemProperties,
    searchFiles,
    saveAsTemplate,
    applyTemplate
};