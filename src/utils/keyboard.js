/**
 * Keyboard utility functions for the file explorer
 */

/**
 * Check if multiple selection is enabled based on the key event.
 * @param {KeyboardEvent} event - The keyboard event.
 * @returns {boolean} - Whether multiple selection is enabled.
 */
export const isMultiSelectKey = (event) => {
    return event.ctrlKey || event.metaKey;
};

/**
 * Check if range selection is enabled based on the key event.
 * @param {KeyboardEvent} event - The keyboard event.
 * @returns {boolean} - Whether range selection is enabled.
 */
export const isRangeSelectKey = (event) => {
    return event.shiftKey;
};

/**
 * Register global keyboard shortcuts.
 * @param {Object} handlers - An object mapping key combinations to handler functions.
 * @returns {Function} - A cleanup function to remove the event listeners.
 *
 * @example
 * // Usage
 * const cleanup = registerShortcuts({
 *   'Control+f': () => console.log('Search'),
 *   'Control+c': () => console.log('Copy'),
 *   'Delete': () => console.log('Delete'),
 * });
 *
 * // Later, to clean up
 * cleanup();
 */
export const registerShortcuts = (handlers) => {
    const handleKeyDown = (event) => {
        // Build the key combination string
        let combo = '';

        if (event.ctrlKey) combo += 'Control+';
        if (event.metaKey) combo += 'Meta+';
        if (event.altKey) combo += 'Alt+';
        if (event.shiftKey) combo += 'Shift+';

        // Add the key itself
        combo += event.key;

        // Check if we have a handler for this combination
        if (handlers[combo]) {
            event.preventDefault();
            handlers[combo](event);
        }
    };

    document.addEventListener('keydown', handleKeyDown);

    // Return a cleanup function
    return () => {
        document.removeEventListener('keydown', handleKeyDown);
    };
};

/**
 * Common keyboard shortcuts for the file explorer.
 */
export const SHORTCUTS = {
    COPY: 'Control+c',
    CUT: 'Control+x',
    PASTE: 'Control+v',
    SELECT_ALL: 'Control+a',
    DELETE: 'Delete',
    RENAME: 'F2',
    SEARCH: 'Control+f',
    NEW_FOLDER: 'Control+Shift+n',
    NEW_FILE: 'Control+n',
    REFRESH: 'F5',
    BACK: 'Alt+ArrowLeft',
    FORWARD: 'Alt+ArrowRight',
    UP_DIRECTORY: 'Alt+ArrowUp',
    HOME: 'Home',
    END: 'End',
};

/**
 * Format a shortcut for display.
 * @param {string} shortcut - The shortcut string.
 * @returns {string} - Formatted shortcut for display.
 */
export const formatShortcut = (shortcut) => {
    return shortcut
        .replace('Control+', 'Ctrl+')
        .replace('Meta+', '⌘')
        .replace('Alt+', 'Alt+')
        .replace('Shift+', 'Shift+')
        .replace('ArrowLeft', '←')
        .replace('ArrowRight', '→')
        .replace('ArrowUp', '↑')
        .replace('ArrowDown', '↓');
};

/**
 * Get the native modifier key based on the platform.
 * @returns {string} - 'Control' for Windows/Linux, 'Meta' for macOS.
 */
export const getPrimaryModifier = () => {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    return isMac ? 'Meta' : 'Control';
};

/**
 * Handle keyboard navigation in a list or grid.
 * @param {Event} event - The keyboard event.
 * @param {Array} items - The list of items.
 * @param {number} currentIndex - The current selected index.
 * @param {Object} options - Additional options for navigation.
 * @returns {number} - The new selected index.
 */
export const handleNavigation = (event, items, currentIndex, options = {}) => {
    const {
        columnsPerRow = 4, // Default for grid view
        isGrid = false,
        wraparound = false,
    } = options;

    const itemsLength = items.length;

    if (itemsLength === 0) return -1;

    let newIndex = currentIndex;

    switch (event.key) {
        case 'ArrowUp':
            if (isGrid) {
                newIndex = currentIndex - columnsPerRow;
            } else {
                newIndex = currentIndex - 1;
            }
            break;

        case 'ArrowDown':
            if (isGrid) {
                newIndex = currentIndex + columnsPerRow;
            } else {
                newIndex = currentIndex + 1;
            }
            break;

        case 'ArrowLeft':
            newIndex = currentIndex - 1;
            break;

        case 'ArrowRight':
            newIndex = currentIndex + 1;
            break;

        case 'Home':
            newIndex = 0;
            break;

        case 'End':
            newIndex = itemsLength - 1;
            break;

        default:
            return currentIndex;
    }

    // Apply wraparound if enabled
    if (wraparound) {
        if (newIndex < 0) {
            newIndex = itemsLength - 1;
        } else if (newIndex >= itemsLength) {
            newIndex = 0;
        }
    } else {
        // Clamp to valid range
        newIndex = Math.max(0, Math.min(itemsLength - 1, newIndex));
    }

    return newIndex;
};