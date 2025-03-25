/**
 * Dateityp-Definitionen und Icons für die Anwendung
 */

// Dateityp-Gruppen-Definitionen
export const fileGroups = {
    // Dokumente
    documents: {
        name: 'Dokumente',
        extensions: ['doc', 'docx', 'odt', 'rtf', 'txt', 'md', 'pdf', 'tex'],
        icon: 'file-text',
        color: '#2b579a'
    },
    // Tabellen
    spreadsheets: {
        name: 'Tabellen',
        extensions: ['xls', 'xlsx', 'ods', 'csv', 'tsv'],
        icon: 'file-excel',
        color: '#217346'
    },
    // Präsentationen
    presentations: {
        name: 'Präsentationen',
        extensions: ['ppt', 'pptx', 'odp', 'key'],
        icon: 'file-powerpoint',
        color: '#d24726'
    },
    // Bilder
    images: {
        name: 'Bilder',
        extensions: ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'tiff', 'ico'],
        icon: 'file-image',
        color: '#ff9800'
    },
    // Audio
    audio: {
        name: 'Audio',
        extensions: ['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a', 'wma'],
        icon: 'file-audio',
        color: '#8bc34a'
    },
    // Video
    video: {
        name: 'Video',
        extensions: ['mp4', 'avi', 'mov', 'wmv', 'mkv', 'webm', 'flv', 'm4v'],
        icon: 'file-video',
        color: '#e91e63'
    },
    // Archive
    archives: {
        name: 'Archive',
        extensions: ['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'iso'],
        icon: 'file-archive',
        color: '#795548'
    },
    // Code
    code: {
        name: 'Code',
        extensions: [
            'html', 'htm', 'css', 'js', 'jsx', 'ts', 'tsx', 'json', 'xml',
            'py', 'java', 'c', 'cpp', 'cs', 'go', 'rb', 'php', 'swift', 'kt',
            'sh', 'bash', 'ps1', 'sql', 'yaml', 'yml', 'toml'
        ],
        icon: 'file-code',
        color: '#00bcd4'
    },
    // Fonts
    fonts: {
        name: 'Schriftarten',
        extensions: ['ttf', 'otf', 'woff', 'woff2', 'eot'],
        icon: 'file',
        color: '#9c27b0'
    },
    // Executable
    executables: {
        name: 'Ausführbare Dateien',
        extensions: ['exe', 'msi', 'app', 'dmg', 'deb', 'rpm'],
        icon: 'file',
        color: '#f44336'
    }
};

// Zuordnung von Dateierweiterungen zu Gruppen
export const extensionMap = {};

// Erstelle die Zuordnung von Erweiterungen zu Gruppen
Object.entries(fileGroups).forEach(([groupKey, group]) => {
    group.extensions.forEach(ext => {
        extensionMap[ext.toLowerCase()] = {
            group: groupKey,
            icon: group.icon,
            color: group.color
        };
    });
});

/**
 * Bestimmt den Dateityp basierend auf Pfad und Erweiterung
 *
 * @param {string} path - Dateipfad oder -name
 * @param {string} [type='file'] - Elementtyp ('file' oder 'directory')
 * @returns {Object} Informationen zum Dateityp
 */
export const getFileType = (path, type = 'file') => {
    // Wenn es ein Verzeichnis ist, gib den Verzeichnistyp zurück
    if (type === 'directory') {
        return {
            type: 'directory',
            icon: 'folder',
            color: '#ffc107',
            groupName: 'Ordner'
        };
    }

    // Extrahiere die Erweiterung aus dem Pfad
    const fileName = path.split(/[\\/]/).pop() || '';
    const extension = fileName.includes('.')
        ? fileName.split('.').pop().toLowerCase()
        : '';

    // Wenn die Erweiterung bekannt ist, gib die zugehörigen Informationen zurück
    if (extension && extensionMap[extension]) {
        const { group, icon, color } = extensionMap[extension];
        return {
            type: 'file',
            extension,
            icon,
            color,
            group,
            groupName: fileGroups[group].name
        };
    }

    // Standardwerte für unbekannte Dateitypen
    return {
        type: 'file',
        extension: extension || '',
        icon: 'file',
        color: '#607d8b',
        group: 'other',
        groupName: 'Andere'
    };
};

