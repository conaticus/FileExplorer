import React, { useState, useEffect, useRef } from 'react';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import './pathBreadcrumb.css';

const PathBreadcrumb = () => {
    const { currentPath, navigateTo } = useHistory();
    const { loadDirectory } = useFileSystem();
    const [isEditing, setIsEditing] = useState(false);
    const [editValue, setEditValue] = useState('');
    const inputRef = useRef(null);

    // Parse path into segments for breadcrumb
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

    // Enable path editing mode
    const handleClick = () => {
        setIsEditing(true);
        setEditValue(currentPath || '');
    };

    // Navigate to path when Enter is pressed
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

    // Navigate to segment
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

    return (
        <div className="path-breadcrumb" onClick={!isEditing ? handleClick : undefined}>
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
    );
};

export default PathBreadcrumb;