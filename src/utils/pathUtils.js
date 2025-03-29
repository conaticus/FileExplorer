/**
 * Pfad-Hilfsfunktionen für die Anwendung
 */

/**
 * Prüft, ob ein Pfad ein Windows-Pfad ist
 *
 * @param {string} path - Der zu prüfende Pfad
 * @returns {boolean} Ob der Pfad ein Windows-Pfad ist
 */
export const isWindowsPath = (path) => {
    if (!path) return false;
    return path.includes('\\') || /^[A-Z]:/i.test(path);
};

/**
 * Normalisiert einen Pfad (standardisiert Separatoren)
 *
 * @param {string} path - Der zu normalisierende Pfad
 * @param {boolean} [toWindows=false] - Ob die Separatoren zu Windows-Backslashes konvertiert werden sollen
 * @returns {string} Normalisierter Pfad
 */
export const normalizePath = (path, toWindows = false) => {
    if (!path) return '';

    // Bestimme, ob es ein Windows-Pfad ist
    const isWinPath = isWindowsPath(path);

    // Bestimme den zu verwendenden Separator
    const separator = toWindows || isWinPath ? '\\' : '/';

    // Ersetze alle Separatoren durch den gewünschten Separator
    let normalizedPath = path.replace(/[\\/]+/g, separator);

    // Entferne den Separator am Ende, es sei denn, es ist ein Stammverzeichnis oder Laufwerk
    if (isWinPath) {
        if (normalizedPath.length > 3 && normalizedPath.endsWith(separator)) {
            normalizedPath = normalizedPath.slice(0, -1);
        }
    } else {
        if (normalizedPath.length > 1 && normalizedPath.endsWith(separator)) {
            normalizedPath = normalizedPath.slice(0, -1);
        }
    }

    return normalizedPath;
};

/**
 * Spaltet einen Pfad in Verzeichnis und Dateinamen
 *
 * @param {string} path - Der zu spaltende Pfad
 * @returns {Object} Objekt mit Verzeichnis und Dateinamen
 */
export const splitPath = (path) => {
    if (!path) return { dir: '', base: '' };

    // Normalisiere den Pfad
    const normalizedPath = normalizePath(path);

    // Bestimme, ob es ein Windows-Pfad ist
    const isWinPath = isWindowsPath(normalizedPath);

    // Bestimme den Separator
    const separator = isWinPath ? '\\' : '/';

    // Finde den letzten Separator
    const lastSepIndex = normalizedPath.lastIndexOf(separator);

    if (lastSepIndex === -1) {
        return {
            dir: isWinPath ? '.' : '/',
            base: normalizedPath
        };
    }

    // Spezialfall für Windows-Laufwerke (z.B. C:\)
    if (isWinPath && lastSepIndex === 2 && normalizedPath.charAt(1) === ':') {
        return {
            dir: normalizedPath.substring(0, lastSepIndex + 1),
            base: normalizedPath.substring(lastSepIndex + 1)
        };
    }

    return {
        dir: normalizedPath.substring(0, lastSepIndex) || (isWinPath ? '.' : '/'),
        base: normalizedPath.substring(lastSepIndex + 1)
    };
};

/**
 * Spaltet einen Dateinamen in Namen und Erweiterung
 *
 * @param {string} filename - Der zu spaltende Dateiname
 * @returns {Object} Objekt mit Namen und Erweiterung
 */
export const splitFilename = (filename) => {
    if (!filename) return { name: '', ext: '' };

    // Finde den letzten Punkt
    const lastDotIndex = filename.lastIndexOf('.');

    // Wenn kein Punkt gefunden wurde oder der Punkt am Anfang steht, hat die Datei keine Erweiterung
    if (lastDotIndex <= 0) {
        return { name: filename, ext: '' };
    }

    return {
        name: filename.substring(0, lastDotIndex),
        ext: filename.substring(lastDotIndex + 1)
    };
};

/**
 * Verbindet Pfadsegmente zu einem vollständigen Pfad
 *
 * @param {...string} paths - Pfadsegmente
 * @returns {string} Verbundener Pfad
 */
export const joinPath = (...paths) => {
    if (paths.length === 0) return '';

    // Prüfe, ob es ein Windows-Pfad ist
    const isWinPath = isWindowsPath(paths[0]);

    // Bestimme den Separator
    const separator = isWinPath ? '\\' : '/';

    // Normalisiere alle Pfade
    const normalizedPaths = paths.map(path =>
        path?.replace(/[\\/]+/g, separator).replace(new RegExp(`${separator}$`), '') || ''
    );

    // Verbinde die Pfade
    let result = normalizedPaths[0];

    for (let i = 1; i < normalizedPaths.length; i++) {
        const path = normalizedPaths[i];

        if (!path) continue;

        // Wenn der Pfad absolut ist, ersetze den bisherigen Pfad
        if (path.startsWith(separator) || /^[A-Z]:/i.test(path)) {
            result = path;
        } else {
            // Andernfalls füge den Pfad an
            result += separator + path;
        }
    }

    return result;
};

/**
 * Ermittelt den relativen Pfad von einem Pfad zu einem anderen
 *
 * @param {string} from - Ausgangspfad
 * @param {string} to - Zielpfad
 * @returns {string} Relativer Pfad
 */
