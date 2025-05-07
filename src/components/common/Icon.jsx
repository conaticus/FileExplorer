import React from 'react';
import './common.css';

/**
 * Icon mapping to SVG paths or class names
 * This is a simplified version - in a real implementation,
 * this would either use an icon library or have SVG definitions for all icons
 */
const ICON_MAP = {
    // Navigation
    'home': 'icon-home',
    'folder': 'icon-folder',
    'file': 'icon-file',
    'search': 'icon-search',
    'arrow-left': 'icon-arrow-left',
    'arrow-right': 'icon-arrow-right',
    'arrow-up': 'icon-arrow-up',
    'arrow-down': 'icon-arrow-down',
    'chevron-left': 'icon-chevron-left',
    'chevron-right': 'icon-chevron-right',
    'chevron-up': 'icon-chevron-up',
    'chevron-down': 'icon-chevron-down',

    // Actions
    'plus': 'icon-plus',
    'minus': 'icon-minus',
    'x': 'icon-x',
    'check': 'icon-check',
    'edit': 'icon-edit',
    'trash': 'icon-trash',
    'copy': 'icon-copy',
    'cut': 'icon-cut',
    'paste': 'icon-paste',
    'download': 'icon-download',
    'upload': 'icon-upload',
    'refresh': 'icon-refresh',
    'settings': 'icon-settings',
    'more-vertical': 'icon-more-vertical',
    'more-horizontal': 'icon-more-horizontal',

    // File types
    'image': 'icon-image',
    'video': 'icon-video',
    'audio': 'icon-audio',
    'document': 'icon-document',
    'spreadsheet': 'icon-spreadsheet',
    'presentation': 'icon-presentation',
    'code': 'icon-code',
    'archive': 'icon-archive',
    'pdf': 'icon-pdf',

    // UI elements
    'eye': 'icon-eye',
    'eye-off': 'icon-eye-off',
    'info': 'icon-info',
    'alert-circle': 'icon-alert-circle',
    'alert-triangle': 'icon-alert-triangle',
    'bell': 'icon-bell',
    'menu': 'icon-menu',
    'grid': 'icon-grid',
    'list': 'icon-list',
    'terminal': 'icon-terminal',
    'star': 'icon-star',
    'heart': 'icon-heart',
    'bookmark': 'icon-bookmark',
    'tag': 'icon-tag',
    'lock': 'icon-lock',
    'unlock': 'icon-unlock',
    'filter': 'icon-filter',
    'sort': 'icon-sort',
    'external-link': 'icon-external-link',
    'link': 'icon-link',
    'sun': 'icon-sun',
    'moon': 'icon-moon',

    // Default for unknown icons
    'default': 'icon-help-circle'
};

/**
 * Icon component
 * @param {Object} props - Component props
 * @param {string} props.name - Icon name
 * @param {string} [props.size='medium'] - Icon size (small, medium, large)
 * @param {string} [props.color] - Custom color (CSS color value)
 * @param {string} [props.className] - Additional CSS class names
 * @returns {React.ReactElement} Icon component
 */
const Icon = ({ name, size = 'medium', color, className = '', ...rest }) => {
    // Get icon class from map or use default
    const iconClass = ICON_MAP[name] || ICON_MAP.default;

    // Build class name based on props
    const classes = [
        'icon',
        iconClass,
        `icon-size-${size}`,
        className
    ].filter(Boolean).join(' ');

    // Create style object if color is provided
    const style = color ? { color } : undefined;

    return <span className={classes} style={style} aria-hidden="true" {...rest} />;
};

export default Icon;