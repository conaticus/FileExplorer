import React, { useState, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useTheme } from '../../providers/ThemeProvider';

// Icon Components
const QuickAccessIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M5 5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5z" />
        <path d="M9 5v14" />
    </svg>
);

const ThisPCIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <path d="M8 21h8" />
        <path d="M12 17v4" />
        <path d="M7 10h10" />
    </svg>
);

const FavoritesIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14 2 9.27l6.91-1.01L12 2z" />
    </svg>
);

const TemplatesIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
        <polyline points="14 2 14 8 20 8" />
        <path d="M8 13h8" />
        <path d="M8 17h8" />
        <path d="M8 9h2" />
    </svg>
);

const NetworkIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="2" y="2" width="20" height="8" rx="2" />
        <rect x="2" y="14" width="20" height="8" rx="2" />
        <path d="M6 10v4" />
        <path d="M12 10v4" />
        <path d="M18 10v4" />
    </svg>
);

const ClockIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
    </svg>
);

const FolderIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
    </svg>
);

const DriveIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M22 12H2" />
        <path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
        <path d="M6 16h.01" />
        <path d="M10 16h.01" />
    </svg>
);

const AddIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 5v14M5 12h14" />
    </svg>
);

const CloseIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M18 6L6 18M6 6l12 12" />
    </svg>
);

