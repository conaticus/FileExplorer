/**
 * Tastenkombinationen für die Anwendung
 */

import { useEffect } from 'react';

// Definiert Tastenkombinationen für verschiedene Aktionen
export const SHORTCUTS = {
    // Navigation
    BACK: { key: '[', ctrl: true, description: 'Zurück' },
    FORWARD: { key: ']', ctrl: true, description: 'Vorwärts' },
    UP: { key: 'Backspace', description: 'Übergeordneter Ordner' },
    HOME: { key: 'Home', description: 'Home-Verzeichnis' },
    REFRESH: { key: 'F5', description: 'Aktualisieren' },

    // Ansichten
    VIEW_LIST: { key: '1', ctrl: true, description: 'Listenansicht' },
    VIEW_GRID: { key: '2', ctrl: true, description: 'Rasteransicht' },
    VIEW_DETAILS: { key: '3', ctrl: true, description: 'Detailansicht' },

    // Panels
    TOGGLE_SIDEBAR: { key: 'b', ctrl: true, description: 'Seitenleiste ein-/ausblenden' },
    TOGGLE_DETAILS: { key: 'i', ctrl: true, description: 'Detailansicht ein-/ausblenden' },
    TOGGLE_TERMINAL: { key: '`', ctrl: true, description: 'Terminal ein-/ausblenden' },

    // Aktionen
    SEARCH: { key: 'f', ctrl: true, description: 'Suchen' },
    SELECT_ALL: { key: 'a', ctrl: true, description: 'Alles auswählen' },
    CUT: { key: 'x', ctrl: true, description: 'Ausschneiden' },
    COPY: { key: 'c', ctrl: true, description: 'Kopieren' },
    PASTE: { key: 'v', ctrl: true, description: 'Einfügen' },
    DELETE: { key: 'Delete', description: 'Löschen' },
    RENAME: { key: 'F2', description: 'Umbenennen' },

    // Dateien und Ordner
    NEW_FILE: { key: 'n', ctrl: true, description: 'Neue Datei' },
    NEW_FOLDER: { key: 'n', ctrl: true, shift: true, description: 'Neuer Ordner' },

    // Anderes
    HELP: { key: 'F1', description: 'Hilfe' },
    KEYBOARD_SHORTCUTS: { key: 'k', ctrl: true, description: 'Tastenkombinationen anzeigen' },

    // Fokus
    FOCUS_ADDRESS_BAR: { key: 'l', ctrl: true, description: 'Adressleiste fokussieren' }
};

/**
 * Formatiert eine Tastenkombination für die Anzeige
 *
 * @param {Object} shortcut - Tastenkombination-Objekt
 * @param {string} [platform='win'] - Plattform ('win', 'mac', 'linux')
 * @returns {string} Formatierte Tastenkombination
 */
export const formatKeyboardShortcut = (shortcut, platform = 'win') => {
    if (!shortcut) return '';

    const keyLabels = {
        win: {
            ctrl: 'Strg',
            alt: 'Alt',
            shift: 'Umschalt',
            meta: 'Win'
        },
        mac: {
            ctrl: '⌃',
            alt: '⌥',
            shift: '⇧',
            meta: '⌘'
        },
        linux: {
            ctrl: 'Ctrl',
            alt: 'Alt',
            shift: 'Shift',
            meta: 'Super'
        }
    };

    const specialKeys = {
        ' ': 'Space',
        'ArrowUp': '↑',
        'ArrowDown': '↓',
        'ArrowLeft': '←',
        'ArrowRight': '→',
        'Escape': 'Esc',
        'Delete': 'Del',
        'Backspace': '⌫'
    };

    // Verwende die Labels für die angegebene Plattform oder Windows als Fallback
    const labels = keyLabels[platform] || keyLabels.win;

    // Erstelle die Tastenkombination
    const parts = [];

    if (shortcut.meta) parts.push(labels.meta);
    if (shortcut.ctrl) parts.push(labels.ctrl);
    if (shortcut.alt) parts.push(labels.alt);
    if (shortcut.shift) parts.push(labels.shift);

    const key = specialKeys[shortcut.key] || shortcut.key.toUpperCase();
    parts.push(key);

    if (platform === 'mac') {
        return parts.join('');
    }

    return parts.join('+');
};

/**
 * Prüft, ob ein Tastendruck einer Tastenkombination entspricht
 *
 * @param {KeyboardEvent} event - Tastendruck-Event
 * @param {Object} shortcut - Tastenkombination-Objekt
 * @returns {boolean} Ob der Tastendruck der Tastenkombination entspricht
 */
export const matchesShortcut = (event, shortcut) => {
    if (!event || !shortcut) return false;

    const key = event.key;

    return (
        key.toLowerCase() === shortcut.key.toLowerCase() &&
        !!event.ctrlKey === !!shortcut.ctrl &&
        !!event.altKey === !!shortcut.alt &&
        !!event.shiftKey === !!shortcut.shift &&
        !!event.metaKey === !!shortcut.meta
    );
};

/**
 * Hook zum Registrieren eines Keyboard-Handlers
 *
 * @param {Object} handlers - Map von Tastenkombinationen zu Handler-Funktionen
 * @param {boolean} [enabled=true] - Ob die Tastenkombinationen aktiviert sind
 */
export const useKeyboardShortcuts = (handlers, enabled = true) => {
    useEffect(() => {
        if (!enabled) return;

        const handleKeyDown = (event) => {
            // Überspringe Tastenkombinationen, wenn ein Eingabefeld fokussiert ist
            if (
                event.target.tagName === 'INPUT' ||
                event.target.tagName === 'TEXTAREA' ||
                event.target.isContentEditable
            ) {
                return;
            }

            for (const [shortcutName, handler] of Object.entries(handlers)) {
                const shortcut = SHORTCUTS[shortcutName];

                if (shortcut && matchesShortcut(event, shortcut)) {
                    event.preventDefault();
                    handler(event);
                    break;
                }
            }
        };

        document.addEventListener('keydown', handleKeyDown);

        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [handlers, enabled]);
};

/**
 * Gibt alle verfügbaren Tastenkombinationen zurück
 *
 * @param {string} [platform='win'] - Plattform ('win', 'mac', 'linux')
 * @returns {Array<Object>} Liste der Tastenkombinationen
 */
export const getAllShortcuts = (platform = 'win') => {
    return Object.entries(SHORTCUTS).map(([name, shortcut]) => ({
        name,
        display: formatKeyboardShortcut(shortcut, platform),
        description: shortcut.description,
        ...shortcut
    }));
};

/**
 * Erkennt die aktuelle Plattform
 *
 * @returns {string} Plattform ('win', 'mac', 'linux')
 */
export const detectPlatform = () => {
    const userAgent = window.navigator.userAgent.toLowerCase();

    if (userAgent.includes('mac')) return 'mac';
    if (userAgent.includes('linux')) return 'linux';
    return 'win';
};

export default {
    SHORTCUTS,
    formatKeyboardShortcut,
    matchesShortcut,
    useKeyboardShortcuts,
    getAllShortcuts,
    detectPlatform
};