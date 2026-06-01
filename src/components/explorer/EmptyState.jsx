import React from 'react';
import './emptyState.css';
import zhCN from '../../locales/zh-CN.js';

/**
 * Component for displaying various empty state messages
 * @param {Object} props - Component properties
 * @param {string} [props.type='empty-folder'] - Type of empty state to display
 * @param {string} [props.searchTerm=null] - Search term to display in 'no-results' state
 * @param {string} [props.title=null] - Custom title to override default
 * @param {string} [props.message=null] - Custom message to override default
 * @returns {React.ReactElement} Empty state component
 */
const EmptyState = ({ type = 'empty-folder', searchTerm = null, title = null, message = null }) => {
    // Different empty states
    const emptyStates = {
        'empty-folder': {
            icon: 'folder-empty',
            title: zhCN.emptyState.emptyFolder,
            message: zhCN.emptyState.emptyFolderHint,
        },
        'no-results': {
            icon: 'search-empty',
            title: searchTerm ? zhCN.emptyState.noResultsFor.replace('{term}', searchTerm) : zhCN.emptyState.noResults,
            message: zhCN.emptyState.noResultsHint,
        },
        'no-favorites': {
            icon: 'star-empty',
            title: zhCN.emptyState.noFavorites,
            message: zhCN.emptyState.noFavoritesHint,
        },
        'no-templates': {
            icon: 'template-empty',
            title: zhCN.emptyState.noTemplates,
            message: zhCN.emptyState.noTemplatesHint,
        },
        'error': {
            icon: 'error',
            title: zhCN.emptyState.error,
            message: zhCN.emptyState.errorHint,
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