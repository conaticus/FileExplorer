import React from 'react';
import FileIcon from './FileIcon';

const FileList = ({
                      items = [],
                      selectedItems = [],
                      onItemClick,
                      onContextMenu
                  }) => {
    // Format date for display
    const formatDate = (dateString) => {
        if (!dateString) return '';
        const date = new Date(dateString);
        return date.toLocaleDateString();
    };

    return (
        <div className="file-list">
            {items.map((item) => {
                const isSelected = selectedItems.includes(item.path);
                const extension = item.name.includes('.') ? item.name.split('.').pop() : '';

                return (
                    <div
                        key={item.path}
                        className={`file-list-item ${isSelected ? 'selected' : ''}`}
                        onClick={() => onItemClick(item)}
                        onDoubleClick={() => onItemClick(item, true)}
                        onContextMenu={(e) => onContextMenu(e, item)}
                    >
                        <div className="file-list-icon">
                            <FileIcon fileType={item.type} extension={extension} />
                        </div>
                        <div className="file-list-details">
                            <div className="file-list-name text-truncate" title={item.name}>
                                {item.name}
                            </div>
                            <div className="file-list-meta">
                                {item.type === 'file' && item.size && (
                                    <span className="file-list-size">{item.size}</span>
                                )}
                                {item.modified && (
                                    <span className="file-list-date">
                                        {formatDate(item.modified)}
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>
                );
            })}
        </div>
    );
};

export default FileList;