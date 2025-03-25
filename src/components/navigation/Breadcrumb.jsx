import React from 'react';

const Breadcrumb = ({ path, onNavigate }) => {
    // Wenn kein Pfad vorhanden ist, zeige einen Platzhalter an
    if (!path) {
        return <div className="breadcrumb-placeholder">Kein Pfad ausgewählt</div>;
    }

    // Funktionen zur Pfadverarbeitung
    const isWindowsPath = path.includes('\\') || /^[A-Z]:/.test(path);
    const separator = isWindowsPath ? '\\' : '/';

    // Normalisiere den Pfad für die Verarbeitung
    const normalizedPath = path.replace(/\\/g, '/');

    // Teile den Pfad in Segmente auf
    const segments = normalizedPath.split('/').filter(segment => segment);

    // Erstelle Breadcrumb-Elemente
    const breadcrumbItems = [];

    // Für Windows-Pfade, füge das Laufwerk hinzu
    if (isWindowsPath && normalizedPath.match(/^[A-Z]:/)) {
        const drive = normalizedPath.substring(0, 2) + '\\';
        breadcrumbItems.push(
            <div
                key="drive"
                className="breadcrumb-item"
                onClick={() => onNavigate(drive)}
            >
                {drive}
            </div>
        );
    } else if (!isWindowsPath && normalizedPath.startsWith('/')) {
        // Für Unix-Pfade, füge das Root-Verzeichnis hinzu
        breadcrumbItems.push(
            <div
                key="root"
                className="breadcrumb-item"
                onClick={() => onNavigate('/')}
            >
                /
            </div>
        );
    }

    // Füge die Pfadsegmente hinzu
    segments.forEach((segment, index) => {
        // Erstelle den Pfad bis zu diesem Segment
        const segmentPath = isWindowsPath
            ? segments.slice(0, index + 1).join('\\')
            : '/' + segments.slice(0, index + 1).join('/');

        // Füge einen Separator hinzu, wenn es nicht das erste Element ist
        if (index > 0 || breadcrumbItems.length > 0) {
            breadcrumbItems.push(
                <div key={`separator-${index}`} className="breadcrumb-separator">
                    {separator}
                </div>
            );
        }

        // Füge das Segment hinzu
        breadcrumbItems.push(
            <div
                key={segmentPath}
                className="breadcrumb-item"
                onClick={() => onNavigate(segmentPath)}
                title={segment}
            >
                {segment}
            </div>
        );
    });

    return <div className="breadcrumb">{breadcrumbItems}</div>;
};

export default Breadcrumb;