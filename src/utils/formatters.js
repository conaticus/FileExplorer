/**
 * Format a file size in bytes to a human-readable string.
 * @param {number} bytes - The file size in bytes.
 * @param {number} decimals - The number of decimal places to show.
 * @returns {string} The formatted file size.
 */
export const formatFileSize = (bytes, decimals = 1) => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

/**
 * Format a date string to a human-readable format.
 * @param {string} dateString - The date string to format.
 * @param {boolean} includeTime - Whether to include the time.
 * @returns {string} The formatted date.
 */
export const formatDate = (dateString, includeTime = false) => {
    if (!dateString) return 'Unknown';

    try {
        const date = new Date(dateString);

        // Check if the date is valid
        if (isNaN(date.getTime())) {
            return 'Invalid date';
        }

        const options = {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
        };

        if (includeTime) {
            options.hour = '2-digit';
            options.minute = '2-digit';
            options.hour12 = true;
        }

        return new Intl.DateTimeFormat('en-US', options).format(date);
    } catch (error) {
        console.error('Error formatting date:', error);
        return 'Invalid date';
    }
};

/**
 * Get a human-readable file type based on the file extension.
 * @param {string} filename - The filename to extract the extension from.
 * @returns {string} The human-readable file type.
 */
export const getFileType = (filename) => {
    if (!filename || !filename.includes('.')) {
        return 'File';
    }

    const extension = filename.split('.').pop().toLowerCase();

    const fileTypes = {
        // Documents
        'pdf': 'PDF Document',
        'doc': 'Word Document',
        'docx': 'Word Document',
        'xls': 'Excel Spreadsheet',
        'xlsx': 'Excel Spreadsheet',
        'ppt': 'PowerPoint Presentation',
        'pptx': 'PowerPoint Presentation',
        'txt': 'Text Document',
        'rtf': 'Rich Text Document',
        'odt': 'OpenDocument Text',
        'ods': 'OpenDocument Spreadsheet',
        'odp': 'OpenDocument Presentation',
        'csv': 'CSV File',
        'md': 'Markdown Document',

        // Images
        'jpg': 'JPEG Image',
        'jpeg': 'JPEG Image',
        'png': 'PNG Image',
        'gif': 'GIF Image',
        'bmp': 'Bitmap Image',
        'svg': 'SVG Image',
        'webp': 'WebP Image',
        'tiff': 'TIFF Image',
        'ico': 'Icon File',

        // Audio
        'mp3': 'MP3 Audio',
        'wav': 'WAV Audio',
        'ogg': 'OGG Audio',
        'flac': 'FLAC Audio',
        'm4a': 'M4A Audio',
        'aac': 'AAC Audio',

        // Video
        'mp4': 'MP4 Video',
        'avi': 'AVI Video',
        'mov': 'QuickTime Video',
        'wmv': 'Windows Media Video',
        'mkv': 'Matroska Video',
        'webm': 'WebM Video',

        // Archives
        'zip': 'ZIP Archive',
        'rar': 'RAR Archive',
        '7z': '7-Zip Archive',
        'tar': 'TAR Archive',
        'gz': 'GZip Archive',
        'bz2': 'BZip2 Archive',

        // Programming
        'html': 'HTML File',
        'css': 'CSS File',
        'js': 'JavaScript File',
        'jsx': 'React JSX File',
        'ts': 'TypeScript File',
        'tsx': 'React TSX File',
        'json': 'JSON File',
        'xml': 'XML File',
        'yaml': 'YAML File',
        'toml': 'TOML File',
        'py': 'Python File',
        'java': 'Java File',
        'c': 'C File',
        'cpp': 'C++ File',
        'h': 'C Header File',
        'cs': 'C# File',
        'php': 'PHP File',
        'rb': 'Ruby File',
        'go': 'Go File',
        'rs': 'Rust File',
        'swift': 'Swift File',
        'kt': 'Kotlin File',
        'sql': 'SQL File',

        // Executables
        'exe': 'Windows Executable',
        'msi': 'Windows Installer',
        'app': 'macOS Application',
        'dmg': 'macOS Disk Image',
        'deb': 'Debian Package',
        'rpm': 'Red Hat Package',
        'apk': 'Android Package',

        // Other
        'iso': 'Disk Image',
        'torrent': 'Torrent File',
    };

    return fileTypes[extension] || `${extension.toUpperCase()} File`;
};

/**
 * Calculate the time elapsed since a specific date.
 * @param {string} dateString - The date string to calculate from.
 * @returns {string} The elapsed time as a human-readable string.
 */
export const calculateTimeElapsed = (dateString) => {
    if (!dateString) return 'Unknown';

    try {
        const date = new Date(dateString);
        const now = new Date();

        // Check if the date is valid
        if (isNaN(date.getTime())) {
            return 'Invalid date';
        }

        const seconds = Math.floor((now - date) / 1000);

        if (seconds < 60) {
            return 'Just now';
        }

        const minutes = Math.floor(seconds / 60);
        if (minutes < 60) {
            return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
        }

        const hours = Math.floor(minutes / 60);
        if (hours < 24) {
            return `${hours} hour${hours === 1 ? '' : 's'} ago`;
        }

        const days = Math.floor(hours / 24);
        if (days < 30) {
            return `${days} day${days === 1 ? '' : 's'} ago`;
        }

        const months = Math.floor(days / 30);
        if (months < 12) {
            return `${months} month${months === 1 ? '' : 's'} ago`;
        }

        const years = Math.floor(months / 12);
        return `${years} year${years === 1 ? '' : 's'} ago`;
    } catch (error) {
        console.error('Error calculating time elapsed:', error);
        return 'Unknown';
    }
};