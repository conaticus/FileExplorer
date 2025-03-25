import React from 'react';
import FileIcon from './FileIcon';

const FileGrid = ({
                      items = [],
                      selectedItems = [],
                      onItemClick,
                      onContextMenu,
                      iconSize = 'medium'
                  }) => {
    // Bestimme die Icon-Größe basierend auf der Einstellung
    const getIconSizeClass = () => {
        switch (iconSize) {
            case 'small': return 'file-grid-small';
            case 'large': return 'file-grid-large';
            case 'medium':
            default: return 'file-grid-medium';
        }
    };

    // Format dates for display
    const formatDate = (dateString) => {
        if (!dateString) return '';
        const date = new Date(dateString);
        return date.toLocaleDateString();
    };

    return (
        <div className={`file-grid ${getIconSizeClass()}`}>
            {items.map((item) => {
                const isSelected = selectedItems.includes(item.path);
                const extension = item.name.split('.').pop();

                return (
                    <div
                        key={item.path}
                        className={`file-grid-item ${isSelected ? 'selected' : ''}`}
                        onClick={() => onItemClick(item)}
                        onDoubleClick={() => onItemClick(item, true)}
                        onContextMenu={(e) => onContextMenu(e, item)}
                    >
                        <div className="file-grid-icon">
                            <FileIcon fileType={item.type} extension={extension} />
                        </div>
                        <div className="file-grid-name text-truncate" title={item.name}>
                            {item.name}
                        </div>
                        {item.modified && (
                            <div className="file-grid-date">
                                {formatDate(item.modified)}
                            </div>
                        )}
                    </div>
                );
            })}
        </div>
    );
};

export default FileGrid;