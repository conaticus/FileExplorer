import React, { useState, useRef } from 'react';
import FileIcon from './FileIcon';
import { formatFileSize, formatDate, getFileType } from '../../utils/formatters';
import './fileItem.css';

const FileItem = ({
                      item,
                      viewMode = 'grid',
                      isSelected = false,
                      onClick,
                      onDoubleClick,
                      onContextMenu
                  }) => {
    const [isRenaming, setIsRenaming] = useState(false);
    const [newName, setNewName] = useState(item.name);
    const inputRef = useRef(null);

    const isDirectory = item.isDirectory || 'sub_file_count' in item;
    const fileType = isDirectory ? 'Folder' : getFileType(item.name);
    const size = isDirectory
        ? `${item.sub_file_count || 0} items`
        : formatFileSize(item.size_in_bytes);

    // Format modified date
    const modified = formatDate(item.last_modified);

    // Handle rename input changes
    const handleRenameChange = (e) => {
        setNewName(e.target.value);
    };

    // Handle rename form submission
    const handleRenameSubmit = (e) => {
        e.preventDefault();
        setIsRenaming(false);

        // Here you would handle the actual rename operation
        // by calling your API/backend
        console.log(`Rename ${item.name} to ${newName}`);
    };

    // Handle keydown events for the rename input
    const handleRenameKeyDown = (e) => {
        if (e.key === 'Escape') {
            e.preventDefault();
            setIsRenaming(false);
            setNewName(item.name);
        }
    };

    // Handle clicking the file item
    const handleClick = (e) => {
        if (isRenaming) return;
        if (onClick) onClick(e);
    };

    // Handle double clicking the file item
    const handleDoubleClick = (e) => {
        if (isRenaming) return;
        if (onDoubleClick) onDoubleClick(e);
    };

    // Handle right-clicking the file item
    const handleContextMenu = (e) => {
        if (isRenaming) return;
        if (onContextMenu) onContextMenu(e);
    };

    // Focus input when renaming starts
    React.useEffect(() => {
        if (isRenaming && inputRef.current) {
            inputRef.current.focus();
            // Select name without extension
            const lastDotIndex = item.name.lastIndexOf('.');
            if (lastDotIndex > 0 && !isDirectory) {
                inputRef.current.setSelectionRange(0, lastDotIndex);
            } else {
                inputRef.current.select();
            }
        }
    }, [isRenaming, item.name, isDirectory]);

    return (
        <div
            className={`file-item view-mode-${viewMode} ${isSelected ? 'selected' : ''} ${isDirectory ? 'directory' : 'file'}`}
            onClick={handleClick}
            onDoubleClick={handleDoubleClick}
            onContextMenu={handleContextMenu}
            data-path={item.path}
        >
            {viewMode === 'grid' && (
                <div className="file-item-grid">
                    <div className="file-icon-container">
                        <FileIcon filename={item.name} isDirectory={isDirectory} />
                    </div>

                    {isRenaming ? (
                        <form onSubmit={handleRenameSubmit} className="rename-form">
                            <input
                                ref={inputRef}
                                type="text"
                                value={newName}
                                onChange={handleRenameChange}
                                onKeyDown={handleRenameKeyDown}
                                onBlur={() => setIsRenaming(false)}
                                className="rename-input"
                            />
                        </form>
                    ) : (
                        <div className="file-name truncate" title={item.name}>
                            {item.name}
                        </div>
                    )}
                </div>
            )}

            {viewMode === 'list' && (
                <div className="file-item-list">
                    <div className="file-icon-container">
                        <FileIcon filename={item.name} isDirectory={isDirectory} />
                    </div>

                    {isRenaming ? (
                        <form onSubmit={handleRenameSubmit} className="rename-form flex-grow">
                            <input
                                ref={inputRef}
                                type="text"
                                value={newName}
                                onChange={handleRenameChange}
                                onKeyDown={handleRenameKeyDown}
                                onBlur={() => setIsRenaming(false)}
                                className="rename-input"
                            />
                        </form>
                    ) : (
                        <div className="file-details">
                            <div className="file-name truncate" title={item.name}>
                                {item.name}
                            </div>
                            <div className="file-info truncate">
                                {size} • {modified}
                            </div>
                        </div>
                    )}
                </div>
            )}

            {viewMode === 'details' && (
                <div className="file-item-details">
                    <div className="file-column column-name">
                        <div className="file-icon-container">
                            <FileIcon filename={item.name} isDirectory={isDirectory} />
                        </div>

                        {isRenaming ? (
                            <form onSubmit={handleRenameSubmit} className="rename-form flex-grow">
                                <input
                                    ref={inputRef}
                                    type="text"
                                    value={newName}
                                    onChange={handleRenameChange}
                                    onKeyDown={handleRenameKeyDown}
                                    onBlur={() => setIsRenaming(false)}
                                    className="rename-input"
                                />
                            </form>
                        ) : (
                            <div className="file-name truncate" title={item.name}>
                                {item.name}
                            </div>
                        )}
                    </div>

                    <div className="file-column column-size">
                        {size}
                    </div>

                    <div className="file-column column-type">
                        {fileType}
                    </div>

                    <div className="file-column column-modified">
                        {modified}
                    </div>
                </div>
            )}
        </div>
    );
};

export default FileItem;