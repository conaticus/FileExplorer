import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import SearchBar from '../search/SearchBar';
import EmptyState from '../explorer/EmptyState';
import FileIcon from '../explorer/FileIcon';
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
    const searchInputRef = useRef(null);
    const [filtersExpanded, setFiltersExpanded] = useState(false);
    const [statsExpanded, setStatsExpanded] = useState(false);
    const [recentSearches, setRecentSearches] = useState([]);
    const [mostAccessedPaths, setMostAccessedPaths] = useState([]);
    const [searchMetrics, setSearchMetrics] = useState({
        total_searches: 0,
        average_search_time_ms: 0,
        cache_hit_rate: 0,
        cache_hits: 0
    });
    const [currentDirectory, setCurrentDirectory] = useState(null);
    const [sortBy, setSortBy] = useState('relevance'); // relevance, name, date, path
    const [showDirectoriesOnly, setShowDirectoriesOnly] = useState(false);
    const [showHiddenFiles, setShowHiddenFiles] = useState(false);
    const [systemInfo, setSystemInfo] = useState(null);
    const [isLoadingStatus, setIsLoadingStatus] = useState(false);

    const { navigateTo, currentPath } = useHistory();
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
            loadSystemInfo();
            checkIndexingStatus(); // Check if indexing is already in progress
            // Set current directory context from history
            if (currentPath) {
                setCurrentDirectory(currentPath);
            }
            // Focus the search input when modal opens
            setTimeout(() => {
                if (searchInputRef.current) {
                    searchInputRef.current.focus();
                }
            }, 100);
        }
    }, [isOpen, currentPath]);

    // Update current directory when user navigates
    useEffect(() => {
        if (currentPath && currentPath !== currentDirectory) {
            setCurrentDirectory(currentPath);
        }
    }, [currentPath]);

    // Re-sort results when sort criteria changes
    useEffect(() => {
        if (results.length > 0) {
            setResults(prevResults => sortResults(prevResults));
        }
    }, [sortBy, showDirectoriesOnly, showHiddenFiles]);

    // Debounced search - search as you type with delay
    useEffect(() => {
        const timeoutId = setTimeout(() => {
            if (query.trim() && query.length >= 3) {
                performSearch(true); // true indicates this is an automatic search
            } else if (!query.trim()) {
                setResults([]);
            }
        }, 300); // 300ms delay after user stops typing

        return () => clearTimeout(timeoutId);
    }, [query, selectedExtensions, showDirectoriesOnly, showHiddenFiles]);

    // Auto-index on app start if search engine is empty
    useEffect(() => {
        const initializeSearchEngine = async () => {
            if (volumes.length > 0 && searchEngineInfo) {
                console.log('Initializing search engine with volumes:', volumes);
                console.log('Search engine info:', searchEngineInfo);

                // Check if search engine has no indexed files
                const hasNoIndexedFiles = !searchEngineInfo.stats?.trie_size || searchEngineInfo.stats.trie_size === 0;

                console.log('Has no indexed files:', hasNoIndexedFiles, 'Is indexing:', isIndexing);

                // Auto-indexing moved to MainLayout for app startup
                const AUTO_INDEX_ENABLED = false;

                if (hasNoIndexedFiles && !isIndexing && AUTO_INDEX_ENABLED) {
                    console.log('Search engine is empty, starting auto-indexing...');
                    
                    // Auto-index just the home directory to test the increased file limits
                    const autoIndexDirectories = [
                        '/Users/daniel'  // Home directory - will test the 150,000 file limit
                    ];

                    console.log('Auto-indexing directories:', autoIndexDirectories);

                    // Filter to only existing directories
                    const validDirectories = [];
                    for (const dir of autoIndexDirectories) {
                        try {
                            // Check if directory exists by attempting to invoke a simple command
                            validDirectories.push(dir);
                        } catch (error) {
                            console.log(`Skipping non-existent directory: ${dir}`);
                        }
                    }

                    if (validDirectories.length > 0) {
                        await startAutoIndexing(validDirectories.map(dir => ({ mount_point: dir })));
                    } else {
                        console.log('No valid directories found for auto-indexing');
                    }
                } else {
                    console.log('Skipping auto-indexing - either has files, already indexing, or auto-index disabled');
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
                
                // Primary method: Use documented get_search_engine_info
                const info = await invoke('get_search_engine_info');
                
                // Check status from documented API
                const isStillIndexing = info.status && (info.status === 'Indexing' || info.status === '"Indexing"');
                
                // Update progress if available
                if (info.progress) {
                    setIndexingProgress({
                        files_indexed: info.progress.files_indexed || 0,
                        files_discovered: info.progress.files_discovered || 0,
                        percentage_complete: info.progress.percentage_complete || 0.0,
                        current_path: info.progress.current_path || null,
                        estimated_time_remaining: info.progress.estimated_time_remaining || null,
                        start_time: info.progress.start_time || Date.now()
                    });
                }

                console.log('Progress poll result:', {
                    status: info.status,
                    isStillIndexing,
                    progress: info.progress
                });

                // Check if indexing is complete
                if (!isStillIndexing) {
                    console.log('Indexing completed, stopping polling. Final status:', info.status);
                    setIsIndexing(false);
                    setIsLoadingStatus(false); // Reset loading status when polling detects completion
                    stopProgressPolling();
                    // Update search engine info to reflect new state
                    setSearchEngineInfo(info);
                }
                
            } catch (error) {
                console.error('Error polling progress:', error);
                
                // Fallback: Try using undocumented commands if they exist
                try {
                    const [status, progress] = await Promise.all([
                        invoke('get_indexing_status'),
                        invoke('get_indexing_progress')
                    ]);
                    
                    setIndexingProgress(progress);
                    
                    if (status !== 'Indexing' && status !== '"Indexing"') {
                        console.log('Indexing completed (fallback), stopping polling. Final status:', status);
                        setIsIndexing(false);
                        setIsLoadingStatus(false); // Reset loading status in fallback case too
                        stopProgressPolling();
                        await loadSearchEngineInfo();
                    }
                } catch (fallbackError) {
                    console.error('Fallback polling also failed:', fallbackError);
                    // On consecutive errors, stop polling to prevent infinite error loops
                    if (!window.progressErrorCount) window.progressErrorCount = 0;
                    window.progressErrorCount++;
                    if (window.progressErrorCount > 10) {
                        console.error('Too many polling errors, stopping progress polling');
                        setIsIndexing(false);
                        setIsLoadingStatus(false); // Reset loading status on error
                        stopProgressPolling();
                    }
                }
            }
        }, 100); // Slightly longer interval for better performance
    };

    const stopProgressPolling = () => {
        if (progressIntervalRef.current) {
            clearInterval(progressIntervalRef.current);
            progressIntervalRef.current = null;
        }
        // Reset error counter
        window.progressErrorCount = 0;
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

    // Check if indexing is currently in progress
    const checkIndexingStatus = async () => {
        setIsLoadingStatus(true);
        try {
            console.log('Checking indexing status using documented API...');
            
            // Use documented get_search_engine_info command
            const info = await invoke('get_search_engine_info');
            
            console.log('Initial search engine info:', info);
            
            // Check if indexing is in progress based on engine status
            const isCurrentlyIndexing = info.status && (info.status === 'Indexing' || info.status === '"Indexing"');

            if (isCurrentlyIndexing) {
                console.log('Indexing already in progress, starting UI updates...');
                setIsIndexing(true);
                
                // Set progress from search engine info if available
                if (info.progress) {
                    setIndexingProgress({
                        files_indexed: info.progress.files_indexed || 0,
                        files_discovered: info.progress.files_discovered || 0,
                        percentage_complete: info.progress.percentage_complete || 0.0,
                        current_path: info.progress.current_path || null,
                        estimated_time_remaining: info.progress.estimated_time_remaining || null,
                        start_time: info.progress.start_time || Date.now()
                    });
                }
                
                startProgressPolling();
            } else {
                console.log('No indexing in progress');
                setIsIndexing(false);
            }
            
            // Update search engine info
            setSearchEngineInfo(info);
            
        } catch (error) {
            console.error('Failed to check indexing status:', error);
            
            // Fallback to undocumented commands if documented API fails
            try {
                const [status, progress] = await Promise.all([
                    invoke('get_indexing_status'),
                    invoke('get_indexing_progress')
                ]);

                console.log('Fallback - Initial indexing status:', status);
                console.log('Fallback - Initial indexing progress:', progress);

                if (status === 'Indexing' || status === '"Indexing"') {
                    console.log('Indexing already in progress (fallback), starting UI updates...');
                    setIsIndexing(true);
                    setIndexingProgress(progress);
                    startProgressPolling();
                } else {
                    setIsIndexing(false);
                }
            } catch (fallbackError) {
                console.error('Fallback status check also failed:', fallbackError);
                setIsIndexing(false);
            }
        } finally {
            setIsLoadingStatus(false);
        }
    };

    // Load system information
    const loadSystemInfo = async () => {
        try {
            console.log('Loading system info...');
            const metaDataJson = await invoke('get_meta_data_as_json');
            const metaData = JSON.parse(metaDataJson);
            console.log('System info loaded:', metaData);
            setSystemInfo(metaData);
        } catch (error) {
            console.error('Failed to load system info:', error);
            setSystemInfo(null);
        }
    };

    // Load search engine information
    const loadSearchEngineInfo = async () => {
        try {
            console.log('Loading search engine info...');
            const info = await invoke('get_search_engine_info');
            console.log('Search engine info loaded:', info);
            setSearchEngineInfo(info);
            
            // Check if indexing is in progress based on engine status and progress
            const isCurrentlyIndexing = info.status && (info.status === 'Indexing' || info.status === '"Indexing"');
            console.log('Engine status indicates indexing:', isCurrentlyIndexing);
            
            // Update indexing state based on search engine info
            if (isCurrentlyIndexing && !isIndexing) {
                console.log('Starting indexing UI based on engine status...');
                setIsIndexing(true);
                // Set progress from search engine info if available
                if (info.progress) {
                    setIndexingProgress({
                        files_indexed: info.progress.files_indexed || 0,
                        files_discovered: info.progress.files_discovered || 0,
                        percentage_complete: info.progress.percentage_complete || 0.0,
                        current_path: info.progress.current_path || null,
                        estimated_time_remaining: info.progress.estimated_time_remaining || null,
                        start_time: info.progress.start_time || Date.now()
                    });
                }
                startProgressPolling();
            } else if (!isCurrentlyIndexing && isIndexing) {
                console.log('Stopping indexing UI based on engine status...');
                setIsIndexing(false);
                setIsLoadingStatus(false); // Reset loading status when indexing completes
                stopProgressPolling();
            }
            
            // Extract additional data from the comprehensive info
            if (info.recent_activity) {
                setRecentSearches(info.recent_activity.recent_searches || []);
                setMostAccessedPaths(info.recent_activity.most_accessed_paths || []);
            }
            
            if (info.metrics) {
                setSearchMetrics(info.metrics);
            }
        } catch (error) {
            console.error('Failed to load search engine info:', error);
            setSearchEngineInfo(null);
            // If we can't get search engine info, assume no indexing
            if (isIndexing) {
                setIsIndexing(false);
                setIsLoadingStatus(false); // Reset loading status on error
                stopProgressPolling();
            }
        }
    };

    // Perform search using the real API
    const performSearch = async (isAutomatic = false) => {
        if (!query.trim()) {
            setResults([]);
            return;
        }

        // Check if search engine is ready based on documented status
        if (!searchEngineInfo || searchEngineInfo.status === 'Indexing' || searchEngineInfo.status === '"Indexing"') {
            console.log('Search blocked - search engine not ready or indexing in progress');
            return;
        }

        // Check if there are indexed files to search
        if (!searchEngineInfo.stats?.trie_size || searchEngineInfo.stats.trie_size === 0) {
            console.log('Search blocked - no files indexed yet');
            setResults([]);
            return;
        }

        // Store current focus state for automatic searches
        const wasInputFocused = isAutomatic && document.activeElement === searchInputRef.current;

        setIsSearching(true);
        setResults([]);

        try {
            const searchStartTime = performance.now();
            let searchResults;

            // Use extension filtering if extensions are selected
            if (selectedExtensions.length > 0) {
                console.log(`Searching with extensions: ${selectedExtensions.join(', ')}`);
                searchResults = await invoke('search_with_extension', {
                    query: query.trim(),
                    extensions: selectedExtensions
                });
            } else {
                // Use basic search
                console.log(`Performing basic search for: "${query.trim()}"`);
                searchResults = await invoke('search', {
                    query: query.trim()
                });
            }

            const searchEndTime = performance.now();
            const searchTime = searchEndTime - searchStartTime;
            
            console.log(`Search completed in ${searchTime.toFixed(2)}ms with ${searchResults.length} results`);

            // Convert API results to our format and apply frontend filtering
            let formattedResults = searchResults.map(([path, score]) => {
                const fileName = path.split(/[/\\]/).pop() || path;
                const directory = path.substring(0, path.lastIndexOf(fileName) - 1) || '/';
                const isDirectory = !fileName.includes('.') || path.endsWith('/');

                return {
                    path,
                    name: fileName,
                    directory,
                    score,
                    isDirectory,
                    extension: isDirectory ? null : fileName.split('.').pop()?.toLowerCase()
                };
            });

            // Apply frontend filters
            if (showDirectoriesOnly) {
                formattedResults = formattedResults.filter(result => result.isDirectory);
            }

            if (!showHiddenFiles) {
                formattedResults = formattedResults.filter(result => !result.name.startsWith('.'));
            }

            // Apply sorting
            formattedResults = sortResults(formattedResults);

            setResults(formattedResults);
            
            // Update recent searches (keep last 10) - only for queries with 3+ characters
            if (query.trim().length >= 3) {
                setRecentSearches(prev => {
                    const updated = [query.trim(), ...prev.filter(q => q !== query.trim())];
                    return updated.slice(0, 10);
                });
            }

            // Update search engine stats after successful search
            await loadSearchEngineInfo();

        } catch (error) {
            console.error('Search failed:', error);
            // Show user-friendly error message only for manual searches
            if (query.trim().length >= 3) {
                const errorMessage = error.message || error;
                if (errorMessage.includes('No search engine available')) {
                    console.warn('Search engine not ready:', errorMessage);
                } else {
                    console.error('Search error:', errorMessage);
                }
            }
        } finally {
            setIsSearching(false);
            
            // Restore focus to input if it was focused before automatic search
            if (wasInputFocused && searchInputRef.current) {
                setTimeout(() => {
                    searchInputRef.current.focus();
                }, 0);
            }
        }
    };

    // Clear search
    const clearSearch = () => {
        setQuery('');
        setResults([]);
    };

    // Sort results based on selected criteria
    const sortResults = (results) => {
        return [...results].sort((a, b) => {
            switch (sortBy) {
                case 'name':
                    return a.name.localeCompare(b.name);
                case 'path':
                    return a.path.localeCompare(b.path);
                case 'extension':
                    if (!a.extension && !b.extension) return 0;
                    if (!a.extension) return 1;
                    if (!b.extension) return -1;
                    return a.extension.localeCompare(b.extension);
                case 'relevance':
                default:
                    return b.score - a.score; // Higher score first
            }
        });
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
            // Record path usage for ranking improvement
            await recordPathUsage(result.path);
            
            // Update most accessed paths
            setMostAccessedPaths(prev => {
                const updated = [result.path, ...prev.filter(p => p !== result.path)];
                return updated.slice(0, 10);
            });

            await loadDirectory(result.directory);
            navigateTo(result.directory);
            onClose();
        } catch (error) {
            console.error('Failed to open item location:', error);
            alert(`Failed to open location: ${error.message || error}`);
        }
    };

    // Record path usage to improve future search ranking
    const recordPathUsage = async (path) => {
        try {
            // Note: This would be a new backend command we'd need to implement
            // For now, we'll just track it in the frontend
            console.log('Recording path usage:', path);
            // await invoke('record_path_usage', { path });
        } catch (error) {
            console.error('Failed to record path usage:', error);
        }
    };

    // Quick search from recent searches
    const searchFromRecent = (recentQuery) => {
        setQuery(recentQuery);
        // Automatically trigger search
        setTimeout(() => {
            performSearch(false); // This is a user-initiated search, so don't preserve focus
        }, 100);
    };

    // Quick navigate to most accessed path
    const navigateToAccessedPath = async (path) => {
        try {
            const directory = path.substring(0, path.lastIndexOf('/')) || '/';
            await loadDirectory(directory);
            navigateTo(directory);
            onClose();
        } catch (error) {
            console.error('Failed to navigate to path:', error);
            alert(`Failed to navigate: ${error.message || error}`);
        }
    };

    // Auto-start indexing for volumes
    const startAutoIndexing = async (volumesToIndex = volumes) => {
        if (volumesToIndex.length === 0 || isIndexing) {
            console.log('Skipping auto-indexing - no volumes or already indexing');
            return;
        }

        console.log('Starting auto-indexing for volumes:', volumesToIndex);

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
            for (const volume of volumesToIndex) {
                console.log(`Adding ${volume.mount_point} to search index...`);

                // Check if the volume path exists and is accessible
                try {
                    const result = await invoke('add_paths_recursive_async', {
                        folder: volume.mount_point
                    });

                    console.log(`Indexing result for ${volume.mount_point}:`, result);
                } catch (volumeError) {
                    console.error(`Failed to index volume ${volume.mount_point}:`, volumeError);
                    // Continue with other volumes even if one fails
                }
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
        if (!systemInfo?.user_home_dir) {
            alert('User home directory not available for indexing');
            return;
        }

        console.log('Starting manual indexing for home directory:', systemInfo.user_home_dir);

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
            console.log(`Manually indexing home directory: ${systemInfo.user_home_dir}...`);

            // Try async version first, fallback to sync version
            let result;
            try {
                result = await invoke('add_paths_recursive_async', {
                    folder: systemInfo.user_home_dir
                });
            } catch (asyncError) {
                console.log('Async indexing failed, trying sync version:', asyncError);
                result = await invoke('add_paths_recursive', {
                    folder: systemInfo.user_home_dir
                });
            }

            console.log(`Manual indexing result for ${systemInfo.user_home_dir}:`, result);
            console.log('Manual indexing initiated successfully');
            
            // Don't show alert immediately - let polling handle completion notification
            console.log('Indexing started successfully. Progress will be shown above.');

        } catch (error) {
            console.error('Manual indexing failed:', error);
            alert(`Failed to start indexing: ${error.message || error}`);
            setIsIndexing(false);
            stopProgressPolling();
        }
    };

    // Clear search engine index
    const clearSearchEngine = async () => {
        if (!window.confirm('Are you sure you want to clear the entire search index? This will remove all indexed files and you will need to re-index.')) {
            return;
        }

        try {
            console.log('Clearing search engine...');
            await invoke('clear_search_engine');
            console.log('Search engine cleared successfully');
            
            // Update UI state
            setResults([]);
            setQuery('');
            await loadSearchEngineInfo();
            
            alert('Search engine index cleared successfully. You can now re-index your files.');
        } catch (error) {
            console.error('Failed to clear search engine:', error);
            alert(`Failed to clear search engine: ${error.message || error}`);
        }
    };

    const handleSubmit = (e) => {
        e.preventDefault();
        performSearch(false); // false indicates this is a manual search
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
                                ref={searchInputRef}
                                type="text"
                                className="search-input-field"
                                value={query}
                                onChange={(e) => setQuery(e.target.value)}
                                placeholder="Search files and folders....."
                                autoFocus
                            />
                            {isSearching && (
                                <div className="search-loading-indicator" style={{ 
                                    position: 'absolute', 
                                    right: '8px', 
                                    top: '50%', 
                                    transform: 'translateY(-50%)',
                                    fontSize: '12px',
                                    color: '#666',
                                    pointerEvents: 'none', // Prevent interference with input
                                    zIndex: 1
                                }}>
                                    Searching...
                                </div>
                            )}
                        </div>
                        <Button
                            type="submit"
                            variant="primary"
                            disabled={isSearching || !query.trim()}
                            style={{ opacity: query.trim().length >= 3 ? 1 : 0.5 }}
                        >
                            {isSearching ? 'Searching...' : 'Search'}
                        </Button>
                    </div>

                    {/* Search hint for instant search */}
                    {query.length > 0 && query.length < 3 && (
                        <div style={{ 
                            fontSize: '12px', 
                            color: '#666', 
                            marginBottom: '8px',
                            padding: '4px 8px',
                            backgroundColor: '#f0f0f0',
                            borderRadius: '4px'
                        }}>
                            Type at least 3 characters to start searching...
                        </div>
                    )}

                    {/* Search Engine Status */}
                    {(searchEngineInfo || isLoadingStatus) && (
                        <div className="search-engine-status">
                            {isLoadingStatus && (
                                <div className="status-info">
                                    <span className="status-label">Status:</span>
                                    <span className="status-value loading">
                                        Checking indexing status...
                                    </span>
                                </div>
                            )}
                            {searchEngineInfo && !isLoadingStatus && (
                                <>
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
                                </>
                            )}
                            {(isIndexing || isLoadingStatus) && (
                                <div className="status-info">
                                    <span className="status-label">Indexing:</span>
                                    <span className="status-value indexing">
                                        {isLoadingStatus ? 'Checking...' : 'In Progress...'}
                                    </span>
                                </div>
                            )}
                        </div>
                    )}

                    {/* Indexing Progress UI */}
                    {(isIndexing || isLoadingStatus) && (
                        <div className="indexing-progress">
                            {isLoadingStatus && !isIndexing && (
                                <div className="progress-header">
                                    <h3>Checking Indexing Status...</h3>
                                    <span className="progress-percentage">Please wait...</span>
                                </div>
                            )}
                            
                            {isIndexing && (
                                <>
                                    <div className="progress-header">
                                        <h3>
                                            {indexingProgress.files_discovered === 0 ? 
                                                'Starting Discovery...' : 
                                                indexingProgress.files_indexed === 0 ? 'Discovering Files...' :
                                                indexingProgress.files_indexed < indexingProgress.files_discovered ? 'Indexing Files...' :
                                                'Finalizing...'
                                            }
                                        </h3>
                                        <span className="progress-percentage">
                                            {indexingProgress.files_discovered === 0 ? 
                                                'Starting...' : 
                                                indexingProgress.files_indexed === 0 ? `${indexingProgress.files_discovered} found` :
                                                `${indexingProgress.percentage_complete.toFixed(1)}%`
                                            }
                                        </span>
                                    </div>

                                    <div className="progress-bar">
                                        <div
                                            className="progress-fill"
                                            style={{ 
                                                width: indexingProgress.files_discovered === 0 ? 
                                                    '2%' : // Show a small progress to indicate activity
                                                    indexingProgress.files_indexed === 0 ? '5%' : // Show more when discovering
                                                    `${Math.max(5, indexingProgress.percentage_complete)}%` // Ensure minimum 5% when indexing
                                            }}
                                        />
                                    </div>

                                    <div className="progress-details">
                                        <div className="progress-stats">
                                            <span>
                                                {indexingProgress.files_indexed} indexed / {indexingProgress.files_discovered} discovered
                                                {indexingProgress.files_discovered === 0 && ' (starting...)'}
                                                {indexingProgress.files_discovered > 0 && indexingProgress.files_indexed === 0 && ' (discovering & indexing...)'}
                                            </span>
                                            {indexingProgress.estimated_time_remaining && indexingProgress.files_indexed > 0 && (
                                                <span>
                                                    {formatTimeRemaining(indexingProgress.estimated_time_remaining)} remaining
                                                </span>
                                            )}
                                        </div>

                                        {indexingProgress.current_path && (
                                            <div className="current-file">
                                                <span className="current-file-label">
                                                    {indexingProgress.files_discovered === 0 ? 'Starting:' : 
                                                     indexingProgress.files_indexed === 0 ? 'Discovering:' : 'Processing:'}
                                                </span>
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
                                </>
                            )}

                            {isLoadingStatus && !isIndexing && (
                                <div className="status-checking-indicator" style={{
                                    textAlign: 'center',
                                    padding: '20px',
                                    color: '#666',
                                    fontSize: '14px'
                                }}>
                                    <div style={{ marginBottom: '10px' }}>
                                        <span className="icon icon-search" style={{ 
                                            width: '24px', 
                                            height: '24px',
                                            display: 'inline-block'
                                        }}></span>
                                    </div>
                                    Checking if background indexing is in progress...
                                </div>
                            )}
                        </div>
                    )}

                    {/* Accordions Container */}
                    <div className="accordions-container">
                        {/* Search Controls Accordion */}
                        <div className="accordion">
                            <button
                                className="accordion-header"
                                onClick={() => setFiltersExpanded(!filtersExpanded)}
                                type="button"
                            >
                                <span>Search Controls & Filters</span>
                                <span className={`accordion-chevron ${filtersExpanded ? 'expanded' : ''}`}>▼</span>
                            </button>

                            {filtersExpanded && (
                                <div className="accordion-content">
                                    {/* Current Directory Context */}
                                    {currentDirectory && (
                                        <div className="search-control-section">
                                            <h4>Current Context</h4>
                                            <div className="current-directory-info">
                                                <span className="directory-label">Current Directory:</span>
                                                <span className="directory-path" title={currentDirectory}>
                                                    {currentDirectory}
                                                </span>
                                                <small>(Files in this directory will be ranked higher)</small>
                                            </div>
                                        </div>
                                    )}

                                    {/* Sort Controls */}
                                    <div className="search-control-section">
                                        <h4>Sort Results By</h4>
                                        <div className="sort-controls">
                                            {[
                                                { value: 'relevance', label: 'Relevance (Score)' },
                                                { value: 'name', label: 'Name' },
                                                { value: 'path', label: 'Path' },
                                                { value: 'extension', label: 'File Type' }
                                            ].map(option => (
                                                <label key={option.value} className="radio-option">
                                                    <input
                                                        type="radio"
                                                        name="sortBy"
                                                        value={option.value}
                                                        checked={sortBy === option.value}
                                                        onChange={(e) => setSortBy(e.target.value)}
                                                        disabled={isSearching}
                                                    />
                                                    <span>{option.label}</span>
                                                </label>
                                            ))}
                                        </div>
                                    </div>

                                    {/* Filter Controls */}
                                    <div className="search-control-section">
                                        <h4>Filter Options</h4>
                                        <div className="filter-controls">
                                            <label className="checkbox-option">
                                                <input
                                                    type="checkbox"
                                                    checked={showDirectoriesOnly}
                                                    onChange={(e) => setShowDirectoriesOnly(e.target.checked)}
                                                    disabled={isSearching}
                                                />
                                                <span>Show directories only</span>
                                            </label>
                                            <label className="checkbox-option">
                                                <input
                                                    type="checkbox"
                                                    checked={showHiddenFiles}
                                                    onChange={(e) => setShowHiddenFiles(e.target.checked)}
                                                    disabled={isSearching}
                                                />
                                                <span>Show hidden files</span>
                                            </label>
                                        </div>
                                    </div>

                                    {/* Extension Filters */}
                                    <div className="search-control-section">
                                        <h4>File Type Filters</h4>
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

                                    {/* Recent Searches */}
                                    {recentSearches.length > 0 && (
                                        <div className="search-control-section">
                                            <h4>Recent Searches</h4>
                                            <div className="recent-searches">
                                                {recentSearches.slice(0, 5).map((recentQuery, index) => (
                                                    <button
                                                        key={index}
                                                        className="recent-search-item"
                                                        onClick={() => searchFromRecent(recentQuery)}
                                                        disabled={isSearching}
                                                        title={`Search for: ${recentQuery}`}
                                                    >
                                                        "{recentQuery}"
                                                    </button>
                                                ))}
                                            </div>
                                        </div>
                                    )}

                                    {/* Most Accessed Paths */}
                                    {mostAccessedPaths.length > 0 && (
                                        <div className="search-control-section">
                                            <h4>Most Accessed Paths</h4>
                                            <div className="most-accessed-paths">
                                                {mostAccessedPaths.slice(0, 5).map((path, index) => (
                                                    <button
                                                        key={index}
                                                        className="accessed-path-item"
                                                        onClick={() => navigateToAccessedPath(path)}
                                                        title={`Navigate to: ${path}`}
                                                    >
                                                        <span className="path-name">
                                                            {path.split('/').pop() || path}
                                                        </span>
                                                        <span className="path-location">
                                                            {path}
                                                        </span>
                                                    </button>
                                                ))}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>

                        {/* Performance & Statistics Accordion */}
                        {searchEngineInfo && (
                            <div className="accordion">
                                <button
                                    className="accordion-header"
                                    onClick={() => setStatsExpanded(!statsExpanded)}
                                    type="button"
                                >
                                    <span>Engine Statistics & Performance</span>
                                    <span className={`accordion-chevron ${statsExpanded ? 'expanded' : ''}`}>▼</span>
                                </button>

                                {statsExpanded && (
                                    <div className="accordion-content">
                                        <div className="search-engine-info-compact">
                                            <div className="info-grid">
                                                <div className="info-item">
                                                    <span className="info-label">Engine Status:</span>
                                                    <span className="info-value">{searchEngineInfo.status || 'Unknown'}</span>
                                                </div>
                                                <div className="info-item">
                                                    <span className="info-label">Indexed Files:</span>
                                                    <span className="info-value">{searchEngineInfo.stats?.trie_size || 0}</span>
                                                </div>
                                                <div className="info-item">
                                                    <span className="info-label">Cache Size:</span>
                                                    <span className="info-value">{searchEngineInfo.stats?.cache_size || 0}</span>
                                                </div>
                                                <div className="info-item">
                                                    <span className="info-label">Total Searches:</span>
                                                    <span className="info-value">{searchMetrics.total_searches || 0}</span>
                                                </div>
                                                <div className="info-item">
                                                    <span className="info-label">Avg Search Time:</span>
                                                    <span className="info-value">{searchMetrics.average_search_time_ms || 0}ms</span>
                                                </div>
                                                <div className="info-item">
                                                    <span className="info-label">Cache Hit Rate:</span>
                                                    <span className="info-value">
                                                        {searchMetrics.cache_hit_rate ? 
                                                            `${(searchMetrics.cache_hit_rate * 100).toFixed(1)}%` : 
                                                            'N/A'
                                                        }
                                                    </span>
                                                </div>
                                                {searchEngineInfo.last_updated && (
                                                    <div className="info-item">
                                                        <span className="info-label">Last Updated:</span>
                                                        <span className="info-value">
                                                            {new Date(searchEngineInfo.last_updated).toLocaleString()}
                                                        </span>
                                                    </div>
                                                )}
                                            </div>

                                            {/* Manual Indexing Control */}
                                            <div className="index-management">
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    onClick={startManualIndexing}
                                                    disabled={isSearching || isIndexing || !systemInfo?.user_home_dir}
                                                >
                                                    {isIndexing ? 'Indexing...' : 'Re-index Home Directory'}
                                                </Button>
                                                
                                                {/* Test indexing with a smaller directory */}
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    onClick={async () => {
                                                        if (!systemInfo?.user_home_dir) {
                                                            alert('System info not available');
                                                            return;
                                                        }

                                                        console.log('Starting test indexing...');
                                                        setIsIndexing(true);
                                                        setIndexingProgress({
                                                            files_indexed: 0,
                                                            files_discovered: 0,
                                                            percentage_complete: 0.0,
                                                            current_path: null,
                                                            estimated_time_remaining: null,
                                                            start_time: Date.now()
                                                        });
                                                        startProgressPolling();
                                                        
                                                        try {
                                                            // Test with user's Documents directory
                                                            const documentsPath = systemInfo.current_running_os === 'windows' 
                                                                ? `${systemInfo.user_home_dir}\\Documents`
                                                                : `${systemInfo.user_home_dir}/Documents`;
                                                            
                                                            console.log(`Testing indexing with: ${documentsPath}`);
                                                            
                                                            const result = await invoke('add_paths_recursive_async', {
                                                                folder: documentsPath
                                                            });
                                                            
                                                            console.log('Test indexing result:', result);
                                                        } catch (error) {
                                                            console.error('Test indexing failed:', error);
                                                            alert(`Test indexing failed: ${error.message || error}`);
                                                            setIsIndexing(false);
                                                            stopProgressPolling();
                                                        }
                                                    }}
                                                    disabled={isSearching || isIndexing || !systemInfo?.user_home_dir}
                                                    style={{ marginLeft: '8px' }}
                                                >
                                                    Test Index (Documents)
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
                                <div className="results-info">
                                    <span className="results-count">
                                        {results.length} results found for "{query}"
                                    </span>
                                    <span className="sort-info">
                                        Sorted by: {sortBy === 'relevance' ? 'Relevance (Score)' : 
                                                   sortBy === 'name' ? 'Name' : 
                                                   sortBy === 'path' ? 'Path' : 'File Type'}
                                    </span>
                                </div>
                                <div className="results-actions">
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        onClick={clearSearch}
                                    >
                                        Clear
                                    </Button>
                                </div>
                            </div>

                            <div className="results-items-container">
                                {results.map((result, index) => (
                                    <div key={index} className="result-item-container">
                                        <div className="result-icon-container">
                                            <FileIcon 
                                                filename={result.name} 
                                                isDirectory={result.isDirectory} 
                                                size="small"
                                            />
                                        </div>

                                        <div className="result-details-container">
                                            <div
                                                className="result-name-container"
                                                onClick={() => openItemLocation(result)}
                                                title={`Open location: ${result.directory}`}
                                            >
                                                <span className="result-name">{result.name}</span>
                                                {result.extension && (
                                                    <span className="result-extension">.{result.extension}</span>
                                                )}
                                                {result.isDirectory && (
                                                    <span className="result-type-indicator">(Directory)</span>
                                                )}
                                            </div>
                                            <div
                                                className="result-path-container"
                                                onClick={() => openItemLocation(result)}
                                                title={result.path}
                                            >
                                                {result.path}
                                            </div>
                                            {currentDirectory && result.path.startsWith(currentDirectory) && (
                                                <div className="result-context-indicator">
                                                    <span className="context-dot">•</span> In current directory
                                                </div>
                                            )}
                                        </div>

                                        <div className="result-meta-container">
                                            <div className="result-score-container" title="Relevance score">
                                                {Math.round(result.score * 100)}%
                                            </div>
                                            {sortBy === 'relevance' && (
                                                <div className="result-rank" title="Search result rank">
                                                    #{index + 1}
                                                </div>
                                            )}
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
