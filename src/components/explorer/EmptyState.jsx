import React from 'react';
import './emptyState.css';

const EmptyState = ({ type = 'empty-folder', searchTerm = null, title = null, message = null }) => {
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
        'no-templates': {
            icon: 'template-empty',
            title: 'No templates',
            message: 'You haven\'t saved any templates yet. Templates help you create files and folders with predefined structures.',
        },
        'error': {
            icon: 'error',
            title: 'Something went wrong',
            message: 'Please try again or check your connection',
        },
    };

    const emptyState = emptyStates[type] || emptyStates['empty-folder'];

    // Allow overriding title and message via props
    const finalTitle = title || emptyState.title;
    const finalMessage = message || emptyState.message;

    return (
        <div className="empty-state">
            <div className={`empty-state-icon icon-${emptyState.icon}`}></div>
            <h3 className="empty-state-title">{finalTitle}</h3>
            <p className="empty-state-message">{finalMessage}</p>
        </div>
    );
};

export default EmptyState;