const SideBar = ({
                     isOpen,
                     favorites = [],
                     recentPaths = [],
                     onToggle,
                     onNavigate,
                     onAddFavorite,
                     onRemoveFavorite,
                     enableGlassEffect = false
                 }) => {
    const { rootFolders, dataSources, addDataSource } = useFileSystem();
    const { themeSettings } = useTheme();

    const [activeTab, setActiveTab] = useState('quickAccess');
    const [isAddingSource, setIsAddingSource] = useState(false);
    const [newSourcePath, setNewSourcePath] = useState('');

    // Available tabs
    const tabs = [
        { id: 'quickAccess', label: 'Quick Access', icon: <QuickAccessIcon /> },
        { id: 'thisPC', label: 'This PC', icon: <ThisPCIcon /> },
        { id: 'favorites', label: 'Favorites', icon: <FavoritesIcon /> },
        { id: 'templates', label: 'Templates', icon: <TemplatesIcon /> },
        { id: 'network', label: 'Network', icon: <NetworkIcon /> },
    ];

    // Add a new path
    const handleAddSource = () => {
        if (newSourcePath) {
            addDataSource(newSourcePath);
            setNewSourcePath('');
            setIsAddingSource(false);
        }
    };

    // Format a path for display
    const formatDisplayPath = (path) => {
        if (!path) return '';
        const trimmedPath = path.replace(/[/\\]$/, '');
        const parts = trimmedPath.split(/[/\\]/);
        return parts[parts.length - 1] || path;
    };

    return (
        <div className={`sidebar ${!isOpen ? 'collapsed' : ''} ${enableGlassEffect ? 'glass-effect' : ''}`}>
            {/* Tabs */}
            <div className="sidebar-tabs">
                {tabs.map(tab => (
                    <button
                        key={tab.id}
                        className={`sidebar-tab ${activeTab === tab.id ? 'active' : ''}`}
                        onClick={() => setActiveTab(tab.id)}
                        title={tab.label}
                    >
                        {tab.icon}
                        <span className="sidebar-tab-label">{tab.label}</span>
                    </button>
                ))}
            </div>

            {/* Tab content */}
            <div className="sidebar-content">
                {activeTab === 'quickAccess' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">QUICK ACCESS</h3>
                        <ul className="sidebar-list">
                            {rootFolders.map(folder => (
                                <li key={folder.path} className="sidebar-list-item">
                                    <button
                                        className="sidebar-item-button"
                                        onClick={() => onNavigate(folder.path)}
                                    >
                                        <div className="sidebar-item-icon">
                                            <FolderIcon />
                                        </div>
                                        <span className="sidebar-item-label">{folder.name}</span>
                                    </button>
                                </li>
                            ))}
                        </ul>

                        <h3 className="sidebar-section-title">
              <span className="title-with-icon">
                <ClockIcon /> RECENT
              </span>
                        </h3>
                        <ul className="sidebar-list">
                            {recentPaths.length > 0 ? (
                                recentPaths.map(path => (
                                    <li key={path} className="sidebar-list-item">
                                        <button
                                            className="sidebar-item-button"
                                            onClick={() => onNavigate(path)}
                                        >
                                            <div className="sidebar-item-icon">
                                                <FolderIcon />
                                            </div>
                                            <span className="sidebar-item-label text-truncate" title={path}>
                        {formatDisplayPath(path)}
                      </span>
                                        </button>
                                    </li>
                                ))
                            ) : (
                                <li className="sidebar-empty-state">No recent folders</li>
                            )}
                        </ul>
                    </div>
                )}

                {activeTab === 'thisPC' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">DRIVES</h3>
                        <ul className="sidebar-list">
                            {dataSources.map(source => (
                                <li key={source.path} className="sidebar-list-item">
                                    <button
                                        className="sidebar-item-button"
                                        onClick={() => onNavigate(source.path)}
                                    >
                                        <div className="sidebar-item-icon">
                                            <DriveIcon />
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
                        {source.freeSpace} free of {source.totalSpace}
                      </span>
                                        </div>
                                    )}
                                </li>
                            ))}
                        </ul>

                        {/* Add Source */}
                        {isAddingSource ? (
                            <div className="add-source-form">
                                <input
                                    type="text"
                                    className="add-source-input"
                                    value={newSourcePath}
                                    onChange={(e) => setNewSourcePath(e.target.value)}
                                    placeholder="Enter path..."
                                    autoFocus
                                />
                                <div className="add-source-buttons">
                                    <button
                                        className="add-source-button-cancel"
                                        onClick={() => setIsAddingSource(false)}
                                    >
                                        <CloseIcon /> Cancel
                                    </button>
                                    <button
                                        className="add-source-button-add"
                                        onClick={handleAddSource}
                                    >
                                        <AddIcon /> Add
                                    </button>
                                </div>
                            </div>
                        ) : (
                            <button
                                className="add-source-button"
                                onClick={() => setIsAddingSource(true)}
                            >
                                <AddIcon />
                                <span>Add location</span>
                            </button>
                        )}
                    </div>
                )}

                {activeTab === 'favorites' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">
              <span className="title-with-icon">
                <FavoritesIcon /> FAVORITES
              </span>
                        </h3>
                        {favorites.length > 0 ? (
                            <ul className="sidebar-list">
                                {favorites.map(path => (
                                    <li key={path} className="sidebar-list-item">
                                        <button
                                            className="sidebar-item-button"
                                            onClick={() => onNavigate(path)}
                                        >
                                            <div className="sidebar-item-icon">
                                                <FolderIcon />
                                            </div>
                                            <span className="sidebar-item-label text-truncate" title={path}>
                        {formatDisplayPath(path)}
                      </span>
                                        </button>
                                        <button
                                            className="sidebar-item-action"
                                            onClick={() => onRemoveFavorite(path)}
                                            title="Remove from favorites"
                                        >
                                            <CloseIcon />
                                        </button>
                                    </li>
                                ))}
                            </ul>
                        ) : (
                            <div className="sidebar-empty-state">
                                <p>No favorites yet</p>
                                <p className="empty-state-hint">
                                    Right-click on a folder and select "Add to Favorites"
                                </p>
                            </div>
                        )}
                    </div>
                )}

                {activeTab === 'templates' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">
              <span className="title-with-icon">
                <TemplatesIcon /> TEMPLATES
              </span>
                        </h3>
                        <div className="sidebar-empty-state">
                            <p>No templates available</p>
                            <p className="empty-state-hint">
                                Save folders and files as templates to use them later
                            </p>
                        </div>
                    </div>
                )}

                {activeTab === 'network' && (
                    <div className="sidebar-section">
                        <h3 className="sidebar-section-title">
              <span className="title-with-icon">
                <NetworkIcon /> NETWORK
              </span>
                        </h3>
                        <div className="sidebar-empty-state">
                            <p>Network view is not implemented yet</p>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

export default SideBar;