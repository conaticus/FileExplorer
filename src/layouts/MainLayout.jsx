import React, { useState, useEffect } from 'react';
import { useAppState } from '../providers/AppStateProvider';
import { useFileSystem } from '../providers/FileSystemProvider';
import { useTheme } from '../providers/ThemeProvider';

// Komponenten importieren
import SideBar from '../components/panels/SideBar';
import DetailPanel from '../components/panels/DetailPanel';
import TerminalPanel from '../components/panels/TerminalPanel';
import StatusBar from '../components/panels/StatusBar';
import LocationBar from '../components/navigation/LocationBar';
import NavButtons from '../components/navigation/NavButtons';
import GlobalSearch from '../components/search/GlobalSearch';
import FileView from '../components/file-view/FileView';
import FileContextMenu from '../components/context-menu/FileContextMenu';

const MainLayout = () => {
    const { state, actions } = useAppState();
    const { isLoading, error } = useFileSystem();
    const { colors, themeSettings } = useTheme();

    const [items, setItems] = useState([]);
    const [isContextMenuOpen, setIsContextMenuOpen] = useState(false);
    const [contextMenuPosition, setContextMenuPosition] = useState({ x: 0, y: 0 });
    const [contextMenuTargetItem, setContextMenuTargetItem] = useState(null);

    // Set a default path if none is set
    useEffect(() => {
        if (!state.currentPath) {
            actions.setCurrentPath('C:\\Users\\User\\Documents');
        }
    }, [state.currentPath, actions]);

    // Lade Dateien und Ordner beim ersten Rendern und bei Änderung des Pfads
    useEffect(() => {
        const loadItems = async () => {
            if (!state.currentPath) return;

            actions.setLoading(true);
            try {
                // [Backend Integration] - Verzeichnisinhalt vom Backend abrufen
                // /* BACKEND_INTEGRATION: Verzeichnisinhalt laden */

                // Beispieldaten für die Anzeige
                const mockItems = [
                    { name: 'Bilder', path: `${state.currentPath}\\Bilder`, type: 'directory', modified: '2023-05-03T10:30:00Z' },
                    { name: 'Ordner1', path: `${state.currentPath}\\Ordner1`, type: 'directory', modified: '2023-01-05T11:20:00Z' },
                    { name: 'Ordner2', path: `${state.currentPath}\\Ordner2`, type: 'directory', modified: '2023-12-02T16:40:00Z' },
                    { name: 'config.json', path: `${state.currentPath}\\config.json`, type: 'file', size: '4 KB', modified: '2023-02-04T09:30:00Z' },
                    { name: 'Dokument1.docx', path: `${state.currentPath}\\Dokument1.docx`, type: 'file', size: '25 KB', modified: '2023-01-15T10:30:00Z' },
                    { name: 'Präsentation.pptx', path: `${state.currentPath}\\Präsentation.pptx`, type: 'file', size: '2.3 MB', modified: '2023-10-03T09:15:00Z' },
                    { name: 'Tabelle.xlsx', path: `${state.currentPath}\\Tabelle.xlsx`, type: 'file', size: '156 KB', modified: '2023-02-20T14:45:00Z' },
                    { name: 'test.txt', path: `${state.currentPath}\\test.txt`, type: 'file', size: '2 KB', modified: '2023-04-01T08:00:00Z' },
                ];

                setItems(mockItems);
            } catch (error) {
                console.error('Error loading directory contents:', error);
                actions.setError(error.message);
            } finally {
                actions.setLoading(false);
            }
        };

        loadItems();
    }, [state.currentPath, actions]);

    // Sortiere die Elemente
    const sortedItems = [...items].sort((a, b) => {
        // Sortiere Ordner vor Dateien
        if (a.type !== b.type) {
            return a.type === 'directory' ? -1 : 1;
        }

        // Sortiere nach dem ausgewählten Sortiermerkmal
        switch (state.sortBy) {
            case 'name':
                return state.sortDirection === 'asc'
                    ? a.name.localeCompare(b.name)
                    : b.name.localeCompare(a.name);

            case 'date':
                return state.sortDirection === 'asc'
                    ? new Date(a.modified) - new Date(b.modified)
                    : new Date(b.modified) - new Date(a.modified);

            case 'size':
                // Nur für Dateien relevant
                if (a.type === 'directory' && b.type === 'directory') {
                    return state.sortDirection === 'asc'
                        ? a.name.localeCompare(b.name)
                        : b.name.localeCompare(a.name);
                }

                // Größe parsen (entfernen von "KB", "MB", etc.)
                const getSizeInBytes = (sizeStr) => {
                    if (!sizeStr) return 0;
                    const num = parseFloat(sizeStr);
                    if (sizeStr.includes('KB')) return num * 1024;
                    if (sizeStr.includes('MB')) return num * 1024 * 1024;
                    if (sizeStr.includes('GB')) return num * 1024 * 1024 * 1024;
                    return num;
                };

                return state.sortDirection === 'asc'
                    ? getSizeInBytes(a.size) - getSizeInBytes(b.size)
                    : getSizeInBytes(b.size) - getSizeInBytes(a.size);

            case 'type':
                // Dateierweiterung extrahieren
                const getExtension = (filename) => {
                    if (!filename || !filename.includes('.')) return '';
                    return filename.split('.').pop().toLowerCase();
                };

                return state.sortDirection === 'asc'
                    ? getExtension(a.name).localeCompare(getExtension(b.name))
                    : getExtension(b.name).localeCompare(getExtension(a.name));

            default:
                return 0;
        }
    });

    // Öffne das Kontextmenü
    const handleContextMenu = (e, item) => {
        e.preventDefault();
        setContextMenuPosition({ x: e.clientX, y: e.clientY });
        setContextMenuTargetItem(item);
        setIsContextMenuOpen(true);
    };

    // Schließe das Kontextmenü
    const closeContextMenu = () => {
        setIsContextMenuOpen(false);
    };

    // Klick auf eine Datei oder einen Ordner
    const handleItemClick = (item, isDoubleClick = false) => {
        // Wenn es ein Verzeichnis ist und ein Doppelklick erfolgt, öffne das Verzeichnis
        if (item.type === 'directory' && isDoubleClick) {
            actions.setCurrentPath(item.path);
        }
        // Wenn es eine Datei ist und ein Doppelklick erfolgt, öffne die Datei
        else if (item.type === 'file' && isDoubleClick) {
            // [Backend Integration] - Datei mit Standardanwendung öffnen
            // /* BACKEND_INTEGRATION: Datei öffnen */
            console.log(`Opening file: ${item.path}`);
        }
        // Bei einfachem Klick wähle das Element aus
        else {
            // Prüfe, ob Strg-Taste gedrückt ist (für Mehrfachauswahl)
            const isCtrlPressed = false; // TODO: Implementiere Abfrage für Strg-Taste

            if (isCtrlPressed) {
                // Füge das Element zur Auswahl hinzu oder entferne es, wenn es bereits ausgewählt ist
                if (state.selectedItems.includes(item.path)) {
                    actions.removeSelectedItem(item.path);
                } else {
                    actions.addSelectedItem(item.path);
                }
            } else {
                // Setze die Auswahl auf dieses Element
                actions.setSelectedItems([item.path]);
            }
        }
    };

    // Aktualisieren bei Änderungen des Sortiermerkmals oder der Sortierrichtung
    const handleSortChange = (sortBy) => {
        if (state.sortBy === sortBy) {
            // Ändere die Sortierrichtung, wenn das gleiche Merkmal erneut ausgewählt wird
            actions.setSortDirection(state.sortDirection === 'asc' ? 'desc' : 'asc');
        } else {
            // Setze das neue Sortiermerkmal und die Standardrichtung (aufsteigend)
            actions.setSortBy(sortBy);
            actions.setSortDirection('asc');
        }
    };

    // Mock data for sidebar (mit abhängigkeiten im dependency array)
    useEffect(() => {
        if (actions.addRecent && actions.addFavorite) {
            actions.addRecent('C:\\Users\\User\\Documents');
            actions.addRecent('C:\\Users\\User\\Pictures');
            actions.addRecent('C:\\Users\\User\\Desktop');
            actions.addFavorite('C:\\Users\\User\\Documents');
            actions.addFavorite('C:\\Users\\User\\Pictures');
        }
    }, [actions]);

    return (
        <div className="explorer-layout">
            {/* Obere Navigationsleiste */}
            <div className="explorer-header">
                <NavButtons
                    canGoBack={state.historyIndex > 0}
                    canGoForward={state.historyIndex < state.history.length - 1}
                    onGoBack={actions.goBack}
                    onGoForward={actions.goForward}
                />
                <LocationBar
                    currentPath={state.currentPath}
                    onPathChange={actions.setCurrentPath}
                />
                <GlobalSearch
                    isSearchActive={state.isSearchActive}
                    searchQuery={state.searchQuery}
                    onSearch={actions.search}
                    onClearSearch={actions.clearSearch}
                />
            </div>

            {/* Hauptbereich */}
            <div className="main-container">
                {/* Seitenleiste */}
                <SideBar
                    isOpen={state.isSidebarOpen}
                    favorites={state.favorites}
                    recentPaths={state.recentPaths}
                    onToggle={() => actions.toggleSidebar()}
                    onNavigate={actions.setCurrentPath}
                    onAddFavorite={actions.addFavorite}
                    onRemoveFavorite={actions.removeFavorite}
                />

                {/* Hauptinhalt */}
                <div className="content-area">
                    <FileView
                        items={sortedItems}
                        viewMode={state.viewMode || themeSettings.defaultView}
                        selectedItems={state.selectedItems}
                        onItemClick={handleItemClick}
                        onContextMenu={handleContextMenu}
                        onSortChange={handleSortChange}
                        sortBy={state.sortBy}
                        sortDirection={state.sortDirection}
                        isLoading={isLoading}
                        error={error}
                    />
                </div>

                {/* Detailansicht (rechts) */}
                <DetailPanel
                    isOpen={state.isDetailPanelOpen}
                    selectedItems={state.selectedItems}
                    onClose={() => actions.toggleDetailPanel(false)}
                />
            </div>

            {/* Terminal (unten) */}
            <TerminalPanel
                isOpen={state.isTerminalPanelOpen}
                currentPath={state.currentPath}
                onClose={() => actions.toggleTerminalPanel(false)}
            />

            {/* Statusleiste */}
            <StatusBar
                selectedItems={state.selectedItems}
                currentPath={state.currentPath}
                isLoading={isLoading}
            />

            {/* Kontextmenü */}
            {isContextMenuOpen && (
                <FileContextMenu
                    position={contextMenuPosition}
                    targetItem={contextMenuTargetItem}
                    selectedItems={state.selectedItems}
                    onClose={closeContextMenu}
                    inEmptySpace={!contextMenuTargetItem}
                    currentPath={state.currentPath}
                />
            )}
        </div>
    );
};

export default MainLayout;