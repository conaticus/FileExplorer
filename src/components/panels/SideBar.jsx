import React, { useState } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';

const SideBar = ({
                     isOpen,
                     favorites = [],
                     recentPaths = [],
                     onToggle,
                     onNavigate,
                     onAddFavorite,
                     onRemoveFavorite
                 }) => {
    const { rootFolders, dataSources, addDataSource } = useFileSystem();
    const [activeTab, setActiveTab] = useState('quickAccess');
    const [isAddingSource, setIsAddingSource] = useState(false);
    const [newSourcePath, setNewSourcePath] = useState('');

    // Tabs für die Seitenleiste
    const tabs = [
        { id: 'quickAccess', label: 'Schnellzugriff', icon: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2M9 5h6' },
        { id: 'thisPC', label: 'Dieser PC', icon: 'M20 17.58A5 5 0 0 0 18 8h-1.26A8 8 0 1 0 4 16.25M8 16h.01' },
        { id: 'favorites', label: 'Favoriten', icon: 'M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14 2 9.27 8.91 8.26 12 2z' },
        { id: 'templates', label: 'Templates', icon: 'M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z M3 6l18 0 M16 10a4 4 0 0 1-8 0' },
    ];

    // Neuen Pfad hinzufügen
    const handleAddSource = () => {
        if (newSourcePath) {
            addDataSource(newSourcePath);
            setNewSourcePath('');
            setIsAddingSource(false);
        }
    };

    // Formatiere einen Pfad für die Anzeige
    const formatDisplayPath = (path) => {
        if (!path) return '';

        // Entferne Pfadtrenner vom Ende
        const trimmedPath = path.replace(/[/\\]$/, '');

        // Extrahiere den letzten Teil des Pfads
        const parts = trimmedPath.split(/[/\\]/);
        return parts[parts.length - 1] || path;
    };

    return (
        <div className={`sidebar ${isOpen ? '' : 'collapsed'}`}>
            {/* Tabs */}
            <div className="sidebar-tabs">
                {tabs.map(tab => (
                    <button
                        key={tab.id}
                        className={`sidebar-tab ${activeTab === tab.id ? 'active' : ''}`}
                        onClick={() => setActiveTab(tab.id)}
                        title={tab.label}
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            width="20"
                            height="20"
                        >
                            <path d={tab.icon} />
                        </svg>
                        <span className="sidebar-tab-label">{tab.label}</span>
                    </button>
                ))}
            </div>

            {/* Tab-Inhalt */}
            <div className="sidebar-content">
                {activeTab === 'quickAccess' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">SCHNELLZUGRIFF</h3>
                        <ul className="sidebar-list">
                            {rootFolders.map(folder => (
                                <li key={folder.path} className="sidebar-list-item">
                                    <button
                                        className="sidebar-item-button"
                                        onClick={() => onNavigate(folder.path)}
                                    >
                                        <div className="sidebar-item-icon">
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                strokeWidth="2"
                                                strokeLinecap="round"
                                                strokeLinejoin="round"
                                                width="18"
                                                height="18"
                                            >
                                                <path d="M10 3H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h16a1 1 0 0 0 1-1V7a1 1 0 0 0-1-1h-8l-2-2z" />
                                            </svg>
                                        </div>
                                        <span className="sidebar-item-label">{folder.name}</span>
                                    </button>
                                </li>
                            ))}
                        </ul>

                        <h3 className="sidebar-section-title">ZULETZT BESUCHT</h3>
                        <ul className="sidebar-list">
                            {recentPaths.map(path => (
                                <li key={path} className="sidebar-list-item">
                                    <button
                                        className="sidebar-item-button"
                                        onClick={() => onNavigate(path)}
                                    >
                                        <div className="sidebar-item-icon">
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                strokeWidth="2"
                                                strokeLinecap="round"
                                                strokeLinejoin="round"
                                                width="18"
                                                height="18"
                                            >
                                                <path d="M12 8v4l3 3M3 12a9 9 0 1 0 18 0 9 9 0 0 0-18 0z" />
                                            </svg>
                                        </div>
                                        <span className="sidebar-item-label text-truncate" title={path}>
                      {formatDisplayPath(path)}
                    </span>
                                    </button>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                {activeTab === 'thisPC' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">DATENTRÄGER</h3>
                        <ul className="sidebar-list">
                            {dataSources.map(source => (
                                <li key={source.path} className="sidebar-list-item">
                                    <button
                                        className="sidebar-item-button"
                                        onClick={() => onNavigate(source.path)}
                                    >
                                        <div className="sidebar-item-icon">
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                strokeWidth="2"
                                                strokeLinecap="round"
                                                strokeLinejoin="round"
                                                width="18"
                                                height="18"
                                            >
                                                <path d="M3 3h18v18H3zM3 9h18M3 15h18M9 3v18" />
                                            </svg>
                                        </div>
                                        <span className="sidebar-item-label">{source.name}</span>
                                    </button>
                                    {source.freeSpace && (
                                        <div className="sidebar-item-details">
                                            <div className="storage-bar">
                                                <div
                                                    className="storage-bar-used"
                                                    style={{
                                                        width: `${100 - (parseInt(source.freeSpace) / parseInt(source.totalSpace) * 100)}%`
                                                    }}
                                                ></div>
                                            </div>
                                            <span className="storage-info">
                        {source.freeSpace} frei von {source.totalSpace}
                      </span>
                                        </div>
                                    )}
                                </li>
                            ))}
                        </ul>

                        {/* Quelle hinzufügen */}
                        {isAddingSource ? (
                            <div className="add-source-form">
                                <input
                                    type="text"
                                    className="add-source-input"
                                    value={newSourcePath}
                                    onChange={(e) => setNewSourcePath(e.target.value)}
                                    placeholder="Pfad eingeben..."
                                />
                                <div className="add-source-buttons">
                                    <button
                                        className="add-source-button-cancel"
                                        onClick={() => setIsAddingSource(false)}
                                    >
                                        Abbrechen
                                    </button>
                                    <button
                                        className="add-source-button-add"
                                        onClick={handleAddSource}
                                    >
                                        Hinzufügen
                                    </button>
                                </div>
                            </div>
                        ) : (
                            <button
                                className="add-source-button"
                                onClick={() => setIsAddingSource(true)}
                            >
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
                                >
                                    <path d="M12 5v14M5 12h14" />
                                </svg>
                                <span>Quelle hinzufügen</span>
                            </button>
                        )}
                    </div>
                )}

                {activeTab === 'favorites' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">FAVORITEN</h3>
                        {favorites.length > 0 ? (
                            <ul className="sidebar-list">
                                {favorites.map(path => (
                                    <li key={path} className="sidebar-list-item">
                                        <button
                                            className="sidebar-item-button"
                                            onClick={() => onNavigate(path)}
                                        >
                                            <div className="sidebar-item-icon">
                                                <svg
                                                    xmlns="http://www.w3.org/2000/svg"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    strokeWidth="2"
                                                    strokeLinecap="round"
                                                    strokeLinejoin="round"
                                                    width="18"
                                                    height="18"
                                                >
                                                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14 2 9.27 8.91 8.26 12 2z" />
                                                </svg>
                                            </div>
                                            <span className="sidebar-item-label text-truncate" title={path}>
                        {formatDisplayPath(path)}
                      </span>
                                        </button>
                                        <button
                                            className="sidebar-item-action"
                                            onClick={() => onRemoveFavorite(path)}
                                            title="Aus Favoriten entfernen"
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
                                    </li>
                                ))}
                            </ul>
                        ) : (
                            <div className="sidebar-empty-state">
                                <p>Keine Favoriten vorhanden.</p>
                            </div>
                        )}
                    </div>
                )}

                {activeTab === 'templates' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">TEMPLATES</h3>
                        {/* [Backend Integration] - Templates vom Backend abrufen */}
                        {/* BACKEND_INTEGRATION: Templates laden */}
                            <div className="sidebar-empty-state">
                            <p>Templates werden geladen...</p>
                            <p className="sidebar-hint">
                            Speichern Sie Ordner und Dateien als Templates, um sie später wiederzuverwenden.
                            </p>
                            </div>
                            </div>
                            )}
                    </div>
                    </div>
                    );
                };

export default SideBar;