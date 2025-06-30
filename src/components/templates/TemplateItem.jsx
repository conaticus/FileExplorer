import React, { useState } from 'react';
import Icon from '../common/Icon';
import IconButton from '../common/IconButton';
import Modal from '../common/Modal';
import Button from '../common/Button';
import './templates.css';

/**
 * TemplateItem component - Displays a single template with actions
 *
 * @param {Object} props - Component props
 * @param {Object} props.template - Template object to display
 * @param {string} props.template.name - Template name
 * @param {string} props.template.path - Template path
 * @param {string} props.template.type - Template type (file or folder)
 * @param {number} [props.template.size] - Template size in bytes
 * @param {string} [props.template.createdAt] - Template creation date
 * @param {Function} props.onUse - Callback when the template is used
 * @param {Function} props.onRemove - Callback when the template is removed
 * @returns {React.ReactElement} TemplateItem component
 */
const TemplateItem = ({ template, onUse, onRemove }) => {
    const [isConfirmDeleteOpen, setIsConfirmDeleteOpen] = useState(false);

    /**
     * Determines the appropriate icon based on template type and file extension
     * @returns {string} Icon name to use
     */
    const getTemplateIcon = () => {
        if (template.type === 'folder') {
            return 'folder';
        }

        // For files, extract extension
        const extension = template.name.split('.').pop()?.toLowerCase();

        switch (extension) {
            case 'doc':
            case 'docx':
            case 'txt':
            case 'md':
                return 'document';
            case 'xls':
            case 'xlsx':
            case 'csv':
                return 'spreadsheet';
            case 'ppt':
            case 'pptx':
                return 'presentation';
            case 'pdf':
                return 'pdf';
            case 'jpg':
            case 'jpeg':
            case 'png':
            case 'gif':
            case 'svg':
                return 'image';
            case 'mp3':
            case 'wav':
            case 'ogg':
                return 'audio';
            case 'mp4':
            case 'avi':
            case 'mov':
                return 'video';
            case 'zip':
            case 'rar':
            case '7z':
                return 'archive';
            case 'html':
            case 'css':
            case 'js':
            case 'jsx':
            case 'ts':
            case 'tsx':
            case 'json':
                return 'code';
            default:
                return 'file';
        }
    };

    /**
     * Formats file size into human-readable format
     * @param {number} bytes - Size in bytes
     * @returns {string} Formatted size string
     */
    const formatSize = (bytes) => {
        if (!bytes && bytes !== 0) return 'Unknown size';

        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let size = bytes;
        let unitIndex = 0;

        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }

        return `${size.toFixed(1)} ${units[unitIndex]}`;
    };

    /**
     * Formats date into human-readable format
     * @param {string} dateStr - Date string
     * @returns {string} Formatted date string
     */
    const formatDate = (dateStr) => {
        if (!dateStr) return 'Unknown date';

        try {
            const date = new Date(dateStr);
            return date.toLocaleDateString(undefined, {
                year: 'numeric',
                month: 'short',
                day: 'numeric'
            });
        } catch (err) {
            return 'Invalid date';
        }
    };

    /**
     * Handles delete button click
     * Opens confirmation modal
     */
    const handleDeleteClick = () => {
        setIsConfirmDeleteOpen(true);
    };

    /**
     * Confirms template deletion
     * Closes modal and calls onRemove callback
     */
    const confirmDelete = () => {
        setIsConfirmDeleteOpen(false);
        onRemove();
    };

    return (
        <>
            <div className="template-item">
                <div className="template-icon">
                    <Icon name={getTemplateIcon()} size="large" />
                </div>

                <div className="template-details">
                    <h3 className="template-name">{template.name}</h3>

                    <div className="template-meta">
                        <span className="template-type">{template.type === 'folder' ? 'Folder' : 'File'}</span>
                        {template.size && (
                            <span className="template-size">{formatSize(template.size)}</span>
                        )}
                        {template.createdAt && (
                            <span className="template-date">{formatDate(template.createdAt)}</span>
                        )}
                    </div>
                </div>

                <div className="template-actions">
                    <IconButton
                        icon="trash"
                        tooltip="Delete template"
                        onClick={handleDeleteClick}
                        aria-label="Delete template"
                    />
                    <Button
                        variant="primary"
                        size="sm"
                        onClick={onUse}
                    >
                        Use
                    </Button>
                </div>
            </div>

            {/* Confirm delete modal */}
            <Modal
                isOpen={isConfirmDeleteOpen}
                onClose={() => setIsConfirmDeleteOpen(false)}
                title="Confirm Delete"
                size="sm"
                footer={
                    <>
                        <Button
                            variant="ghost"
                            onClick={() => setIsConfirmDeleteOpen(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            variant="danger"
                            onClick={confirmDelete}
                        >
                            Delete
                        </Button>
                    </>
                }
            >
                <p>
                    Are you sure you want to delete the template "{template.name}"?
                    This action cannot be undone.
                </p>
            </Modal>
        </>
    );
};

export default TemplateItem;