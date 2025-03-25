import React, { createContext, useContext, useState, useReducer, useEffect } from 'react';

// Erstellen eines Kontexts für den Anwendungszustand
export const AppStateContext = createContext();

// Aktionstypen
const ActionTypes = {
    SET_CURRENT_PATH: 'SET_CURRENT_PATH',
    ADD_TO_HISTORY: 'ADD_TO_HISTORY',
    GO_BACK: 'GO_BACK',
    GO_FORWARD: 'GO_FORWARD',
    SET_SELECTED_ITEMS: 'SET_SELECTED_ITEMS',
    ADD_SELECTED_ITEM: 'ADD_SELECTED_ITEM',
    REMOVE_SELECTED_ITEM: 'REMOVE_SELECTED_ITEM',
    CLEAR_SELECTED_ITEMS: 'CLEAR_SELECTED_ITEMS',
    SET_VIEW_MODE: 'SET_VIEW_MODE',
    TOGGLE_DETAIL_PANEL: 'TOGGLE_DETAIL_PANEL',
    TOGGLE_TERMINAL_PANEL: 'TOGGLE_TERMINAL_PANEL',
    SET_SEARCH_QUERY: 'SET_SEARCH_QUERY',
    SET_SEARCH_RESULTS: 'SET_SEARCH_RESULTS',
    CLEAR_SEARCH: 'CLEAR_SEARCH',
    ADD_FAVORITE: 'ADD_FAVORITE',
    REMOVE_FAVORITE: 'REMOVE_FAVORITE',
    ADD_RECENT: 'ADD_RECENT',
    SET_SORT_BY: 'SET_SORT_BY',
    SET_SORT_DIRECTION: 'SET_SORT_DIRECTION',
    SET_LOADING: 'SET_LOADING',
    SET_ERROR: 'SET_ERROR',
    TOGGLE_SIDEBAR: 'TOGGLE_SIDEBAR',
};

// Anfänglicher Zustand
const initialState = {
    currentPath: '', // Aktueller Dateipfad
    history: [], // Verlauf besuchter Pfade
    historyIndex: -1, // Aktueller Index im Verlauf
    selectedItems: [], // Ausgewählte Dateien und Ordner
    viewMode: 'list', // Ansichtsmodus: 'list', 'grid', 'details'
    isDetailPanelOpen: false, // Ist das Detailpanel geöffnet?
    isTerminalPanelOpen: false, // Ist das Terminalpanel geöffnet?
    searchQuery: '', // Aktuelle Suchanfrage
    searchResults: [], // Suchergebnisse
    isSearchActive: false, // Ist die Suche aktiv?
    favorites: [], // Favorisierte Pfade
    recentPaths: [], // Zuletzt besuchte Pfade
    sortBy: 'name', // Sortieren nach: 'name', 'date', 'size', 'type'
    sortDirection: 'asc', // Sortierrichtung: 'asc', 'desc'
    isLoading: false, // Wird geladen?
    error: null, // Fehler
    isSidebarOpen: true, // Ist die Seitenleiste geöffnet?
};

