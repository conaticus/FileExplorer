import React from 'react';
import FileIcon from './FileIcon';
import { formatFileSize, formatDate, getFileType } from '../../utils/formatters';
import './detailsPanel.css';

/**
 * Component that displays detailed information about a selected item
 * @param {Object} props - Component properties
 * @param {Object} [props.item] - The selected item to display details for
 * @param {boolean} [props.isMultipleSelection=false] - Whether multiple items are selected
 * @returns {React.ReactElement} Details panel component
 */
const DetailsPanel = ({ item, isMultipleSelection = false }) => {
    // If no item selected or multiple items are selected
    if (!item || isMultipleSelection) {
        return (
            <div className="details-panel">
                <div className="details-header">
                    <h3 className="details-title">
                        {isMultipleSelection
                            ? 'Multiple Items Selected'
                            : 'No Item Selected'}
                    </h3>
                </div>

                <div className="details-content">
                    {isMultipleSelection ? (
                        <div className="details-summary">
                            {/* In a real implementation, this would show the count of files and folders,
              total size, etc. */}
                            <p>Multiple items selected. Select a single item to view details.</p>
                        </div>
                    ) : (
                        <div className="details-empty">
                            <p>Select an item to view its details.</p>
                        </div>
                    )}
                </div>
            </div>
        );
    }

    // Determine if item is a directory or file
    const isDirectory = 'sub_file_count' in item;

    // Format size
    const size = isDirectory
        ? `${item.sub_file_count || 0} files, ${item.sub_dir_count || 0} folders`
        : formatFileSize(item.size_in_bytes);

    // Get file type
    const fileType = isDirectory ? 'Folder' : getFileType(item.name);

    // Get extension (for files)
    const extension = !isDirectory && item.name.includes('.')
        ? item.name.split('.').pop().toUpperCase()
        : '';

    // Format dates
    const created = formatDate(item.created, true);
    const modified = formatDate(item.last_modified, true);
    const accessed = formatDate(item.accessed, true);

    return (
        <div className="details-panel">
            <div className="details-header">
                <h3 className="details-title">Properties</h3>
            </div>

            <div className="details-content">
                <div className="details-preview">
                    <div className="details-icon">
                        <FileIcon
                            filename={item.name}
                            isDirectory={isDirectory}
                            size="large"
                        />
                    </div>

                    <div className="details-name truncate" title={item.name}>
                        {item.name}
                    </div>

                    <div className="details-type">
                        {fileType}{extension ? ` (${extension})` : ''}
                    </div>
                </div>

                <div className="details-section">
                    <h4 className="details-section-title">General</h4>

                    <div className="details-row">
                        <span className="details-label">Location:</span>
                        <span className="details-value truncate" title={item.path}>
                          {item.path.replace(`/${item.name}`, '')}
                        </span>
                    </div>

                    <div className="details-row">
                        <span className="details-label">Size:</span>
                        <span className="details-value">
                          {size}
                            {!isDirectory && item.size_in_bytes != null && (
                                <span className="details-value-secondary"> ({item.size_in_bytes.toLocaleString()} bytes)</span>
                            )}
                        </span>
                    </div>

                    {!isDirectory && (
                        <div className="details-row">
                            <span className="details-label">Type:</span>
                            <span className="details-value">{fileType}</span>
                        </div>
                    )}
                </div>

                <div className="details-section">
                    <h4 className="details-section-title">Dates</h4>

                    <div className="details-row">
                        <span className="details-label">Created:</span>
                        <span className="details-value">{created}</span>
                    </div>

                    <div className="details-row">
                        <span className="details-label">Modified:</span>
                        <span className="details-value">{modified}</span>
                    </div>

                    <div className="details-row">
                        <span className="details-label">Accessed:</span>
                        <span className="details-value">{accessed}</span>
                    </div>
                </div>

                <div className="details-section">
                    <h4 className="details-section-title">Permissions</h4>

                    <div className="details-row">
                        <span className="details-label">Access rights:</span>
                        <span className="details-value">{item.access_rights_as_string}</span>
                    </div>

                    <div className="details-row">
                        <span className="details-label">Octal:</span>
                        <span className="details-value">{item.access_rights_as_number}</span>
                    </div>

                    <div className="details-row">
                        <span className="details-label">Symlink:</span>
                        <span className="details-value">{item.is_symlink ? 'Yes' : 'No'}</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default DetailsPanel;

