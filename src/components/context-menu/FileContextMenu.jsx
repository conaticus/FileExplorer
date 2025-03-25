import React from 'react';
import ContextMenu from './ContextMenu';
import { useAppState } from '../../providers/AppStateProvider';

const FileContextMenu = ({
                             position,
                             targetItem,
                             selectedItems = [],
                             onClose,
                             inEmptySpace = false,
                             currentPath = '',
                         }) => {
    const { actions } = useAppState();

    // Menüelemente für Dateien und Ordner
    const getFileMenuItems = () => {
        const isMultipleSelection = selectedItems.length > 1;
        const isDirectory = targetItem?.type === 'directory';

        // Gemeinsame Aktionen für Dateien und Ordner
        const commonItems = [
            {
                id: 'open',
                label: isDirectory ? 'Öffnen' : 'Öffnen',
                icon: 'open',
                group: 'action',
                onClick: () => {
                    if (isDirectory) {
                        actions.setCurrentPath(targetItem.path);
                    } else {
                        // [Backend Integration] - Datei öffnen
                        // /* BACKEND_INTEGRATION: Datei öffnen */
                        console.log('Open file:', targetItem.path);
                    }
                },
            },
            {
                id: 'copy',
                label: 'Kopieren',
                icon: 'copy',
                group: 'clipboard',
                onClick: () => {
                    const itemsToCopy = isMultipleSelection ? selectedItems : [targetItem.path];
                    // [Backend Integration] - In die Zwischenablage kopieren
                    // /* BACKEND_INTEGRATION: In Zwischenablage kopieren */
                    console.log('Copy items:', itemsToCopy);
                },
            },
            {
                id: 'cut',
                label: 'Ausschneiden',
                icon: 'cut',
                group: 'clipboard',
                onClick: () => {
                    const itemsToCut = isMultipleSelection ? selectedItems : [targetItem.path];
                    // [Backend Integration] - In die Zwischenablage ausschneiden
                    // /* BACKEND_INTEGRATION: In Zwischenablage ausschneiden */
                    console.log('Cut items:', itemsToCut);
                },
            },
            {
                id: 'delete',
                label: 'Löschen',
                icon: 'delete',
                group: 'edit',
                onClick: () => {
                    const itemsToDelete = isMultipleSelection ? selectedItems : [targetItem.path];
                    // [Backend Integration] - Elemente löschen
                    // /* BACKEND_INTEGRATION: Elemente löschen */
                    console.log('Delete items:', itemsToDelete);

                    // Bestätigungsdialog anzeigen
                    if (confirm(`${itemsToDelete.length} Element(e) wirklich löschen?`)) {
                        actions.deleteItems(itemsToDelete);
                    }
                },
            },
            {
                id: 'rename',
                label: 'Umbenennen',
                icon: 'rename',
                group: 'edit',
                disabled: isMultipleSelection, // Umbenennen nur für einzelne Elemente
                onClick: () => {
                    const newName = prompt('Neuen Namen eingeben:', targetItem.name);
                    if (newName && newName !== targetItem.name) {
                        actions.renameItem(targetItem.path, newName);
                    }
                },
            },
        ];

        // Ordnerspezifische Aktionen
        const directoryItems = isDirectory ? [
            {
                id: 'newFile',
                label: 'Neue Datei',
                icon: 'file',
                group: 'new',
                onClick: () => {
                    const newFileName = prompt('Name der neuen Datei:', 'Neue Datei.txt');
                    if (newFileName) {
                        actions.createFile(targetItem.path, newFileName);
                    }
                },
            },
            {
                id: 'newFolder',
                label: 'Neuer Ordner',
                icon: 'folder',
                group: 'new',
                onClick: () => {
                    const newFolderName = prompt('Name des neuen Ordners:', 'Neuer Ordner');
                    if (newFolderName) {
                        actions.createFolder(targetItem.path, newFolderName);
                    }
                },
            },
        ] : [];

        // Template-Aktionen
        const templateItems = [
            {
                id: 'saveAsTemplate',
                label: 'Als Template speichern',
                icon: 'template',
                group: 'template',
                onClick: () => {
                    const templateName = prompt('Name des Templates:', targetItem.name);
                    if (templateName) {
                        actions.saveTemplate(targetItem.path, templateName);
                    }
                },
            },
        ];

        // Favoriten-Aktionen
        const favoriteItems = isDirectory ? [
            {
                id: 'addToFavorites',
                label: 'Zu Favoriten hinzufügen',
                icon: 'pin',
                group: 'favorite',
                onClick: () => {
                    actions.addFavorite(targetItem.path);
                },
            },
        ] : [];

        // Eigenschaften-Aktionen
        const propertyItems = [
            {
                id: 'properties',
                label: 'Eigenschaften',
                icon: 'properties',
                group: 'property',
                onClick: () => {
                    // Öffne das Detailpanel
                    actions.toggleDetailPanel(true);
                },
            },
        ];

        return [
            ...commonItems,
            ...directoryItems,
            ...templateItems,
            ...favoriteItems,
            ...propertyItems,
        ];
    };

    // Menüelemente für den leeren Bereich
    const getEmptySpaceMenuItems = () => {
        return [
            {
                id: 'refresh',
                label: 'Aktualisieren',
                icon: 'M1 4v6h6M23 20v-6h-6M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15',
                group: 'action',
                onClick: () => {
                    // Aktualisiere die Ansicht
                    console.log('Refresh view');
                },
            },
            {
                id: 'newFile',
                label: 'Neue Datei',
                icon: 'file',
                group: 'new',
                onClick: () => {
                    const newFileName = prompt('Name der neuen Datei:', 'Neue Datei.txt');
                    if (newFileName) {
                        actions.createFile(currentPath, newFileName);
                    }
                },
            },
            {
                id: 'newFolder',
                label: 'Neuer Ordner',
                icon: 'folder',
                group: 'new',
                onClick: () => {
                    const newFolderName = prompt('Name des neuen Ordners:', 'Neuer Ordner');
                    if (newFolderName) {
                        actions.createFolder(currentPath, newFolderName);
                    }
                },
            },
            {
                id: 'paste',
                label: 'Einfügen',
                icon: 'paste',
                group: 'clipboard',
                // Deaktiviert, wenn nichts in der Zwischenablage ist
                disabled: true, // TODO: Implementiere die Prüfung für die Zwischenablage
                onClick: () => {
                    // [Backend Integration] - Aus der Zwischenablage einfügen
                    // /* BACKEND_INTEGRATION: Aus Zwischenablage einfügen */
                    console.log('Paste items to:', currentPath);
                },
            },
            {
                id: 'selectAll',
                label: 'Alles auswählen',
                icon: 'M3 5h18M3 12h18M3 19h18',
                group: 'edit',
                onClick: () => {
                    // Alle Elemente auswählen
                    // [Backend Integration] - Alle Elemente im aktuellen Verzeichnis auswählen
                    // /* BACKEND_INTEGRATION: Alle Elemente auswählen */
                    console.log('Select all items');
                },
            },
            {
                id: 'properties',
                label: 'Eigenschaften',
                icon: 'properties',
                group: 'property',
                onClick: () => {
                    // Öffne die Eigenschaften des aktuellen Ordners
                    // [Backend Integration] - Eigenschaften des aktuellen Ordners anzeigen
                    // /* BACKEND_INTEGRATION: Eigenschaften des Ordners anzeigen */
                    console.log('Show properties for:', currentPath);
                },
            },
        ];
    };

    const menuItems = inEmptySpace
        ? getEmptySpaceMenuItems()
        : getFileMenuItems();

    return (
        <ContextMenu
            position={position}
            items={menuItems}
            onClose={onClose}
        />
    );
};

export default FileContextMenu;