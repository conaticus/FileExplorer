import React, { useState, useRef, useEffect } from 'react';
import FileIcon from './FileIcon';
import RenameModal from '../common/RenameModal';
import { formatFileSize, formatDate, getFileType } from '../../utils/formatters';
import { replaceFileName } from '../../utils/pathUtils.js';
import { invoke } from '@tauri-apps/api/core';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { showError, showConfirm } from '../../utils/NotificationSystem';
import './fileItem.css';

const FileItem = ({
                      item,
                      viewMode = 'grid',
                      isSelected = false,
                      onClick,
                      onDoubleClick,
                      onContextMenu
                  }) => {
    const [isRenameModalOpen, setIsRenameModalOpen] = useState(false);
    const { loadDirectory } = useFileSystem();
    const { currentPath } = useHistory();

    const isDirectory = item.isDirectory || 'sub_file_count' in item;
    const fileType = isDirectory ? 'Folder' : getFileType(item.name);
    const size = isDirectory
        ? `${item.sub_file_count || 0} items`
        : formatFileSize(item.size_in_bytes);

    // Format modified date
    const modified = formatDate(item.last_modified);

    // Listen for rename modal open events
    useEffect(() => {
        const handleOpenRenameModal = (e) => {
            if (e.detail && e.detail.item && e.detail.item.path === item.path) {
                setIsRenameModalOpen(true);
            }
        };

        document.addEventListener('open-rename-modal', handleOpenRenameModal);

        return () => {
            document.removeEventListener('open-rename-modal', handleOpenRenameModal);
        };
    }, [item.path]);

    // Handle rename with robust path handling
    const handleRename = async (item, newName) => {
        if (!newName || newName === item.name) return;

        try {
            // Use the robust path utility to create the new path
            const newPath = replaceFileName(item.path, newName);

            console.log(`Renaming: "${item.path}" -> "${newPath}"`);

            await invoke('rename', {
                oldPath: item.path,
                newPath: newPath
            });

            console.log('Rename operation completed successfully');

            // Reload current directory
            if (currentPath) {
                await loadDirectory(currentPath);
            }
        } catch (error) {
            console.error('Rename operation failed:', error);
            if (error.message && error.message.includes('already exists')) {
                const shouldCreateCopy = await showConfirm(`A file named "${newName}" already exists. Create a copy instead?`, 'File Exists');
                if (shouldCreateCopy) {
                    const extension = newName.includes('.') ? newName.split('.').pop() : '';
                    const baseName = extension ? newName.replace(`.${extension}`, '') : newName;
                    const copyName = extension ? `${baseName} - Copy.${extension}` : `${baseName} - Copy`;
                    handleRename(item, copyName);
                }
            } else {
                showError(`Failed to rename: ${error.message || error}`);
            }
        }
    };

    // Handle clicking the file item
    const handleClick = (e) => {
        if (onClick) onClick(e);
    };

    // Handle double clicking the file item
    const handleDoubleClick = (e) => {
        if (onDoubleClick) onDoubleClick(e);
    };

    // Handle right-clicking the file item
    const handleContextMenu = (e) => {
        if (onContextMenu) onContextMenu(e);
    };

    return (
        <>
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

                        <div className="file-name truncate" title={item.name}>
                            {item.name}
                        </div>
                    </div>
                )}

                {viewMode === 'list' && (
                    <div className="file-item-list">
                        <div className="file-icon-container">
                            <FileIcon filename={item.name} isDirectory={isDirectory} />
                        </div>

                        <div className="file-details">
                            <div className="file-name truncate" title={item.name}>
                                {item.name}
                            </div>
                            <div className="file-info truncate">
                                {size} • {modified}
                            </div>
                        </div>
                    </div>
                )}

                {viewMode === 'details' && (
                    <div className="file-item-details">
                        <div className="file-column column-name">
                            <div className="file-icon-container">
                                <FileIcon filename={item.name} isDirectory={isDirectory} />
                            </div>

                            <div className="file-name truncate" title={item.name}>
                                {item.name}
                            </div>
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

            {/* Rename Modal */}
            <RenameModal
                isOpen={isRenameModalOpen}
                onClose={() => setIsRenameModalOpen(false)}
                item={item}
                onRename={handleRename}
            />
        </>
    );
};

export default FileItem;