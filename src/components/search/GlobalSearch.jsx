import React, { useState, useEffect, useRef } from 'react';
import SearchResults from './SearchResults';
import { useFileSystem } from '../../providers/FileSystemProvider';

const GlobalSearch = ({
                          isSearchActive,
                          searchQuery,
                          onSearch,
                          onClearSearch
                      }) => {
    const { searchFiles } = useFileSystem();
    const [localQuery, setLocalQuery] = useState(searchQuery || '');
    const [showResults, setShowResults] = useState(false);
    const [searchResults, setSearchResults] = useState([]);
    const [isLoading, setIsLoading] = useState(false);
    const [searchLocation, setSearchLocation] = useState('all'); // 'all', 'current', 'custom'
    const [customLocation, setCustomLocation] = useState('');
    const [isAdvancedSearch, setIsAdvancedSearch] = useState(false);
    const [searchOptions, setSearchOptions] = useState({
        includeSubfolders: true,
        caseSensitive: false,
        matchWholeWord: false,
        searchContents: false,
        fileTypes: []
    });

    const searchInputRef = useRef(null);
    const searchResultsRef = useRef(null);

    // Aktualisiere den lokalen Zustand, wenn sich der externe Zustand ändert
    useEffect(() => {
        setLocalQuery(searchQuery || '');
    }, [searchQuery]);

    // Fokussiere das Suchfeld mit Tastenkombination
    useEffect(() => {
        const handleKeyDown = (e) => {
            // Wenn Strg+F gedrückt wird (oder Cmd+F auf Mac)
            if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
                e.preventDefault();
                if (searchInputRef.current) {
                    searchInputRef.current.focus();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, []);

    // Klicks außerhalb des Suchergebnisbereichs behandeln
    useEffect(() => {
        const handleClickOutside = (e) => {
            if (
                showResults &&
                searchResultsRef.current &&
                !searchResultsRef.current.contains(e.target) &&
                !searchInputRef.current.contains(e.target)
            ) {
                setShowResults(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [showResults]);

    // Führe die Suche durch
    const handleSearch = async (e) => {
        e?.preventDefault();

        if (!localQuery.trim()) {
            handleClearSearch();
            return;
        }

        setIsLoading(true);
        setShowResults(true);

        try {
            // Bestimme den Suchpfad basierend auf der ausgewählten Option
            let searchPath = null;

            if (searchLocation === 'current') {
                // Verwende den aktuellen Pfad
                // [Backend Integration] - Aktuellen Pfad vom Backend abrufen
                searchPath = '/current/path'; // Beispiel
            } else if (searchLocation === 'custom' && customLocation) {
                searchPath = customLocation;
            }

            // [Backend Integration] - Suche im Backend durchführen
            // /* BACKEND_INTEGRATION: Suche durchführen */

            // Beispieldaten
            const results = [
                { name: `file-${localQuery}.txt`, path: `/path/to/file-${localQuery}.txt`, type: 'file', size: '12 KB', modified: new Date().toISOString() },
                { name: `document-${localQuery}.docx`, path: `/path/to/document-${localQuery}.docx`, type: 'file', size: '24 KB', modified: new Date().toISOString() },
                { name: localQuery, path: `/path/to/${localQuery}`, type: 'directory', modified: new Date().toISOString() }
            ];

            setSearchResults(results);

            // Aktualisiere den globalen Suchzustand
            onSearch(localQuery);
        } catch (error) {
            console.error('Error during search:', error);
            setSearchResults([]);
        } finally {
            setIsLoading(false);
        }
    };

    // Suche löschen
    const handleClearSearch = () => {
        setLocalQuery('');
        setSearchResults([]);
        setShowResults(false);
        onClearSearch();
        if (searchInputRef.current) {
            searchInputRef.current.focus();
        }
    };

    // Suchoptionen aktualisieren
    const updateSearchOption = (option, value) => {
        setSearchOptions(prev => ({
            ...prev,
            [option]: value
        }));
    };

    // Dateityp-Filter aktualisieren
    const toggleFileType = (fileType) => {
        setSearchOptions(prev => ({
            ...prev,
            fileTypes: prev.fileTypes.includes(fileType)
                ? prev.fileTypes.filter(type => type !== fileType)
                : [...prev.fileTypes, fileType]
        }));
    };

    // Erweiterte Suchoptionen rendern
    const renderAdvancedOptions = () => {
        return (
            <div className="search-advanced-options">
                <div className="search-option-group">
                    <h4>Suchort</h4>
                    <div className="search-radio-group">
                        <label className="search-radio">
                            <input
                                type="radio"
                                name="searchLocation"
                                checked={searchLocation === 'all'}
                                onChange={() => setSearchLocation('all')}
                            />
                            <span>Überall</span>
                        </label>
                        <label className="search-radio">
                            <input
                                type="radio"
                                name="searchLocation"
                                checked={searchLocation === 'current'}
                                onChange={() => setSearchLocation('current')}
                            />
                            <span>Aktueller Ordner</span>
                        </label>
                        <label className="search-radio">
                            <input
                                type="radio"
                                name="searchLocation"
                                checked={searchLocation === 'custom'}
                                onChange={() => setSearchLocation('custom')}
                            />
                            <span>Benutzerdefinierter Ort</span>
                        </label>
                    </div>

                    {searchLocation === 'custom' && (
                        <input
                            type="text"
                            className="search-custom-location"
                            value={customLocation}
                            onChange={(e) => setCustomLocation(e.target.value)}
                            placeholder="Pfad eingeben..."
                        />
                    )}
                </div>

                <div className="search-option-group">
                    <h4>Suchoptionen</h4>
                    <label className="search-checkbox">
                        <input
                            type="checkbox"
                            checked={searchOptions.includeSubfolders}
                            onChange={(e) => updateSearchOption('includeSubfolders', e.target.checked)}
                        />
                        <span>Unterordner einbeziehen</span>
                    </label>
                    <label className="search-checkbox">
                        <input
                            type="checkbox"
                            checked={searchOptions.caseSensitive}
                            onChange={(e) => updateSearchOption('caseSensitive', e.target.checked)}
                        />
                        <span>Groß-/Kleinschreibung beachten</span>
                    </label>
                    <label className="search-checkbox">
                        <input
                            type="checkbox"
                            checked={searchOptions.matchWholeWord}
                            onChange={(e) => updateSearchOption('matchWholeWord', e.target.checked)}
                        />
                        <span>Ganzes Wort</span>
                    </label>
                    <label className="search-checkbox">
                        <input
                            type="checkbox"
                            checked={searchOptions.searchContents}
                            onChange={(e) => updateSearchOption('searchContents', e.target.checked)}
                        />
                        <span>Dateiinhalt durchsuchen</span>
                    </label>
                </div>

                <div className="search-option-group">
                    <h4>Dateitypen</h4>
                    <div className="search-file-types">
                        <label className="search-checkbox">
                            <input
                                type="checkbox"
                                checked={searchOptions.fileTypes.includes('document')}
                                onChange={() => toggleFileType('document')}
                            />
                            <span>Dokumente</span>
                        </label>
                        <label className="search-checkbox">
                            <input
                                type="checkbox"
                                checked={searchOptions.fileTypes.includes('image')}
                                onChange={() => toggleFileType('image')}
                            />
                            <span>Bilder</span>
                        </label>
                        <label className="search-checkbox">
                            <input
                                type="checkbox"
                                checked={searchOptions.fileTypes.includes('audio')}
                                onChange={() => toggleFileType('audio')}
                            />
                            <span>Audio</span>
                        </label>
                        <label className="search-checkbox">
                            <input
                                type="checkbox"
                                checked={searchOptions.fileTypes.includes('video')}
                                onChange={() => toggleFileType('video')}
                            />
                            <span>Video</span>
                        </label>
                        <label className="search-checkbox">
                            <input
                                type="checkbox"
                                checked={searchOptions.fileTypes.includes('archive')}
                                onChange={() => toggleFileType('archive')}
                            />
                            <span>Archive</span>
                        </label>
                    </div>
                </div>
            </div>
        );
    };

    return (
        <div className="global-search">
            <form onSubmit={handleSearch} className="search-form">
                <div className="search-input-container">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        width="16"
                        height="16"
                        className="search-icon"
                    >
                        <path d="M11 17.25a6.25 6.25 0 1 1 0-12.5 6.25 6.25 0 0 1 0 12.5z" />
                        <path d="M16 16l4.5 4.5" />
                    </svg>

                    <input
                        type="text"
                        className="search-input"
                        value={localQuery}
                        onChange={(e) => setLocalQuery(e.target.value)}
                        onFocus={() => localQuery && setShowResults(true)}
                        placeholder="Suchen..."
                        aria-label="Globale Suche"
                        ref={searchInputRef}
                    />

                    {localQuery && (
                        <button
                            type="button"
                            className="search-clear-button"
                            onClick={handleClearSearch}
                            aria-label="Suche löschen"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                width="14"
                                height="14"
                            >
                                <path d="M18 6L6 18M6 6l12 12" />
                            </svg>
                        </button>
                    )}

                    <button
                        type="button"
                        className={`search-options-button ${isAdvancedSearch ? 'active' : ''}`}
                        onClick={() => setIsAdvancedSearch(!isAdvancedSearch)}
                        aria-label="Erweiterte Suchoptionen"
                        title="Erweiterte Suchoptionen"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            width="14"
                            height="14"
                        >
                            <path d="M3 6h18M6 12h12M9 18h6" />
                        </svg>
                    </button>

                    <button
                        type="submit"
                        className="search-button"
                        aria-label="Suchen"
                    >
                        <span>Suchen</span>
                    </button>
                </div>

                {isAdvancedSearch && renderAdvancedOptions()}
            </form>

            {showResults && (
                <div className="search-results-container" ref={searchResultsRef}>
                    <SearchResults
                        results={searchResults}
                        isLoading={isLoading}
                        query={localQuery}
                        onItemClick={(item) => {
                            // [Backend Integration] - Zum Element navigieren
                            // /* BACKEND_INTEGRATION: Zum Element navigieren */
                            console.log('Navigate to:', item.path);
                            setShowResults(false);
                        }}
                        onClose={() => setShowResults(false)}
                    />
                </div>
            )}
        </div>
    );
};

export default GlobalSearch;