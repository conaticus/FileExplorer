/**
 * Formatierungsfunktionen für die Anwendung
 */

/**
 * Formatiert eine Dateigröße in eine lesbare Form
 *
 * @param {number|string} bytes - Dateigröße in Bytes
 * @param {number} [decimals=2] - Anzahl der Nachkommastellen
 * @returns {string} Formatierte Dateigröße mit Einheit
 */
export const formatFileSize = (bytes, decimals = 2) => {
    if (bytes === 0 || bytes === undefined || bytes === null) return '0 Bytes';

    // Wenn bytes ein String ist, versuche ihn zu konvertieren
    if (typeof bytes === 'string') {
        // Entferne non-numerische Zeichen, außer Dezimalpunkt
        const numericBytes = parseFloat(bytes.replace(/[^\d.-]/g, ''));

        // Wenn der String bereits eine Einheit enthält, gib ihn zurück
        if (isNaN(numericBytes) || bytes !== numericBytes.toString()) {
            return bytes;
        }

        bytes = numericBytes;
    }

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

/**
 * Formatiert ein Datum in eine lesbare Form
 *
 * @param {Date|string|number} date - Das zu formatierende Datum
 * @param {Object} [options] - Formatierungsoptionen
 * @param {boolean} [options.includeTime=true] - Ob die Uhrzeit einbezogen werden soll
 * @param {boolean} [options.includeSeconds=false] - Ob Sekunden angezeigt werden sollen
 * @param {string} [options.locale='de-DE'] - Locale für die Formatierung
 * @returns {string} Formatiertes Datum
 */
export const formatDate = (date, options = {}) => {
    const {
        includeTime = true,
        includeSeconds = false,
        locale = 'de-DE'
    } = options;

    if (!date) return '';

    try {
        const dateObj = date instanceof Date ? date : new Date(date);

        // Wenn das Datum ungültig ist, gib den ursprünglichen Wert zurück
        if (isNaN(dateObj.getTime())) {
            return String(date);
        }

        // Formatierungsoptionen für toLocaleDateString und toLocaleTimeString
        const dateOptions = {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit'
        };

        const timeOptions = {
            hour: '2-digit',
            minute: '2-digit',
            second: includeSeconds ? '2-digit' : undefined,
            hour12: false
        };

        let formatted = dateObj.toLocaleDateString(locale, dateOptions);

        if (includeTime) {
            formatted += ' ' + dateObj.toLocaleTimeString(locale, timeOptions);
        }

        return formatted;
    } catch (error) {
        console.error('Error formatting date:', error);
        return String(date);
    }
};

/**
 * Formatiert einen relativen Zeitraum (z.B. "vor 2 Stunden")
 *
 * @param {Date|string|number} date - Das zu formatierende Datum
 * @param {string} [locale='de-DE'] - Locale für die Formatierung
 * @returns {string} Relativer Zeitraum
 */
export const formatRelativeTime = (date, locale = 'de-DE') => {
    if (!date) return '';

    try {
        const dateObj = date instanceof Date ? date : new Date(date);

        // Wenn das Datum ungültig ist, gib den ursprünglichen Wert zurück
        if (isNaN(dateObj.getTime())) {
            return String(date);
        }

        const now = new Date();
        const diffMs = now - dateObj;
        const diffSec = Math.floor(diffMs / 1000);
        const diffMin = Math.floor(diffSec / 60);
        const diffHour = Math.floor(diffMin / 60);
        const diffDay = Math.floor(diffHour / 24);
        const diffMonth = Math.floor(diffDay / 30);
        const diffYear = Math.floor(diffDay / 365);

        // Formatierungstexte für verschiedene Sprachen
        const texts = {
            'de-DE': {
                now: 'gerade eben',
                seconds: 'vor {0} Sekunden',
                minute: 'vor einer Minute',
                minutes: 'vor {0} Minuten',
                hour: 'vor einer Stunde',
                hours: 'vor {0} Stunden',
                day: 'gestern',
                days: 'vor {0} Tagen',
                month: 'vor einem Monat',
                months: 'vor {0} Monaten',
                year: 'vor einem Jahr',
                years: 'vor {0} Jahren'
            },
            'en-US': {
                now: 'just now',
                seconds: '{0} seconds ago',
                minute: 'a minute ago',
                minutes: '{0} minutes ago',
                hour: 'an hour ago',
                hours: '{0} hours ago',
                day: 'yesterday',
                days: '{0} days ago',
                month: 'a month ago',
                months: '{0} months ago',
                year: 'a year ago',
                years: '{0} years ago'
            }
        };

        // Verwende die Texte für die angegebene Sprache oder Englisch als Fallback
        const t = texts[locale] || texts['en-US'];

        // Ersetze den Platzhalter {0} in den Texten
        const replaceParams = (text, value) => text.replace('{0}', value);

        // Formatiere den relativen Zeitraum
        if (diffSec < 5) return t.now;
        if (diffSec < 60) return replaceParams(t.seconds, diffSec);
        if (diffMin === 1) return t.minute;
        if (diffMin < 60) return replaceParams(t.minutes, diffMin);
        if (diffHour === 1) return t.hour;
        if (diffHour < 24) return replaceParams(t.hours, diffHour);
        if (diffDay === 1) return t.day;
        if (diffDay < 30) return replaceParams(t.days, diffDay);
        if (diffMonth === 1) return t.month;
        if (diffMonth < 12) return replaceParams(t.months, diffMonth);
        if (diffYear === 1) return t.year;
        return replaceParams(t.years, diffYear);
    } catch (error) {
        console.error('Error formatting relative time:', error);
        return String(date);
    }
};

/**
 * Formatiert einen Pfad für die Anzeige
 *
 * @param {string} path - Der zu formatierende Pfad
 * @param {number} [maxLength=50] - Maximale Länge des formatierten Pfads
 * @returns {string} Formatierter Pfad
 */
export const formatPath = (path, maxLength = 50) => {
    if (!path) return '';

    // Wenn der Pfad kürzer als die maximale Länge ist, gib ihn unverändert zurück
    if (path.length <= maxLength) {
        return path;
    }

    // Bestimme, ob es ein Windows-Pfad ist
    const isWindowsPath = path.includes('\\') || /^[A-Z]:/.test(path);
    const separator = isWindowsPath ? '\\' : '/';

    // Teile den Pfad in Segmente auf
    const segments = path.split(separator).filter(Boolean);

    // Wenn es zu wenige Segmente gibt, kürze den Pfad in der Mitte
    if (segments.length <= 2) {
        const halfMaxLength = Math.floor(maxLength / 2) - 1;
        return path.substring(0, halfMaxLength) + '...' + path.substring(path.length - halfMaxLength);
    }

    // Behalte das erste und das letzte Segment bei
    const firstSegment = segments[0];
    const lastSegment = segments[segments.length - 1];

    // Wenn der Pfad mit einem Laufwerk beginnt (z.B. C:), füge es dem ersten Segment hinzu
    const firstPart = isWindowsPath && path.match(/^[A-Z]:/i)
        ? path.substring(0, 2) + separator + firstSegment
        : (isWindowsPath ? '' : separator) + firstSegment;

    // Erstelle den formatierten Pfad
    return firstPart + separator + '...' + separator + lastSegment;
};

/**
 * Formatiert Text nach maximaler Länge mit Ellipsis
 *
 * @param {string} text - Der zu formatierende Text
 * @param {number} [maxLength=50] - Maximale Länge des formatierten Texts
 * @param {boolean} [ellipsisInMiddle=false] - Ob die Ellipsis in der Mitte platziert werden soll
 * @returns {string} Formatierter Text
 */
export const truncateText = (text, maxLength = 50, ellipsisInMiddle = false) => {
    if (!text) return '';

    // Wenn der Text kürzer als die maximale Länge ist, gib ihn unverändert zurück
    if (text.length <= maxLength) {
        return text;
    }

    // Wenn die Ellipsis in der Mitte platziert werden soll
    if (ellipsisInMiddle) {
        const halfMaxLength = Math.floor(maxLength / 2) - 1;
        return text.substring(0, halfMaxLength) + '...' + text.substring(text.length - halfMaxLength);
    }

    // Andernfalls platziere die Ellipsis am Ende
    return text.substring(0, maxLength - 3) + '...';
};

export default {
    formatFileSize,
    formatDate,
    formatRelativeTime,
    formatPath,
    truncateText
};