// Reducer-Funktion
const appStateReducer = (state, action) => {
    switch (action.type) {
        case ActionTypes.SET_CURRENT_PATH:
            return {
                ...state,
                currentPath: action.payload,
            };

        case ActionTypes.ADD_TO_HISTORY:
            // Wenn wir uns nicht am Ende des Verlaufs befinden, entferne alles nach dem aktuellen Index
            const newHistory = state.historyIndex < state.history.length - 1
                ? state.history.slice(0, state.historyIndex + 1)
                : state.history;

            // Füge den neuen Pfad zum Verlauf hinzu, wenn er sich vom aktuellen unterscheidet
            if (newHistory.length === 0 || newHistory[newHistory.length - 1] !== action.payload) {
                return {
                    ...state,
                    history: [...newHistory, action.payload],
                    historyIndex: newHistory.length,
                };
            }
            return state;

        case ActionTypes.GO_BACK:
            if (state.historyIndex > 0) {
                return {
                    ...state,
                    historyIndex: state.historyIndex - 1,
                    currentPath: state.history[state.historyIndex - 1],
                };
            }
            return state;

        case ActionTypes.GO_FORWARD:
            if (state.historyIndex < state.history.length - 1) {
                return {
                    ...state,
                    historyIndex: state.historyIndex + 1,
                    currentPath: state.history[state.historyIndex + 1],
                };
            }
            return state;

        case ActionTypes.SET_SELECTED_ITEMS:
            return {
                ...state,
                selectedItems: action.payload,
            };

        case ActionTypes.ADD_SELECTED_ITEM:
            return {
                ...state,
                selectedItems: [...state.selectedItems, action.payload],
            };

        case ActionTypes.REMOVE_SELECTED_ITEM:
            return {
                ...state,
                selectedItems: state.selectedItems.filter(item => item !== action.payload),
            };

        case ActionTypes.CLEAR_SELECTED_ITEMS:
            return {
                ...state,
                selectedItems: [],
            };

        case ActionTypes.SET_VIEW_MODE:
            return {
                ...state,
                viewMode: action.payload,
            };

        case ActionTypes.TOGGLE_DETAIL_PANEL:
            return {
                ...state,
                isDetailPanelOpen: action.payload !== undefined ? action.payload : !state.isDetailPanelOpen,
            };

        case ActionTypes.TOGGLE_TERMINAL_PANEL:
            return {
                ...state,
                isTerminalPanelOpen: action.payload !== undefined ? action.payload : !state.isTerminalPanelOpen,
            };

        case ActionTypes.SET_SEARCH_QUERY:
            return {
                ...state,
                searchQuery: action.payload,
                isSearchActive: !!action.payload,
            };

        case ActionTypes.SET_SEARCH_RESULTS:
            return {
                ...state,
                searchResults: action.payload,
            };

        case ActionTypes.CLEAR_SEARCH:
            return {
                ...state,
                searchQuery: '',
                searchResults: [],
                isSearchActive: false,
            };

        case ActionTypes.ADD_FAVORITE:
            // Prüfen, ob der Pfad bereits in den Favoriten ist
            if (!state.favorites.includes(action.payload)) {
                return {
                    ...state,
                    favorites: [...state.favorites, action.payload],
                };
            }
            return state;

        case ActionTypes.REMOVE_FAVORITE:
            return {
                ...state,
                favorites: state.favorites.filter(path => path !== action.payload),
            };

        case ActionTypes.ADD_RECENT:
            // Füge den Pfad zu den letzten Pfaden hinzu und begrenze die Anzahl auf 10
            const newRecentPaths = [
                action.payload,
                ...state.recentPaths.filter(path => path !== action.payload),
            ].slice(0, 10);

            return {
                ...state,
                recentPaths: newRecentPaths,
            };

        case ActionTypes.SET_SORT_BY:
            return {
                ...state,
                sortBy: action.payload,
            };

        case ActionTypes.SET_SORT_DIRECTION:
            return {
                ...state,
                sortDirection: action.payload,
            };

        case ActionTypes.SET_LOADING:
            return {
                ...state,
                isLoading: action.payload,
            };

        case ActionTypes.SET_ERROR:
            return {
                ...state,
                error: action.payload,
            };

        case ActionTypes.TOGGLE_SIDEBAR:
            return {
                ...state,
                isSidebarOpen: action.payload !== undefined ? action.payload : !state.isSidebarOpen,
            };

        default:
            return state;
    }
};

