import React, { useState, useEffect, useRef } from 'react';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import './pathBreadcrumb.css';

/**
 * PathBreadcrumb component - Displays the current file path as interactive breadcrumbs
 * and provides path editing functionality
 *
 * @param {Object} props - Component props
 * @param {Function} [props.onCopyPath] - Callback function to copy the current path
 * @param {boolean} [props.isVisible=true] - Whether the breadcrumb is visible
 * @returns {React.ReactElement} PathBreadcrumb component
 */
const PathBreadcrumb = ({ onCopyPath, isVisible = true }) => {
    const { currentPath, navigateTo } = useHistory();
    const { loadDirectory } = useFileSystem();
    const [isEditing, setIsEditing] = useState(false);
    const [editValue, setEditValue] = useState('');
    const inputRef = useRef(null);

    /**
     * Parses the current path into segments for breadcrumb navigation
     * Handles both Windows and Unix-style paths
     *
     * @returns {Array<{name: string, path: string}>} Array of path segments with display name and full path
     */
    const getPathSegments = () => {
        if (!currentPath) return [];

        const segments = [];
        let currentSegment = '';

        // Handle Windows paths (C:\path\to\folder)
        if (currentPath.includes(':\\')) {
            const parts = currentPath.split('\\');

            // Add the drive letter (e.g., C:)
            segments.push({
                name: parts[0],
                path: parts[0] + '\\',
            });

            // Add the rest of the path
            for (let i = 1; i < parts.length; i++) {
                if (!parts[i]) continue;

                currentSegment += '\\' + parts[i];
                segments.push({
                    name: parts[i],
                    path: parts[0] + currentSegment,
                });
            }
        }
        // Handle Unix paths (/path/to/folder)
        else {
            const parts = currentPath.split('/');

            // Handle root directory
            if (parts[0] === '') {
                segments.push({
                    name: '/',
                    path: '/',
                });
                parts.shift(); // Remove empty string from beginning
            }

            // Add the rest of the path
            for (let i = 0; i < parts.length; i++) {
                if (!parts[i]) continue;

                currentSegment += '/' + parts[i];
                segments.push({
                    name: parts[i],
                    path: currentSegment,
                });
            }
        }

        return segments;
    };

    const pathSegments = getPathSegments();

    /**
     * Enables path editing mode when clicking on the breadcrumb
     */
    const handleClick = () => {
        setIsEditing(true);
        setEditValue(currentPath || '');
    };

    /**
     * Handles keyboard events in the path input
     * - Enter: Navigate to the entered path
     * - Escape: Cancel editing
     *
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleKeyDown = (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            setIsEditing(false);

            if (editValue && editValue !== currentPath) {
                loadDirectory(editValue);
            }
        } else if (e.key === 'Escape') {
            e.preventDefault();
            setIsEditing(false);
            setEditValue(currentPath || '');
        }
    };

    /**
     * Navigates to the selected path segment
     *
     * @param {string} path - Path to navigate to
     */
    const handleSegmentClick = (path) => {
        loadDirectory(path);
    };

    // Focus input when editing starts
    useEffect(() => {
        if (isEditing && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [isEditing]);

    // Update edit value when path changes
    useEffect(() => {
        setEditValue(currentPath || '');
    }, [currentPath]);

    // Don't render full component if not visible
    if (!isVisible) {
        return <div className="path-breadcrumb-placeholder"></div>;
    }

    return (
        <div className="path-breadcrumb-container">
            <div className={`path-breadcrumb ${isEditing ? 'editing' : ''}`} onClick={!isEditing ? handleClick : undefined}>
                {isEditing ? (
                    <input
                        ref={inputRef}
                        className="path-input"
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        onKeyDown={handleKeyDown}
                        onBlur={() => setIsEditing(false)}
                        aria-label="Path input"
                    />
                ) : (
                    <div className="breadcrumb-segments">
                        {pathSegments.map((segment, index) => (
                            <React.Fragment key={segment.path}>
                                {index > 0 && <span className="segment-divider">/</span>}
                                <button
                                    className={`segment-button ${index === pathSegments.length - 1 ? 'current' : ''}`}
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        handleSegmentClick(segment.path);
                                    }}
                                >
                                    {segment.name}
                                </button>
                            </React.Fragment>
                        ))}
                    </div>
                )}
            </div>

            {onCopyPath && currentPath && (
                <button
                    className="copy-path-btn"
                    onClick={onCopyPath}
                    title="Copy current path"
                    aria-label="Copy current path"
                >
                    <span className="icon icon-copy"></span>
                </button>
            )}
        </div>
    );
};

export default PathBreadcrumb;