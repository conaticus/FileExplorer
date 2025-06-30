import React, { useState, useEffect, useRef } from 'react';
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
    const [searchEngineInfo, setSearchEngineInfo] = useState(null);
    const [selectedExtensions, setSelectedExtensions] = useState([]);
    const [isIndexing, setIsIndexing] = useState(false);
    const [indexingProgress, setIndexingProgress] = useState({
        files_indexed: 0,
        files_discovered: 0,
        percentage_complete: 0.0,
        current_path: null,
        estimated_time_remaining: null,
        start_time: null
    });
    const progressIntervalRef = useRef(null);
    const [filtersExpanded, setFiltersExpanded] = useState(false);
    const [statsExpanded, setStatsExpanded] = useState(false);

    const { navigateTo } = useHistory();
    const { loadDirectory, volumes } = useFileSystem();

    // Common file extensions for filtering
    const commonExtensions = [
        { value: 'txt', label: 'Text Files (.txt)' },
        { value: 'pdf', label: 'PDF Files (.pdf)' },
        { value: 'doc', label: 'Word Documents (.doc)' },
        { value: 'docx', label: 'Word Documents (.docx)' },
        { value: 'jpg', label: 'JPEG Images (.jpg)' },
        { value: 'png', label: 'PNG Images (.png)' },
        { value: 'mp3', label: 'MP3 Audio (.mp3)' },
        { value: 'mp4', label: 'MP4 Video (.mp4)' },
        { value: 'zip', label: 'ZIP Archives (.zip)' },
        { value: 'js', label: 'JavaScript (.js)' },
        { value: 'css', label: 'CSS Files (.css)' },
        { value: 'html', label: 'HTML Files (.html)' }
    ];

    // Load search engine info when modal opens
    useEffect(() => {
        if (isOpen) {
            loadSearchEngineInfo();
        }
    }, [isOpen]);

    // Auto-index on app start if search engine is empty
    useEffect(() => {
        const initializeSearchEngine = async () => {
            if (volumes.length > 0 && searchEngineInfo) {
                console.log('Initializing search engine with volumes:', volumes);
                console.log('Search engine info:', searchEngineInfo);

                // Check if search engine has no indexed files
                const hasNoIndexedFiles = !searchEngineInfo.stats?.trie_size || searchEngineInfo.stats.trie_size === 0;

                console.log('Has no indexed files:', hasNoIndexedFiles, 'Is indexing:', isIndexing);

                if (hasNoIndexedFiles && !isIndexing) {
                    console.log('Search engine is empty, starting auto-indexing...');
                    await startAutoIndexing();
                } else {
                    console.log('Skipping auto-indexing - either has files or already indexing');
                }
            } else {
                console.log('Not initializing search engine - volumes:', volumes.length, 'searchEngineInfo:', !!searchEngineInfo);
            }
        };

        initializeSearchEngine();
    }, [volumes, searchEngineInfo, isIndexing]);

    // Cleanup polling on unmount
    useEffect(() => {
        return () => {
            stopProgressPolling();
        };
    }, []);

    // Start polling for progress when indexing begins
    const startProgressPolling = () => {
        if (progressIntervalRef.current) return; // Already polling

        console.log('Starting progress polling...');

        progressIntervalRef.current = setInterval(async () => {
            try {
                console.log('Polling for progress...');
                const [status, progress] = await Promise.all([
                    invoke('get_indexing_status'),
                    invoke('get_indexing_progress')
                ]);

                console.log('Progress poll result:', {
                    status,
                    progress: {
                        files_indexed: progress.files_indexed,
                        files_discovered: progress.files_discovered,
                        percentage: progress.percentage_complete,
                        current_path: progress.current_path ? progress.current_path.split('/').pop() : null
                    }
                });

                setIndexingProgress(progress);

                // Check if indexing is complete
                if (status !== 'Indexing') {
                    console.log('Indexing completed, stopping polling. Final status:', status);
                    setIsIndexing(false);
                    stopProgressPolling();
                    // Reload search engine info after indexing completes
                    await loadSearchEngineInfo();
                }
            } catch (error) {
                console.error('Error polling progress:', error);
                // Don't stop polling on error, just log it
            }
        }, 200); // Reduced polling interval for more responsive updates
    };

    const stopProgressPolling = () => {
        if (progressIntervalRef.current) {
            clearInterval(progressIntervalRef.current);
            progressIntervalRef.current = null;
        }
    };

    // Format time remaining
    const formatTimeRemaining = (ms) => {
        if (!ms) return 'Calculating...';

        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);

        if (hours > 0) {
            return `${hours}h ${minutes % 60}m`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        } else {
            return `${seconds}s`;
        }
    };

    // Load search engine information
    const loadSearchEngineInfo = async () => {
        try {
            console.log('Loading search engine info...');
            const info = await invoke('get_search_engine_info');
            console.log('Search engine info loaded:', info);
            setSearchEngineInfo(info);
        } catch (error) {
            console.error('Failed to load search engine info:', error);
            setSearchEngineInfo(null);
        }
    };

    // Perform search using the real API
    const performSearch = async () => {
        if (!query.trim()) return;

        setIsSearching(true);
        setResults([]);

        try {
            let searchResults;

            // Use extension filtering if extensions are selected
            if (selectedExtensions.length > 0) {
                searchResults = await invoke('search_with_extension', {
                    query: query.trim(),
                    extensions: selectedExtensions
                });
            } else {
                // Use basic search
                searchResults = await invoke('search', {
                    query: query.trim()
                });
            }

            // Convert API results to our format
            // API returns [[path, score], [path, score], ...]
            const formattedResults = searchResults.map(([path, score]) => {
                const fileName = path.split(/[/\\]/).pop() || path;
                const directory = path.substring(0, path.lastIndexOf(fileName) - 1) || '/';

                return {
                    path,
                    name: fileName,
                    directory,
                    score
                };
            });

            setResults(formattedResults);
        } catch (error) {
            console.error('Search failed:', error);
            // Show user-friendly error message
            const errorMessage = error.message || error;
            if (errorMessage.includes('No search engine available')) {
                alert('Search engine is not ready. Please ensure the backend has indexed your files.');
            } else {
                alert(`Search failed: ${errorMessage}`);
            }
        } finally {
            setIsSearching(false);
        }
    };

    // Clear search
    const clearSearch = () => {
        setQuery('');
        setResults([]);
    };

    // Handle extension selection
    const handleExtensionChange = (extension) => {
        setSelectedExtensions(prev => {
            if (prev.includes(extension)) {
                return prev.filter(ext => ext !== extension);
            } else {
                return [...prev, extension];
            }
        });
    };

    // Open file/folder location
    const openItemLocation = async (result) => {
        try {
            await loadDirectory(result.directory);
            navigateTo(result.directory);
            onClose();
        } catch (error) {
            console.error('Failed to open item location:', error);
            alert(`Failed to open location: ${error.message || error}`);
        }
    };

    // Auto-start indexing for volumes
    const startAutoIndexing = async () => {
        if (volumes.length === 0 || isIndexing) {
            console.log('Skipping auto-indexing - no volumes or already indexing');
            return;
        }

        console.log('Starting auto-indexing for volumes:', volumes);

        setIsIndexing(true);
        setIndexingProgress({
            files_indexed: 0,
            files_discovered: 0,
            percentage_complete: 0.0,
            current_path: null,
            estimated_time_remaining: null,
            start_time: Date.now()
        });

        // Start polling before starting indexing
        startProgressPolling();

        try {
            console.log('Starting background indexing for volumes...');

            // Index all volumes in background
            for (const volume of volumes) {
                console.log(`Adding ${volume.mount_point} to search index...`);

                const result = await invoke('add_paths_recursive_async', {
                    folder: volume.mount_point
                });

                console.log(`Indexing result for ${volume.mount_point}:`, result);
            }

            console.log('Auto-indexing initiated successfully');

        } catch (error) {
            console.error('Auto-indexing failed:', error);
            setIsIndexing(false);
            stopProgressPolling();

            // Show error to user
            alert(`Failed to start indexing: ${error.message || error}`);
        }
    };

    // Manual indexing trigger
    const startManualIndexing = async () => {
        if (volumes.length === 0) {
            alert('No volumes available for indexing');
            return;
        }

        console.log('Starting manual indexing for volumes:', volumes);

        setIsIndexing(true);
        setIndexingProgress({
            files_indexed: 0,
            files_discovered: 0,
            percentage_complete: 0.0,
            current_path: null,
            estimated_time_remaining: null,
            start_time: Date.now()
        });

        // Start polling before starting indexing
        startProgressPolling();

        try {
            for (const volume of volumes) {
                console.log(`Manually indexing ${volume.mount_point}...`);

                const result = await invoke('add_paths_recursive_async', {
                    folder: volume.mount_point
                });

                console.log(`Manual indexing result for ${volume.mount_point}:`, result);
            }

            console.log('Manual indexing initiated successfully');
            alert('Indexing started successfully. This may take some time to complete.');

        } catch (error) {
            console.error('Manual indexing failed:', error);
            alert(`Failed to start indexing: ${error.message || error}`);
            setIsIndexing(false);
            stopProgressPolling();
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

                    {/* Search Engine Status */}
                    {searchEngineInfo && (
                        <div className="search-engine-status">
                            <div className="status-info">
                                <span className="status-label">Status:</span>
                                <span className={`status-value ${searchEngineInfo.status?.toLowerCase()}`}>
                                    {searchEngineInfo.status || 'Unknown'}
                                </span>
                            </div>
                            {searchEngineInfo.stats && (
                                <div className="status-info">
                                    <span className="status-label">Indexed files:</span>
                                    <span className="status-value">
                                        {searchEngineInfo.stats.trie_size || 0} entries
                                    </span>
                                </div>
                            )}
                            {isIndexing && (
                                <div className="status-info">
                                    <span className="status-label">Indexing:</span>
                                    <span className="status-value indexing">In Progress...</span>
                                </div>
                            )}
                        </div>
                    )}

                    {/* Indexing Progress UI */}
                    {isIndexing && (
                        <div className="indexing-progress">
                            <div className="progress-header">
                                <h3>Indexing Files...</h3>
                                <span className="progress-percentage">
                                    {indexingProgress.percentage_complete.toFixed(1)}%
                                </span>
                            </div>

                            <div className="progress-bar">
                                <div
                                    className="progress-fill"
                                    style={{ width: `${indexingProgress.percentage_complete}%` }}
                                />
                            </div>

                            <div className="progress-details">
                                <div className="progress-stats">
                                    <span>
                                        {indexingProgress.files_indexed} / {indexingProgress.files_discovered} files
                                    </span>
                                    {indexingProgress.estimated_time_remaining && (
                                        <span>
                                            {formatTimeRemaining(indexingProgress.estimated_time_remaining)} remaining
                                        </span>
                                    )}
                                </div>

                                {indexingProgress.current_path && (
                                    <div className="current-file">
                                        <span className="current-file-label">Processing:</span>
                                        <span className="current-file-path" title={indexingProgress.current_path}>
                                            {indexingProgress.current_path.split('/').pop() || indexingProgress.current_path}
                                        </span>
                                    </div>
                                )}
                            </div>

                            <button
                                onClick={async () => {
                                    try {
                                        await invoke('stop_indexing');
                                        setIsIndexing(false);
                                        stopProgressPolling();
                                        loadSearchEngineInfo(); // Refresh info after stopping
                                    } catch (error) {
                                        console.error('Failed to stop indexing:', error);
                                    }
                                }}
                                className="stop-indexing-btn"
                            >
                                Stop Indexing
                            </button>
                        </div>
                    )}

                    {/* Accordions Container */}
                    <div className="accordions-container">
                        {/* Extension Filters Accordion */}
                        <div className="accordion">
                            <button
                                className="accordion-header"
                                onClick={() => setFiltersExpanded(!filtersExpanded)}
                                type="button"
                            >
                                <span>File Type Filters</span>
                                <span className={`accordion-chevron ${filtersExpanded ? 'expanded' : ''}`}>▼</span>
                            </button>

                            {filtersExpanded && (
                                <div className="accordion-content">
                                    <div className="extension-filters">
                                        <div className="extension-checkboxes">
                                            {commonExtensions.map(ext => (
                                                <label key={ext.value} className="checkbox-option">
                                                    <input
                                                        type="checkbox"
                                                        checked={selectedExtensions.includes(ext.value)}
                                                        onChange={() => handleExtensionChange(ext.value)}
                                                        disabled={isSearching}
                                                    />
                                                    <span>{ext.label}</span>
                                                </label>
                                            ))}
                                        </div>
                                        {selectedExtensions.length > 0 && (
                                            <div className="selected-extensions">
                                                Selected: {selectedExtensions.join(', ')}
                                                <button
                                                    type="button"
                                                    onClick={() => setSelectedExtensions([])}
                                                    className="clear-extensions"
                                                    disabled={isSearching}
                                                >
                                                    Clear
                                                </button>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            )}
                        </div>

                        {/* Search Engine Statistics Accordion */}
                        {searchEngineInfo && (
                            <div className="accordion">
                                <button
                                    className="accordion-header"
                                    onClick={() => setStatsExpanded(!statsExpanded)}
                                    type="button"
                                >
                                    <span>Search Engine Statistics</span>
                                    <span className={`accordion-chevron ${statsExpanded ? 'expanded' : ''}`}>▼</span>
                                </button>

                                {statsExpanded && (
                                    <div className="accordion-content">
                                        <div className="search-engine-info-compact">
                                            <div className="info-grid">
                                                <div className="info-item">
                                                    <span className="info-label">Status:</span>
                                                    <span className="info-value">{searchEngineInfo.status || 'Unknown'}</span>
                                                </div>
                                                {searchEngineInfo.progress && (
                                                    <>
                                                        <div className="info-item">
                                                            <span className="info-label">Progress:</span>
                                                            <span className="info-value">
                                                                {searchEngineInfo.progress.percentage_complete || 0}%
                                                            </span>
                                                        </div>
                                                        <div className="info-item">
                                                            <span className="info-label">Files indexed:</span>
                                                            <span className="info-value">
                                                                {searchEngineInfo.progress.files_indexed || 0} / {searchEngineInfo.progress.files_discovered || 0}
                                                            </span>
                                                        </div>
                                                    </>
                                                )}
                                                {searchEngineInfo.metrics && (
                                                    <>
                                                        <div className="info-item">
                                                            <span className="info-label">Total searches:</span>
                                                            <span className="info-value">{searchEngineInfo.metrics.total_searches || 0}</span>
                                                        </div>
                                                        <div className="info-item">
                                                            <span className="info-label">Avg search time:</span>
                                                            <span className="info-value">{searchEngineInfo.metrics.average_search_time_ms || 0}ms</span>
                                                        </div>
                                                    </>
                                                )}
                                            </div>

                                            {/* Manual Indexing Control */}
                                            <div className="index-management">
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    onClick={startManualIndexing}
                                                    disabled={isSearching || isIndexing}
                                                >
                                                    {isIndexing ? 'Indexing...' : 'Re-index All Directories'}
                                                </Button>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </form>

                <div className="search-results-container">
                    {isSearching && (
                        <div className="search-progress-container">
                            <div className="progress-spinner"></div>
                            <span>Searching indexed files...</span>
                        </div>
                    )}

                    {!isSearching && results.length === 0 && query && (
                        <div className="no-results-container">
                            <EmptyState
                                type="no-results"
                                searchTerm={query}
                            />
                            <div className="no-results-help">
                                <p>Tips:</p>
                                <ul>
                                    <li>Make sure the backend has indexed your files</li>
                                    <li>Try different keywords</li>
                                    <li>Check spelling</li>
                                    <li>Try using fewer or more specific terms</li>
                                </ul>
                            </div>
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
                                                onClick={() => openItemLocation(result)}
                                                title={`Open location: ${result.directory}`}
                                            >
                                                {result.name}
                                            </div>
                                            <div
                                                className="result-path-container"
                                                onClick={() => openItemLocation(result)}
                                                title={result.path}
                                            >
                                                {result.path}
                                            </div>
                                        </div>

                                        <div className="result-score-container" title="Relevance score">
                                            {Math.round(result.score * 100)}%
                                        </div>

                                        <div className="result-actions-container">
                                            <button
                                                className="action-button-container"
                                                onClick={() => openItemLocation(result)}
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
