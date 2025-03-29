import React, { useState } from 'react';
import FileGrid from './FileGrid';
import FileList from './FileList';
import FileTable from './FileTable';
import ViewModeSelector from './ViewModeSelector';
import SortControls from './SortControls';
import { useTheme } from '../../providers/ThemeProvider';

const FileView = ({
                      items = [],
                      viewMode = 'list',
                      selectedItems = [],
                      onItemClick,
                      onContextMenu,
                      onSortChange,
                      sortBy = 'name',
                      sortDirection = 'asc',
                      isLoading = false,
                      error = null,
                  }) => {
    const { themeSettings } = useTheme();
    const [localViewMode, setLocalViewMode] = useState(viewMode);

    const handleViewModeChange = (mode) => {
        setLocalViewMode(mode);
    };

    // Render-Funktion für die entsprechende Ansicht
    const renderView = () => {
        if (isLoading) {
            return (
                <div className="loading-container">
                    <div className="loading-spinner"></div>
                    <p>Lade Inhalte...</p>
                </div>
            );
        }

        if (error) {
            return (
                <div className="error-container">
                    <p className="error-message">Fehler beim Laden: {error}</p>
                </div>
            );
        }

        if (items.length === 0) {
            return (
                <div className="empty-directory">
                    <p>Dieser Ordner ist leer.</p>
                </div>
            );
        }

        switch (localViewMode) {
            case 'grid':
                return (
                    <FileGrid
                        items={items}
                        selectedItems={selectedItems}
                        onItemClick={onItemClick}
                        onContextMenu={onContextMenu}
                        iconSize={themeSettings.iconSize}
                    />
                );
            case 'details':
                return (
                    <FileTable
                        items={items}
                        selectedItems={selectedItems}
                        onItemClick={onItemClick}
                        onContextMenu={onContextMenu}
                        sortBy={sortBy}
                        sortDirection={sortDirection}
                        onSortChange={onSortChange}
                    />
                );
            case 'list':
            default:
                return (
                    <FileList
                        items={items}
                        selectedItems={selectedItems}
                        onItemClick={onItemClick}
                        onContextMenu={onContextMenu}
                    />
                );
        }
    };

    return (
        <div className="file-view-container">
            <div className="file-view-toolbar">
                {/* Ansichtsmodus-Auswahl */}
                <ViewModeSelector
                    currentMode={localViewMode}
                    onChange={handleViewModeChange}
                />

                {/* Sortierkontrollen */}
                <SortControls
                    sortBy={sortBy}
                    sortDirection={sortDirection}
                    onChange={onSortChange}
                />
            </div>

            {/* Dateiansicht */}
            <div className="file-view-content">
                {renderView()}
            </div>
        </div>
    );
};

export default FileView;