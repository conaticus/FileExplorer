import React from 'react';
import './emptyState.css';

const EmptyState = ({ type = 'empty-folder', searchTerm = null }) => {
    // Different empty states
    const emptyStates = {
        'empty-folder': {
            icon: 'folder-empty',
            title: 'This folder is empty',
            message: 'Drag and drop files here or use the Create button to add content',
        },
        'no-results': {
            icon: 'search-empty',
            title: `No results found${searchTerm ? ` for "${searchTerm}"` : ''}`,
            message: 'Try different keywords or check your spelling',
        },
        'no-favorites': {
            icon: 'star-empty',
            title: 'No favorites yet',
            message: 'Right-click on folders and files to add them to favorites',
        },
        'error': {
            icon: 'error',
            title: 'Something went wrong',
            message: 'Please try again or check your connection',
        },
    };

    const { icon, title, message } = emptyStates[type] || emptyStates['empty-folder'];

    return (
        <div className="empty-state">
            <div className={`empty-state-icon icon-${icon}`}></div>
            <h3 className="empty-state-title">{title}</h3>
            <p className="empty-state-message">{message}</p>
        </div>
    );
};

export default EmptyState;