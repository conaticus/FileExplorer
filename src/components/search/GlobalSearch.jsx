import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import SearchBar from '../search/SearchBar';
import EmptyState from '../explorer/EmptyState';
import './search.css';

const GlobalSearch = ({ isOpen, onClose }) => {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState([]);
    const [isSearching, setIsSearching] = useState(false);
    const [searchPath, setSearchPath] = useState('');
    const [searchOptions, setSearchOptions] = useState({
        caseSensitive: false,
        wholeWords: false,
        includeHidden: false,
        fileTypes: []
    });
    const [searchProgress, setSearchProgress] = useState(null);

    const { navigateTo } = useHistory();
    const { loadDirectory, volumes } = useFileSystem();

    // Initialize search path with first volume
    useEffect(() => {
        if (volumes.length > 0 && !searchPath) {
            setSearchPath(volumes[0].mount_point);
        }
    }, [volumes, searchPath]);

    // Perform search
    const performSearch = async () => {
        if (!query.trim() || !searchPath) return;

        setIsSearching(true);
        setResults([]);
        setSearchProgress({ current: 0, total: 0, currentPath: '' });

        try {
            // Add search path to index if not already added
            try {
                await invoke('add_paths_recursive', { folder: searchPath });
            } catch (error) {
                console.warn('Path might already be indexed:', error);
            }

            // Start search
            const searchResults = await invoke('search', {
                query: query.trim()
            });

            // Process results
            const processedResults = searchResults.map(([path, score]) => ({
                path,
                score,
                name: path.split(/[/\\]/).pop(),
                directory: path.split(/[/\\]/).slice(0, -1).join('/')
            }));

            setResults(processedResults);
            setSearchProgress(null);
        } catch (error) {
            console.error('Search failed:', error);
            alert(`Search failed: ${error.message || error}`);
        } finally {
            setIsSearching(false);
        }
    };

    // Clear search
    const clearSearch = () => {
        setQuery('');
        setResults([]);
        setSearchProgress(null);
    };

    // Open file/folder
    const openItem = async (result) => {
        try {
            // Check if it's a directory by trying to open it
            try {
                await loadDirectory(result.directory);
                navigateTo(result.directory);
                onClose();
            } catch {
                // If loading directory fails, assume it's a file
                await invoke('open_file', { file_path: result.path });
            }
        } catch (error) {
            console.error('Failed to open item:', error);
            alert(`Failed to open: ${error.message || error}`);
        }
    };

    // Open containing folder
    const openContainingFolder = async (result) => {
        try {
            await loadDirectory(result.directory);
            navigateTo(result.directory);
            onClose();
        } catch (error) {
            console.error('Failed to open folder:', error);
            alert(`Failed to open folder: ${error.message || error}`);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="search-overlay">
            <div className="search-modal">
                <div className="search-header">
                    <h2>Global Search</h2>
                    <button
                        className="close-button"
                        onClick={onClose}
                        aria-label="Close search"
                    >
                        <span className="icon icon-x"></span>
                    </button>
                </div>

                <div className="search-form">
                    <div className="search-input-container">
                        <SearchBar
                            value={query}
                            onChange={setQuery}
                            onSubmit={performSearch}
                            placeholder="Search files and folders..."
                        />
                        <button
                            className="search-button"
                            onClick={performSearch}
                            disabled={isSearching || !query.trim()}
                        >
                            {isSearching ? 'Searching...' : 'Search'}
                        </button>
                    </div>

                    <div className="search-options">
                        <div className="search-path">
                            <label>Search in:</label>
                            <select
                                value={searchPath}
                                onChange={(e) => setSearchPath(e.target.value)}
                                className="path-select"
                            >
                                {volumes.map(volume => (
                                    <option key={volume.mount_point} value={volume.mount_point}>
                                        {volume.volume_name} ({volume.mount_point})
                                    </option>
                                ))}
                            </select>
                        </div>

                        <div className="search-filters">
                            <label className="filter-option">
                                <input
                                    type="checkbox"
                                    checked={searchOptions.caseSensitive}
                                    onChange={(e) => setSearchOptions(prev => ({
                                        ...prev,
                                        caseSensitive: e.target.checked
                                    }))}
                                />
                                <span>Case sensitive</span>
                            </label>

                            <label className="filter-option">
                                <input
                                    type="checkbox"
                                    checked={searchOptions.wholeWords}
                                    onChange={(e) => setSearchOptions(prev => ({
                                        ...prev,
                                        wholeWords: e.target.checked
                                    }))}
                                />
                                <span>Whole words</span>
                            </label>

                            <label className="filter-option">
                                <input
                                    type="checkbox"
                                    checked={searchOptions.includeHidden}
                                    onChange={(e) => setSearchOptions(prev => ({
                                        ...prev,
                                        includeHidden: e.target.checked
                                    }))}
                                />
                                <span>Include hidden</span>
                            </label>
                        </div>
                    </div>
                </div>

                <div className="search-results">
                    {isSearching && (
                        <div className="search-progress">
                            <div className="progress-spinner"></div>
                            <span>Searching...</span>
                            {searchProgress && (
                                <div className="progress-details">
                                    <div>Searching in: {searchProgress.currentPath}</div>
                                </div>
                            )}
                        </div>
                    )}

                    {!isSearching && results.length === 0 && query && (
                        <EmptyState
                            type="no-results"
                            searchTerm={query}
                        />
                    )}

                    {!isSearching && results.length > 0 && (
                        <div className="results-container">
                            <div className="results-header">
                                <span>{results.length} results found for "{query}"</span>
                                <button className="clear-button" onClick={clearSearch}>
                                    Clear
                                </button>
                            </div>

                            <div className="results-list">
                                {results.map((result, index) => (
                                    <div key={index} className="result-item">
                                        <div className="result-icon">
                                            <span className="icon icon-file"></span>
                                        </div>

                                        <div className="result-details">
                                            <div className="result-name" onClick={() => openItem(result)}>
                                                {result.name}
                                            </div>
                                            <div className="result-path" onClick={() => openContainingFolder(result)}>
                                                {result.directory}
                                            </div>
                                        </div>

                                        <div className="result-score">
                                            {Math.round(result.score * 100)}%
                                        </div>

                                        <div className="result-actions">
                                            <button
                                                className="action-button"
                                                onClick={() => openContainingFolder(result)}
                                                title="Open containing folder"
                                            >
                                                <span className="icon icon-folder"></span>
                                            </button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default GlobalSearch;