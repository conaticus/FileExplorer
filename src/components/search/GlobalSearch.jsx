import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import SearchBar from '../search/SearchBar';
import EmptyState from '../explorer/EmptyState';
import Modal from '../common/Modal';
import Button from '../common/Button';
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

        try {
            // Mock search results for demonstration
            await new Promise(resolve => setTimeout(resolve, 1000));

            const mockResults = [
                {
                    path: `${searchPath}/Documents/file1.txt`,
                    name: 'file1.txt',
                    directory: `${searchPath}/Documents`,
                    score: 0.95
                },
                {
                    path: `${searchPath}/Pictures/image.jpg`,
                    name: 'image.jpg',
                    directory: `${searchPath}/Pictures`,
                    score: 0.85
                }
            ].filter(item =>
                item.name.toLowerCase().includes(query.toLowerCase())
            );

            setResults(mockResults);
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
    };

    // Open file/folder
    const openItem = async (result) => {
        try {
            await loadDirectory(result.directory);
            navigateTo(result.directory);
            onClose();
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

    const handleSubmit = (e) => {
        e.preventDefault();
        performSearch();
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Global Search"
            size="lg"
        >
            <div className="global-search-content">
                <form onSubmit={handleSubmit} className="search-form-container">
                    <div className="search-input-row">
                        <div className="search-input-wrapper">
                            <input
                                type="text"
                                className="search-input-field"
                                value={query}
                                onChange={(e) => setQuery(e.target.value)}
                                placeholder="Search files and folders..."
                                disabled={isSearching}
                            />
                        </div>
                        <Button
                            type="submit"
                            variant="primary"
                            disabled={isSearching || !query.trim()}
                        >
                            {isSearching ? 'Searching...' : 'Search'}
                        </Button>
                    </div>

                    <div className="search-options-container">
                        <div className="search-path-container">
                            <label htmlFor="search-path">Search in:</label>
                            <select
                                id="search-path"
                                value={searchPath}
                                onChange={(e) => setSearchPath(e.target.value)}
                                className="path-select-field"
                            >
                                {volumes.map(volume => (
                                    <option key={volume.mount_point} value={volume.mount_point}>
                                        {volume.volume_name || volume.mount_point} ({volume.mount_point})
                                    </option>
                                ))}
                            </select>
                        </div>

                        <div className="search-filters-container">
                            <label className="checkbox-option">
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

                            <label className="checkbox-option">
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

                            <label className="checkbox-option">
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
                </form>

                <div className="search-results-container">
                    {isSearching && (
                        <div className="search-progress-container">
                            <div className="progress-spinner"></div>
                            <span>Searching...</span>
                        </div>
                    )}

                    {!isSearching && results.length === 0 && query && (
                        <div className="no-results-container">
                            <EmptyState
                                type="no-results"
                                searchTerm={query}
                            />
                        </div>
                    )}

                    {!isSearching && results.length > 0 && (
                        <div className="results-list-container">
                            <div className="results-header-container">
                                <span>{results.length} results found for "{query}"</span>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={clearSearch}
                                >
                                    Clear
                                </Button>
                            </div>

                            <div className="results-items-container">
                                {results.map((result, index) => (
                                    <div key={index} className="result-item-container">
                                        <div className="result-icon-container">
                                            <span className="icon icon-file"></span>
                                        </div>

                                        <div className="result-details-container">
                                            <div
                                                className="result-name-container"
                                                onClick={() => openItem(result)}
                                            >
                                                {result.name}
                                            </div>
                                            <div
                                                className="result-path-container"
                                                onClick={() => openContainingFolder(result)}
                                            >
                                                {result.directory}
                                            </div>
                                        </div>

                                        <div className="result-score-container">
                                            {Math.round(result.score * 100)}%
                                        </div>

                                        <div className="result-actions-container">
                                            <button
                                                className="action-button-container"
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
        </Modal>
    );
};

export default GlobalSearch;