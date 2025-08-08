import React, { useState, useEffect } from 'react';
import FileIcon from './FileIcon';
import RenameModal from '../common/RenameModal';
import { formatFileSize, formatDate, getFileType } from '../../utils/formatters';
import { replaceFileName } from '../../utils/pathUtils.js';
import { invoke } from '@tauri-apps/api/core';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { useContextMenu } from '../../providers/ContextMenuProvider';
import { showError, showConfirm } from '../../utils/NotificationSystem';
import './fileItem.css';

/**
 * Component that displays a single file or directory item
 * @param {Object} props - Component properties
 * @param {Object} props.item - The file or directory object to display
 * @param {string} [props.viewMode='grid'] - The view mode: 'grid', 'list', or 'details'
 * @param {boolean} [props.isSelected=false] - Whether the item is currently selected
 * @param {Function} props.onClick - Click handler function
 * @param {Function} props.onDoubleClick - Double-click handler function
 * @param {Function} props.onContextMenu - Context menu handler function
 * @returns {React.ReactElement} File/directory item component
 */
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
    const { clipboard } = useContextMenu();

    const isDirectory = item.isDirectory || 'sub_file_count' in item;
    const fileType = isDirectory ? 'Folder' : getFileType(item.name);
    const size = isDirectory
        ? `${item.sub_file_count || 0} items`
        : formatFileSize(item.size_in_bytes);

    // Check if this item is cut (in clipboard with cut operation)
    const isCut = clipboard.operation === 'cut' && 
                  clipboard.items.some(clipItem => clipItem.path === item.path);

    // Format modified date
    const modified = formatDate(item.last_modified);

    /**
     * Sets up event listener for rename modal events
     */
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

    /**
     * Handles the rename operation with robust path handling
     * @param {Object} item - The item to rename
     * @param {string} newName - The new name for the item
     */
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

    /**
     * Handles click events on the file item
     * @param {React.MouseEvent} e - The click event
     */
    const handleClick = (e) => {
        if (onClick) onClick(e);
    };

    /**
     * Handles double-click events on the file item
     * @param {React.MouseEvent} e - The double-click event
     */
    const handleDoubleClick = (e) => {
        if (onDoubleClick) onDoubleClick(e);
    };

    /**
     * Handles right-click context menu events on the file item
     * @param {React.MouseEvent} e - The context menu event
     */
    const handleContextMenu = (e) => {
        if (onContextMenu) onContextMenu(e);
    };

    return (
        <>
            <div
                className={`file-item view-mode-${viewMode.toLowerCase()} ${isSelected ? 'selected' : ''} ${isDirectory ? 'directory' : 'file'} ${isCut ? 'cut' : ''}`}
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