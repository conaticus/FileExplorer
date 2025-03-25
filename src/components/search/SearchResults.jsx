import React from 'react';
import FileIcon from '../file-view/FileIcon';

const SearchResults = ({
                           results = [],
                           isLoading = false,
                           query = '',
                           onItemClick,
                           onClose
                       }) => {
    // Formatiere ein Datum
    const formatDate = (dateString) => {
        if (!dateString) return '';
        const date = new Date(dateString);
        return date.toLocaleDateString();
    };

    // Hervorheben des Suchbegriffs im Text
    const highlightMatch = (text, query) => {
        if (!query || !text) return text;

        const parts = text.split(new RegExp(`(${query})`, 'gi'));

        return parts.map((part, index) => {
            return part.toLowerCase() === query.toLowerCase() ? (
                <span key={index} className="search-highlight">{part}</span>
            ) : (
                part
            );
        });
    };

    return (
        <div className="search-results">
            <div className="search-results-header">
                <div className="search-results-title">
                    {isLoading ? (
                        <span>Suche nach "{query}"...</span>
                    ) : results.length > 0 ? (
                        <span>{results.length} Ergebnisse für "{query}"</span>
                    ) : (
                        <span>Keine Ergebnisse für "{query}"</span>
                    )}
                </div>
                <button
                    className="search-results-close"
                    onClick={onClose}
                    aria-label="Suchergebnisse schließen"
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
            </div>

            <div className="search-results-content">
                {isLoading ? (
                    <div className="search-results-loading">
                        <div className="search-loading-spinner"></div>
                        <p>Suche läuft...</p>
                    </div>
                ) : results.length === 0 ? (
                    <div className="search-results-empty">
                        <p>Keine Ergebnisse gefunden.</p>
                    </div>
                ) : (
                    <ul className="search-results-list">
                        {results.map((item, index) => (
                            <li
                                key={item.path || index}
                                className="search-result-item"
                                onClick={() => onItemClick(item)}
                            >
                                <div className="search-result-icon">
                                    <FileIcon
                                        fileType={item.type}
                                        extension={item.name?.split('.').pop()}
                                    />
                                </div>
                                <div className="search-result-details">
                                    <div className="search-result-name">
                                        {highlightMatch(item.name, query)}
                                    </div>
                                    <div className="search-result-path text-truncate" title={item.path}>
                                        {item.path}
                                    </div>
                                    <div className="search-result-meta">
                                        {item.type === 'file' && item.size && (
                                            <span className="search-result-size">{item.size}</span>
                                        )}
                                        {item.modified && (
                                            <span className="search-result-date">
                        {formatDate(item.modified)}
                      </span>
                                        )}
                                    </div>
                                </div>
                            </li>
                        ))}
                    </ul>
                )}
            </div>
        </div>
    );
};

export default SearchResults;