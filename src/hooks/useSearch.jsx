import { useState, useCallback, useEffect } from 'react';
import { useAppState } from '../providers/AppStateProvider';
import { useFileSystem } from '../providers/FileSystemProvider';

/**
 * Hook für die Suche nach Dateien und Ordnern
 *
 * @param {Object} options - Suchoptionen
 * @param {boolean} [options.autoSearch=false] - Automatisch bei Änderung der Suchanfrage suchen
 * @param {number} [options.debounceTime=500] - Verzögerung in ms für die automatische Suche
 * @returns {Object} Suchfunktionen und -zustand
 */
const useSearch = ({
                       autoSearch = false,
                       debounceTime = 500
                   } = {}) => {
    const { state, actions } = useAppState();
    const { searchFiles } = useFileSystem();

    const [query, setQuery] = useState(state.searchQuery || '');
    const [results, setResults] = useState(state.searchResults || []);
    const [isSearching, setIsSearching] = useState(false);
    const [searchOptions, setSearchOptions] = useState({
        recursive: true,
        caseSensitive: false,
        matchWholeWord: false,
        searchContents: false,
        fileTypes: [],
        location: 'current' // 'current', 'all', oder ein Pfad
    });

    // Aktualisiere die Suchergebnisse, wenn sich der globale Zustand ändert
    useEffect(() => {
        setResults(state.searchResults || []);
    }, [state.searchResults]);

    // Aktualisiere die Suchanfrage, wenn sich der globale Zustand ändert
    useEffect(() => {
        setQuery(state.searchQuery || '');
    }, [state.searchQuery]);

    // Führe die Suche durch
    const search = useCallback(async (searchQuery = query, searchPath = null, options = searchOptions) => {
        if (!searchQuery.trim()) {
            clearSearch();
            return [];
        }

        setIsSearching(true);
        actions.setLoading(true);

        try {
            // Bestimme den Suchpfad
            let path = searchPath;
            if (!path) {
                if (options.location === 'current') {
                    path = state.currentPath;
                } else if (options.location === 'all') {
                    path = null; // Suche überall
                } else {
                    path = options.location; // Benutzerdefinierter Pfad
                }
            }

            // Aktualisiere die Suchanfrage im globalen Zustand
            actions.setSearchQuery(searchQuery);

            // [Backend Integration] - Suche im Backend durchführen
            // /* BACKEND_INTEGRATION: Suche durchführen */

            // Führe die Suche durch
            const searchResults = await searchFiles(
                searchQuery,
                path,
                options.recursive,
                {
                    caseSensitive: options.caseSensitive,
                    matchWholeWord: options.matchWholeWord,
                    searchContents: options.searchContents,
                    fileTypes: options.fileTypes
                }
            );

            // Aktualisiere die Suchergebnisse
            setResults(searchResults);
            actions.setSearchResults(searchResults);

            return searchResults;
        } catch (error) {
            console.error('Error during search:', error);
            actions.setError(`Fehler bei der Suche: ${error.message}`);
            return [];
        } finally {
            setIsSearching(false);
            actions.setLoading(false);
        }
    }, [query, searchOptions, state.currentPath, searchFiles, actions, clearSearch]);

    // Lösche die Suche
    function clearSearch() {
        setQuery('');
        setResults([]);
        actions.clearSearch();
    }

    // Automatische Suche mit Verzögerung
    useEffect(() => {
        if (!autoSearch || !query.trim()) return;

        const timer = setTimeout(() => {
            search();
        }, debounceTime);

        return () => clearTimeout(timer);
    }, [query, autoSearch, debounceTime, search]);

    // Aktualisiere die Suchoptionen
    const updateSearchOptions = useCallback((newOptions) => {
        setSearchOptions(prev => ({ ...prev, ...newOptions }));
    }, []);

    return {
        query,
        results,
        isSearching,
        isSearchActive: state.isSearchActive,
        searchOptions,
        setQuery,
        search,
        clearSearch,
        updateSearchOptions
    };
};

export default useSearch;