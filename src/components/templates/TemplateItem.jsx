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

    // Safety check for template object
    if (!template || typeof template !== 'object') {
        console.error('TemplateItem: Invalid template prop:', template);
        return null;
    }

    // Provide defaults for required properties
    const safeName = template.name || 'Unknown Template';
    const safePath = template.path || '';
    const safeType = template.type || 'file';

    /**
     * Determines the appropriate icon based on template type and file extension
     * Uses only icons that actually exist in the CSS files, with fallbacks
     * @returns {string} Icon name to use
     */
    const getTemplateIcon = () => {
        if (safeType === 'folder') {
            return 'folder'; // maps to icon-folder (exists in contextMenu.css)
        }

        // For files, try to determine the type but fall back to basic file icon
        if (!safeName || typeof safeName !== 'string') {
            return 'file';
        }

        const extension = safeName.includes('.')
            ? safeName.split('.').pop()?.toLowerCase()
            : '';

        if (!extension) {
            return 'file';
        }

        // Only use specific icons if we're confident they exist
        // Otherwise fall back to the basic file icon
        switch (extension) {
            case 'pdf':
                return 'pdf'; // Only if icon-pdf exists in CSS
            case 'jpg':
            case 'jpeg':
            case 'png':
            case 'gif':
            case 'svg':
            case 'webp':
                return 'image'; // Only if icon-image exists in CSS
            case 'mp4':
            case 'avi':
            case 'mov':
            case 'mkv':
                return 'video'; // Only if icon-video exists in CSS
            case 'mp3':
            case 'wav':
            case 'ogg':
            case 'flac':
                return 'audio'; // Only if icon-audio exists in CSS
            case 'zip':
            case 'rar':
            case '7z':
            case 'tar':
            case 'gz':
                return 'archive'; // Only if icon-archive exists in CSS
            case 'html':
            case 'css':
            case 'js':
            case 'jsx':
            case 'ts':
            case 'tsx':
            case 'json':
            case 'py':
            case 'java':
            case 'c':
            case 'cpp':
                return 'code'; // Only if icon-code exists in CSS
            default:
                return 'file'; // Safe fallback
        }
    };

    /**
     * Formats file size into human-readable format
     * @param {number} bytes - Size in bytes
     * @returns {string} Formatted size string
     */
    const formatSize = (bytes) => {
        if (!bytes && bytes !== 0) return 'Unknown size';
        if (typeof bytes !== 'number') return 'Unknown size';

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
            if (isNaN(date.getTime())) {
                return 'Invalid date';
            }
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
        if (typeof onRemove === 'function') {
            onRemove();
        }
    };

    /**
     * Handles use button click
     */
    const handleUseClick = () => {
        if (typeof onUse === 'function') {
            onUse();
        }
    };

    return (
        <>
            <div className="template-item">
                <div className="template-icon">
                    <Icon name={getTemplateIcon()} size="large" />
                </div>

                <div className="template-details">
                    <h3 className="template-name" title={safeName}>{safeName}</h3>

                    <div className="template-meta">
                        <span className="template-type">
                            {safeType === 'folder' ? 'Folder' : 'File'}
                        </span>
                        {template.size && (
                            <span className="template-size">{formatSize(template.size)}</span>
                        )}
                        {template.createdAt && (
                            <span className="template-date">{formatDate(template.createdAt)}</span>
                        )}
                    </div>
                </div>

                <div className="template-actions">
                    <button
                        className="template-delete-btn"
                        onClick={handleDeleteClick}
                        title="Delete template"
                        aria-label="Delete template"
                    >
                        <span className="icon icon-delete"></span>
                    </button>
                    <Button
                        variant="primary"
                        size="sm"
                        onClick={handleUseClick}
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
                    Are you sure you want to delete the template "{safeName}"?
                    This action cannot be undone.
                </p>
            </Modal>
        </>
    );
};

export default TemplateItem;