/**
 * Liefert ein passendes Icon für einen Dateityp
 *
 * @param {string} fileType - Typ der Datei ('file', 'directory' oder eine Erweiterung)
 * @param {string} [extension=''] - Dateierweiterung (optional)
 * @returns {string} SVG-Pfad für das Icon
 */
export const getFileIconPath = (fileType, extension = '') => {
    // Grundlegende Icon-Pfade
    const iconPaths = {
        folder: 'M10 3H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h16a1 1 0 0 0 1-1V7a1 1 0 0 0-1-1h-8l-2-2z',
        file: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6',
        'file-text': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M16 13H8 M16 17H8 M10 9H8',
        'file-image': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M8 10a1 1 0 1 0 2 0a1 1 0 1 0 -2 0 M20 15l-2-2-4 5-3-3-3 3',
        'file-audio': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 15a1 1 0 1 0 2 0a1 1 0 1 0 -2 0 M13 11a1 1 0 1 0 2 0a1 1 0 1 0 -2 0 M9 11V15 M13 11v4',
        'file-video': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M10 11l5 3l-5 3v-6z',
        'file-archive': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M12 10v8 M10 12h4 M10 16h4',
        'file-code': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M10 12l-2 2l2 2 M14 12l2 2l-2 2',
        'file-excel': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 13l6 6 M15 13l-6 6',
        'file-word': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 13l3 5l3-5',
        'file-powerpoint': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 15a3 3 0 1 0 0-6a3 3 0 0 0 0 6z M9 12h8',
        'file-pdf': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 13h6 M9 17h6 M9 9h1',
    };

    // Bestimme das Icon basierend auf dem Dateityp und der Erweiterung
    if (fileType === 'directory') {
        return iconPaths.folder;
    }

    if (extension) {
        const extensionLower = extension.toLowerCase();

        // Überprüfe, ob die Erweiterung in der Zuordnung vorhanden ist
        if (extensionLower in extensionMap) {
            const iconName = extensionMap[extensionLower].icon;
            return iconPaths[iconName] || iconPaths.file;
        }
    }

    // Standard-Dateityp
    return iconPaths.file;
};

/**
 * Gibt eine Farbzuordnung für einen Dateityp zurück
 *
 * @param {string} fileType - Typ der Datei ('file', 'directory' oder eine Erweiterung)
 * @param {string} [extension=''] - Dateierweiterung (optional)
 * @returns {string} Farbcode für den Dateityp
 */
export const getFileTypeColor = (fileType, extension = '') => {
    // Wenn es ein Verzeichnis ist
    if (fileType === 'directory') {
        return '#ffc107'; // Gelb für Ordner
    }

    // Prüfe, ob die Erweiterung in der Zuordnung vorhanden ist
    if (extension) {
        const extensionLower = extension.toLowerCase();

        if (extensionLower in extensionMap) {
            return extensionMap[extensionLower].color;
        }
    }

    // Standard-Farbe für unbekannte Dateitypen
    return '#607d8b'; // Blaugrau
};

/**
 * Bildet einen Dateityp oder eine Erweiterung auf eine Gruppe ab
 *
 * @param {string} fileType - Typ der Datei ('file', 'directory' oder eine Erweiterung)
 * @param {string} [extension=''] - Dateierweiterung (optional)
 * @returns {string} Name der Dateityp-Gruppe
 */
export const getFileTypeGroup = (fileType, extension = '') => {
    // Wenn es ein Verzeichnis ist
    if (fileType === 'directory') {
        return 'directories';
    }

    // Prüfe, ob die Erweiterung in der Zuordnung vorhanden ist
    if (extension) {
        const extensionLower = extension.toLowerCase();

        if (extensionLower in extensionMap) {
            return extensionMap[extensionLower].group;
        }
    }

    // Standard-Gruppe für unbekannte Dateitypen
    return 'other';
};

export default {
    fileGroups,
    extensionMap,
    getFileType,
    getFileIconPath,
    getFileTypeColor,
    getFileTypeGroup
};