export const relativePath = (from, to) => {
    if (!from || !to) return to || '';

    // Prüfe, ob die Pfade das gleiche Format haben
    const isFromWin = isWindowsPath(from);
    const isToWin = isWindowsPath(to);

    if (isFromWin !== isToWin) {
        // Wenn die Pfade unterschiedliche Formate haben, gib den Zielpfad zurück
        return to;
    }

    // Bestimme den Separator
    const separator = isFromWin ? '\\' : '/';

    // Normalisiere die Pfade
    const normalizedFrom = normalizePath(from);
    const normalizedTo = normalizePath(to);

    // Teile die Pfade in Segmente auf
    const fromSegments = normalizedFrom.split(separator).filter(Boolean);
    const toSegments = normalizedTo.split(separator).filter(Boolean);

    // Finde die gemeinsame Basis
    let commonIndex = 0;
    while (
        commonIndex < fromSegments.length &&
        commonIndex < toSegments.length &&
        fromSegments[commonIndex] === toSegments[commonIndex]
        ) {
        commonIndex++;
    }

    // Erstelle den relativen Pfad
    const upSegments = fromSegments.slice(commonIndex).map(() => '..');
    const downSegments = toSegments.slice(commonIndex);

    const relativeSegments = [...upSegments, ...downSegments];

    if (relativeSegments.length === 0) {
        return '.';
    }

    return relativeSegments.join(separator);
};

/**
 * Ermittelt den übergeordneten Pfad
 *
 * @param {string} path - Der Pfad
 * @returns {string} Übergeordneter Pfad
 */
export const parentPath = (path) => {
    if (!path) return '';

    // Normalisiere den Pfad
    const normalizedPath = normalizePath(path);

    // Bestimme, ob es ein Windows-Pfad ist
    const isWinPath = isWindowsPath(normalizedPath);

    // Bestimme den Separator
    const separator = isWinPath ? '\\' : '/';

    // Spezialfall für Stammverzeichnisse
    if (normalizedPath === separator) {
        return isWinPath ? '' : '/';
    }

    // Spezialfall für Windows-Laufwerke (z.B. C:\)
    if (isWinPath && normalizedPath.length === 3 && normalizedPath.charAt(1) === ':' && normalizedPath.charAt(2) === '\\') {
        return '';
    }

    // Teile den Pfad in Segmente auf
    const { dir } = splitPath(normalizedPath);
    return dir;
};

/**
 * Prüft, ob ein Pfad ein Unterpfad eines anderen Pfads ist
 *
 * @param {string} parent - Elternpfad
 * @param {string} child - Kindpfad
 * @returns {boolean} Ob der Kindpfad ein Unterpfad des Elternpfads ist
 */
export const isSubPath = (parent, child) => {
    if (!parent || !child) return false;

    // Normalisiere die Pfade
    const normalizedParent = normalizePath(parent);
    const normalizedChild = normalizePath(child);

    // Prüfe, ob die Pfade das gleiche Format haben
    const isParentWin = isWindowsPath(normalizedParent);
    const isChildWin = isWindowsPath(normalizedChild);

    if (isParentWin !== isChildWin) {
        return false;
    }

    // Bestimme den Separator
    const separator = isParentWin ? '\\' : '/';

    // Stelle sicher, dass der Elternpfad mit einem Separator endet
    const parentWithSep = normalizedParent.endsWith(separator)
        ? normalizedParent
        : normalizedParent + separator;

    return normalizedChild.startsWith(parentWithSep);
};

/**
 * Extrahiert den Teil eines Pfads nach einem bestimmten Basispfad
 *
 * @param {string} base - Basispfad
 * @param {string} path - Vollständiger Pfad
 * @returns {string} Teil des Pfads nach dem Basispfad
 */
export const extractPathAfterBase = (base, path) => {
    if (!base || !path) return path || '';

    // Normalisiere die Pfade
    const normalizedBase = normalizePath(base);
    const normalizedPath = normalizePath(path);

    // Prüfe, ob die Pfade das gleiche Format haben
    const isBaseWin = isWindowsPath(normalizedBase);
    const isPathWin = isWindowsPath(normalizedPath);

    if (isBaseWin !== isPathWin) {
        return path;
    }

    // Bestimme den Separator
    const separator = isBaseWin ? '\\' : '/';

    // Stelle sicher, dass der Basispfad mit einem Separator endet
    const baseWithSep = normalizedBase.endsWith(separator)
        ? normalizedBase
        : normalizedBase + separator;

    if (!normalizedPath.startsWith(baseWithSep)) {
        return path;
    }

    return normalizedPath.substring(baseWithSep.length);
};

/**
 * Konvertiert einen lokalen Pfad in eine lesbare Form
 *
 * @param {string} path - Der zu konvertierende Pfad
 * @returns {string} Lesbarer Pfad
 */
export const prettifyPath = (path) => {
    if (!path) return '';

    // Normalisiere den Pfad
    const normalizedPath = normalizePath(path);

    // Ersetze Home-Verzeichnis
    const homeDir = '~/';

    if (normalizedPath.startsWith(homeDir)) {
        return '~/' + normalizedPath.substring(homeDir.length);
    }

    return normalizedPath;
};

export default {
    isWindowsPath,
    normalizePath,
    splitPath,
    splitFilename,
    joinPath,
    relativePath,
    parentPath,
    isSubPath,
    extractPathAfterBase,
    prettifyPath
};