import React from 'react';
import FileIcon from './FileIcon';

const FileTable = ({
                       items = [],
                       selectedItems = [],
                       onItemClick,
                       onContextMenu,
                       sortBy = 'name',
                       sortDirection = 'asc',
                       onSortChange
                   }) => {
    // Sortierrichtung anzeigen
    const getSortIndicator = (column) => {
        if (sortBy !== column) return null;
        return sortDirection === 'asc' ? '↑' : '↓';
    };

    // Funktion zum Formatieren des Datums
    const formatDate = (dateString) => {
        if (!dateString) return '';
        const date = new Date(dateString);
        return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
    };

    return (
        <div className="file-table-container">
            <table className="file-table">
                <thead>
                <tr>
                    <th className="file-table-icon-cell"></th>
                    <th
                        className="file-table-name-cell"
                        onClick={() => onSortChange('name')}
                    >
                        Name {getSortIndicator('name')}
                    </th>
                    <th
                        className="file-table-date-cell"
                        onClick={() => onSortChange('date')}
                    >
                        Datum geändert {getSortIndicator('date')}
                    </th>
                    <th
                        className="file-table-type-cell"
                        onClick={() => onSortChange('type')}
                    >
                        Typ {getSortIndicator('type')}
                    </th>
                    <th
                        className="file-table-size-cell"
                        onClick={() => onSortChange('size')}
                    >
                        Größe {getSortIndicator('size')}
                    </th>
                </tr>
                </thead>
                <tbody>
                {items.map((item) => {
                    const isSelected = selectedItems.includes(item.path);
                    const fileExtension = item.name.includes('.') ? item.name.split('.').pop() : '';
                    const fileType = item.type === 'directory'
                        ? 'Ordner'
                        : fileExtension
                            ? `${fileExtension.toUpperCase()}-Datei`
                            : 'Datei';

                    return (
                        <tr
                            key={item.path}
                            className={`file-table-row ${isSelected ? 'selected' : ''}`}
                            onClick={() => onItemClick(item)}
                            onDoubleClick={() => onItemClick(item, true)}
                            onContextMenu={(e) => onContextMenu(e, item)}
                        >
                            <td className="file-table-icon-cell">
                                <FileIcon fileType={item.type} extension={fileExtension} />
                            </td>
                            <td className="file-table-name-cell">{item.name}</td>
                            <td className="file-table-date-cell">{formatDate(item.modified)}</td>
                            <td className="file-table-type-cell">{fileType}</td>
                            <td className="file-table-size-cell">
                                {item.type === 'directory' ? '--' : item.size}
                            </td>
                        </tr>
                    );
                })}
                </tbody>
            </table>
        </div>
    );
};

export default FileTable;