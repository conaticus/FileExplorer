import React, { useEffect } from 'react';
import FileList from '../explorer/FileList';
import EmptyState from '../explorer/EmptyState';
import useSearch from '../../hooks/useSearch';
import { formatFileSize, formatDate } from '../../utils/formatters';
import './searchResults.css';

const SearchResults = ({ query, viewMode, onClearSearch }) => {
    const {
        results,
        isSearching,
        error,
        updateQuery,
        performSearch,
        options
    } = useSearch();

    // Perform search when query changes
    useEffect(() => {
        if (query) {
            updateQuery(query);
            performSearch();
        }
    }, [query, updateQuery, performSearch]);

    // If loading
    if (isSearching) {
        return (
            <div className="search-results-container">
                <div className="search-header">
                    <h2 className="search-title">
                        Searching for "{query}"
                    </h2>
                    <div className="search-progress">
                        <div className="progress progress-indeterminate">
                            <div className="progress-bar"></div>
                        </div>
                        <p className="search-info">Searching in {options.searchIn || 'current location'}...</p>
                    </div>
                </div>

                <div className="search-loading">
                    <div className="spinner"></div>
                </div>
            </div>
        );
    }

    // If error
    if (error) {
        return (
            <div className="search-results-container">
                <div className="search-header">
                    <h2 className="search-title">
                        Search error
                    </h2>
                    <button
                        className="btn btn-ghost btn-sm"
                        onClick={onClearSearch}
                    >
                        Clear search
                    </button>
                </div>

                <div className="alert alert-error">
                    <div className="alert-icon">
                        <span className="icon icon-alert-circle"></span>
                    </div>
                    <div className="alert-content">
                        <div className="alert-title">Search failed</div>
                        <p className="alert-message">{error}</p>
                    </div>
                </div>
            </div>
        );
    }

    // If no results yet or empty query
    if (!results) {
        return null;
    }

    // If no files or directories found
    if (results.files.length === 0 && results.directories.length === 0) {
        return (
            <div className="search-results-container">
                <div className="search-header">
                    <h2 className="search-title">
                        Search results for "{query}"
                    </h2>
                    <button
                        className="btn btn-ghost btn-sm"
                        onClick={onClearSearch}
                    >
                        Clear search
                    </button>
                </div>

                <EmptyState
                    type="no-results"
                    searchTerm={query}
                />
            </div>
        );
    }

    // Total results count
    const totalResults = results.files.length + results.directories.length;

    return (
        <div className="search-results-container">
            <div className="search-header">
                <h2 className="search-title">
                    Search results for "{query}"
                </h2>
                <div className="search-controls">
          <span className="search-count">
            {totalResults} {totalResults === 1 ? 'result' : 'results'} found
          </span>
                    <button
                        className="btn btn-ghost btn-sm"
                        onClick={onClearSearch}
                    >
                        Clear search
                    </button>
                </div>
            </div>

            {/* Search filter tags (optional) */}
            <div className="search-filters">
                {options.caseSensitive && (
                    <span className="search-filter">Case sensitive</span>
                )}
                {options.matchWholeWord && (
                    <span className="search-filter">Whole word</span>
                )}
                {options.fileTypes.length > 0 && (
                    <span className="search-filter">
            Type: {options.fileTypes.join(', ')}
          </span>
                )}
            </div>

            {/* Results */}
            <FileList
                data={results}
                isLoading={false}
                viewMode={viewMode}
                isSearching={true}
            />
        </div>
    );
};

export default SearchResults;