import { useState, useCallback, useEffect } from 'react';
import { searchFiles, parseSearchQuery, DEFAULT_SEARCH_OPTIONS } from '../utils/search';
import { useHistory } from '../providers/HistoryProvider';

/**
 * Hook for handling file search functionality.
 * @param {Object} initialOptions - Initial search options.
 * @returns {Object} Search state and functions.
 */
const useSearch = (initialOptions = DEFAULT_SEARCH_OPTIONS) => {
    const [query, setQuery] = useState('');
    const [options, setOptions] = useState({ ...initialOptions });
    const [results, setResults] = useState(null);
    const [isSearching, setIsSearching] = useState(false);
    const [error, setError] = useState(null);
    const { currentPath } = useHistory();

    // Update searchIn option when current path changes
    useEffect(() => {
        if (currentPath) {
            setOptions(prev => ({
                ...prev,
                searchIn: currentPath,
            }));
        }
    }, [currentPath]);

    // Parse search query and update options
    const updateQuery = useCallback((newQuery) => {
        setQuery(newQuery);

        if (!newQuery.trim()) {
            setResults(null);
            return;
        }

        // Parse the query to extract special tokens
        const parsed = parseSearchQuery(newQuery);

        // Update options without changing searchIn
        setOptions(prev => ({
            ...prev,
            ...parsed.options,
            searchIn: prev.searchIn,
        }));
    }, []);

    // Perform search with current query and options
    const performSearch = useCallback(async () => {
        if (!query.trim()) {
            setResults(null);
            return;
        }

        setIsSearching(true);
        setError(null);

        try {
            const searchResults = await searchFiles(query, options);
            setResults(searchResults);
        } catch (err) {
            console.error('Search failed:', err);
            setError(err.message || 'Search failed');
            setResults(null);
        } finally {
            setIsSearching(false);
        }
    }, [query, options]);

    // Clear search results
    const clearSearch = useCallback(() => {
        setQuery('');
        setResults(null);
        setError(null);
    }, []);

    // Update a single search option
    const updateOption = useCallback((key, value) => {
        setOptions(prev => ({
            ...prev,
            [key]: value,
        }));
    }, []);

    return {
        query,
        options,
        results,
        isSearching,
        error,
        updateQuery,
        performSearch,
        clearSearch,
        updateOption,
    };
};

export default useSearch;