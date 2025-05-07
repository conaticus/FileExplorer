/**
 * Get the icon type for a file based on its extension.
 * @param {string} filename - The filename to determine the icon for.
 * @returns {string} The icon type identifier.
 */
export const getFileIconType = (filename) => {
    if (!filename || !filename.includes('.')) {
        return 'default';
    }

    const extension = filename.split('.').pop().toLowerCase();

    // Image files
    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'tiff', 'ico'];
    if (imageExtensions.includes(extension)) {
        return 'image';
    }

    // Audio files
    const audioExtensions = ['mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac'];
    if (audioExtensions.includes(extension)) {
        return 'audio';
    }

    // Video files
    const videoExtensions = ['mp4', 'avi', 'mov', 'wmv', 'mkv', 'webm', 'flv', '3gp'];
    if (videoExtensions.includes(extension)) {
        return 'video';
    }

    // Code files
    const codeExtensions = [
        'html', 'css', 'js', 'jsx', 'ts', 'tsx', 'json', 'xml', 'yaml', 'toml',
        'py', 'java', 'c', 'cpp', 'h', 'cs', 'php', 'rb', 'go', 'rs', 'swift',
        'kt', 'sql', 'sh', 'bat', 'ps1'
    ];
    if (codeExtensions.includes(extension)) {
        return 'code';
    }

    // Archive files
    const archiveExtensions = ['zip', 'rar', '7z', 'tar', 'gz', 'bz2'];
    if (archiveExtensions.includes(extension)) {
        return 'archive';
    }

    // PDF files
    if (extension === 'pdf') {
        return 'pdf';
    }

    // Text and document files
    const textExtensions = [
        'txt', 'md', 'rtf', 'doc', 'docx', 'odt', 'xls', 'xlsx', 'ods',
        'ppt', 'pptx', 'odp', 'csv'
    ];
    if (textExtensions.includes(extension)) {
        return 'text';
    }

    // Default icon for other file types
    return 'default';
};

/**
 * Get the folder icon type based on the folder name or path.
 * @param {string} folderName - The folder name or path.
 * @returns {string} The folder icon type identifier.
 */
export const getFolderIconType = (folderName) => {
    if (!folderName) {
        return 'folder';
    }

    // Get the last part of the path
    const name = folderName.split('/').pop().toLowerCase();

    // Special folder types
    const specialFolders = {
        'documents': 'documents',
        'docs': 'documents',
        'downloads': 'downloads',
        'pictures': 'pictures',
        'images': 'pictures',
        'photos': 'pictures',
        'music': 'music',
        'audio': 'music',
        'videos': 'videos',
        'movies': 'videos',
        'desktop': 'desktop',
        'projects': 'projects',
        'src': 'source',
        'source': 'source',
        'node_modules': 'node',
        'public': 'public',
        'assets': 'assets',
        'templates': 'templates',
        'config': 'config',
        'settings': 'config',
    };

    return specialFolders[name] || 'folder';
};