export const AppStateProvider = ({ children }) => {
    // Verwende useReducer für komplexen Zustand
    const [state, dispatch] = useReducer(appStateReducer, initialState);

    // Lade den gespeicherten Zustand beim Start
    useEffect(() => {
        try {
            const savedState = JSON.parse(localStorage.getItem('appState'));
            if (savedState) {
                // Lade nur bestimmte Teile des Zustands
                if (savedState.favorites) {
                    dispatch({ type: ActionTypes.SET_SELECTED_ITEMS, payload: savedState.favorites });
                }

                if (savedState.viewMode) {
                    dispatch({ type: ActionTypes.SET_VIEW_MODE, payload: savedState.viewMode });
                }

                if (savedState.recentPaths) {
                    // Aktualisiere den Zustand recentPaths direkt
                    savedState.recentPaths.forEach(path => {
                        dispatch({ type: ActionTypes.ADD_RECENT, payload: path });
                    });
                }
            }

            // [Backend Integration] - Lade den letzten Pfad aus dem Backend
            // /* BACKEND_INTEGRATION: Letzten Pfad aus dem Backend laden */
        } catch (error) {
            console.error('Error loading app state:', error);
        }
    }, []);

    // Speichere den Zustand bei Änderungen
    useEffect(() => {
        const stateToSave = {
            favorites: state.favorites,
            viewMode: state.viewMode,
            recentPaths: state.recentPaths,
        };

        localStorage.setItem('appState', JSON.stringify(stateToSave));
    }, [state.favorites, state.viewMode, state.recentPaths]);

    // Aktionen
    const actions = {
        setCurrentPath: (path) => {
            dispatch({ type: ActionTypes.SET_CURRENT_PATH, payload: path });
            dispatch({ type: ActionTypes.ADD_TO_HISTORY, payload: path });
            dispatch({ type: ActionTypes.ADD_RECENT, payload: path });

            // [Backend Integration] - Aktuellen Pfad im Backend setzen
            // /* BACKEND_INTEGRATION: Aktuellen Pfad im Backend setzen */
        },

        goBack: () => {
            dispatch({ type: ActionTypes.GO_BACK });
        },

        goForward: () => {
            dispatch({ type: ActionTypes.GO_FORWARD });
        },

        setSelectedItems: (items) => {
            dispatch({ type: ActionTypes.SET_SELECTED_ITEMS, payload: items });
        },

        addSelectedItem: (item) => {
            dispatch({ type: ActionTypes.ADD_SELECTED_ITEM, payload: item });
        },

        removeSelectedItem: (item) => {
            dispatch({ type: ActionTypes.REMOVE_SELECTED_ITEM, payload: item });
        },

        clearSelectedItems: () => {
            dispatch({ type: ActionTypes.CLEAR_SELECTED_ITEMS });
        },

        setViewMode: (mode) => {
            dispatch({ type: ActionTypes.SET_VIEW_MODE, payload: mode });
        },

        toggleDetailPanel: (isOpen) => {
            dispatch({ type: ActionTypes.TOGGLE_DETAIL_PANEL, payload: isOpen });
        },

        toggleTerminalPanel: (isOpen) => {
            dispatch({ type: ActionTypes.TOGGLE_TERMINAL_PANEL, payload: isOpen });
        },

        setSearchQuery: (query) => {
            dispatch({ type: ActionTypes.SET_SEARCH_QUERY, payload: query });
        },

        setSearchResults: (results) => {
            dispatch({ type: ActionTypes.SET_SEARCH_RESULTS, payload: results });
        },

        clearSearch: () => {
            dispatch({ type: ActionTypes.CLEAR_SEARCH });
        },

        // Explizite Funktion zum Hinzufügen zu den letzten Pfaden
        addRecent: (path) => {
            dispatch({ type: ActionTypes.ADD_RECENT, payload: path });
        },

        addFavorite: (path) => {
            dispatch({ type: ActionTypes.ADD_FAVORITE, payload: path });

            // [Backend Integration] - Favoriten im Backend speichern
            // /* BACKEND_INTEGRATION: Favoriten im Backend speichern */
        },

        removeFavorite: (path) => {
            dispatch({ type: ActionTypes.REMOVE_FAVORITE, payload: path });

            // [Backend Integration] - Favoriten im Backend aktualisieren
            // /* BACKEND_INTEGRATION: Favoriten im Backend aktualisieren */
        },

        setSortBy: (sortBy) => {
            dispatch({ type: ActionTypes.SET_SORT_BY, payload: sortBy });
        },

        setSortDirection: (direction) => {
            dispatch({ type: ActionTypes.SET_SORT_DIRECTION, payload: direction });
        },

        setLoading: (isLoading) => {
            dispatch({ type: ActionTypes.SET_LOADING, payload: isLoading });
        },

        setError: (error) => {
            dispatch({ type: ActionTypes.SET_ERROR, payload: error });
        },

        toggleSidebar: (isOpen) => {
            dispatch({ type: ActionTypes.TOGGLE_SIDEBAR, payload: isOpen });
        },

        // Dateiverwaltungsfunktionen
        createFile: async (path, name) => {
            // [Backend Integration] - Datei im Backend erstellen
            // /* BACKEND_INTEGRATION: Datei erstellen */
            console.log(`Creating file ${name} at ${path}`);
            return { success: true, path: `${path}/${name}` };
        },

        createFolder: async (path, name) => {
            // [Backend Integration] - Ordner im Backend erstellen
            // /* BACKEND_INTEGRATION: Ordner erstellen */
            console.log(`Creating folder ${name} at ${path}`);
            return { success: true, path: `${path}/${name}` };
        },

        deleteItems: async (items) => {
            // [Backend Integration] - Elemente im Backend löschen
            // /* BACKEND_INTEGRATION: Elemente löschen */
            console.log(`Deleting items: ${items.join(', ')}`);
            return { success: true };
        },

        renameItem: async (oldPath, newName) => {
            // [Backend Integration] - Element im Backend umbenennen
            // /* BACKEND_INTEGRATION: Element umbenennen */
            console.log(`Renaming ${oldPath} to ${newName}`);

            // Pfad aufteilen und letztes Element mit neuem Namen ersetzen
            const pathParts = oldPath.split('/');
            pathParts.pop();
            const newPath = [...pathParts, newName].join('/');

            return { success: true, oldPath, newPath };
        },

        copyItems: async (items, destination) => {
            // [Backend Integration] - Elemente im Backend kopieren
            // /* BACKEND_INTEGRATION: Elemente kopieren */
            console.log(`Copying items: ${items.join(', ')} to ${destination}`);
            return { success: true };
        },

        moveItems: async (items, destination) => {
            // [Backend Integration] - Elemente im Backend verschieben
            // /* BACKEND_INTEGRATION: Elemente verschieben */
            console.log(`Moving items: ${items.join(', ')} to ${destination}`);
            return { success: true };
        },

        getItemProperties: async (path) => {
            // [Backend Integration] - Eigenschaften eines Elements im Backend abrufen
            // /* BACKEND_INTEGRATION: Eigenschaften abrufen */
            console.log(`Getting properties for ${path}`);

            // Beispieldaten
            return {
                name: path.split('/').pop(),
                path: path,
                type: path.includes('.') ? 'file' : 'directory',
                size: '1.2 MB',
                created: new Date().toISOString(),
                modified: new Date().toISOString(),
                accessed: new Date().toISOString(),
            };
        },

        search: async (query, path = null, recursive = true) => {
            // [Backend Integration] - Im Backend suchen
            // /* BACKEND_INTEGRATION: Suche durchführen */
            console.log(`Searching for "${query}" in ${path || 'all locations'}, recursive: ${recursive}`);

            dispatch({ type: ActionTypes.SET_SEARCH_QUERY, payload: query });
            dispatch({ type: ActionTypes.SET_LOADING, payload: true });

            // Simulierte Suchergebnisse
            setTimeout(() => {
                const results = [
                    { name: `sample-${query}.txt`, path: `/path/to/sample-${query}.txt`, type: 'file' },
                    { name: `example-${query}.docx`, path: `/path/to/example-${query}.docx`, type: 'file' },
                    { name: query, path: `/path/to/${query}`, type: 'directory' },
                ];

                dispatch({ type: ActionTypes.SET_SEARCH_RESULTS, payload: results });
                dispatch({ type: ActionTypes.SET_LOADING, payload: false });
            }, 500);
        },

        saveTemplate: async (path, templateName) => {
            // [Backend Integration] - Template im Backend speichern
            // /* BACKEND_INTEGRATION: Template speichern */
            console.log(`Saving template "${templateName}" for ${path}`);
            return { success: true, templateName };
        },
    };

    return (
        <AppStateContext.Provider value={{ state, actions }}>
            {children}
        </AppStateContext.Provider>
    );
};

// Custom Hook für einfachen Zugriff auf den AppState-Kontext
export const useAppState = () => {
    const context = useContext(AppStateContext);
    if (context === undefined) {
        throw new Error('useAppState must be used within an AppStateProvider');
    }
    return context;
};

export default AppStateProvider;