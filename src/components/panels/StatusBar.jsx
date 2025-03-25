import React from 'react';

const StatusBar = ({ selectedItems = [], currentPath, isLoading }) => {
    // Berechne Informationen für die Statusleiste
    const getStatusInfo = () => {
        if (isLoading) {
            return 'Lade...';
        }

        if (selectedItems.length === 0) {
            return 'Keine Elemente ausgewählt';
        } else if (selectedItems.length === 1) {
            return `1 Element ausgewählt: ${selectedItems[0].split(/[\\/]/).pop()}`;
        } else {
            return `${selectedItems.length} Elemente ausgewählt`;
        }
    };

    // Berechne freien Speicherplatz (simuliert)
    const getFreeSpace = () => {
        // [Backend Integration] - Freien Speicherplatz vom Backend abrufen
        // /* BACKEND_INTEGRATION: Freien Speicherplatz abrufen */

        // Beispieldaten
        if (currentPath && currentPath.startsWith('C:')) {
            return '120 GB frei von 256 GB';
        } else if (currentPath && currentPath.startsWith('D:')) {
            return '750 GB frei von 1 TB';
        }

        return 'Speicherinfo nicht verfügbar';
    };

    return (
        <div className="status-bar">
            <div className="status-bar-left">
                {getStatusInfo()}
            </div>

            <div className="status-bar-center">
                {isLoading && (
                    <div className="status-loading-indicator">
                        <div className="loading-dots">
                            <span></span>
                            <span></span>
                            <span></span>
                        </div>
                    </div>
                )}
            </div>

            <div className="status-bar-right">
                {currentPath && getFreeSpace()}
            </div>
        </div>
    );
};

export default StatusBar;