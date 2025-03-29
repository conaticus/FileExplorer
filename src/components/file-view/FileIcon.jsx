import React from 'react';

const FileIcon = ({ fileType, extension }) => {
    // Funktion zum Bestimmen des Icons basierend auf Dateityp und/oder Erweiterung
    const getIconName = () => {
        // Wenn es ein Verzeichnis ist
        if (fileType === 'directory') {
            return 'folder';
        }

        // Basierend auf der Dateierweiterung
        if (extension) {
            const ext = extension.toLowerCase();

            // Dokumente
            if (['doc', 'docx', 'odt', 'rtf'].includes(ext)) return 'file-word';
            if (['xls', 'xlsx', 'ods', 'csv'].includes(ext)) return 'file-excel';
            if (['ppt', 'pptx', 'odp'].includes(ext)) return 'file-powerpoint';
            if (['pdf'].includes(ext)) return 'file-pdf';
            if (['txt', 'md'].includes(ext)) return 'file-text';
            if (['json'].includes(ext)) return 'file-code';

            // Bilder
            if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp'].includes(ext)) {
                return 'file-image';
            }

            // Audio
            if (['mp3', 'wav', 'ogg', 'flac', 'aac'].includes(ext)) {
                return 'file-audio';
            }

            // Video
            if (['mp4', 'avi', 'mov', 'wmv', 'mkv', 'webm'].includes(ext)) {
                return 'file-video';
            }

            // Archiv
            if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) {
                return 'file-archive';
            }

            // Code
            if (['html', 'htm', 'css', 'js', 'jsx', 'ts', 'tsx', 'xml'].includes(ext)) {
                return 'file-code';
            }

            // Programmierung
            if (['py', 'java', 'c', 'cpp', 'cs', 'rb', 'php', 'go', 'swift', 'kt'].includes(ext)) {
                return 'file-code';
            }
        }

        // Standard-Datei-Icon
        return 'file';
    };

    // Bestimme die Icon-ID
    const iconName = getIconName();

    // Vordefinierte Icon-Pfade für häufig verwendete Icons
    const iconPaths = {
        folder: 'M10 3H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h16a1 1 0 0 0 1-1V7a1 1 0 0 0-1-1h-8l-2-2z',
        file: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6',
        'file-text': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M16 13H8 M16 17H8 M10 9H8',
        'file-image': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M8 10a1 1 0 1 0 2 0a1 1 0 1 0 -2 0 M20 15l-2-2-4 5-3-3-3 3',
        'file-audio': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 15a1 1 0 1 0 2 0a1 1 0 1 0 -2 0 M13 11a1 1 0 1 0 2 0a1 1 0 1 0 -2 0 M9 11V15 M13 11v4',
        'file-video': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M10 11l5 3l-5 3v-6z',
        'file-archive': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M12 10v8 M10 12h4 M10 16h4',
        'file-code': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M10 12l-2 2l2 2 M14 12l2 2l-2 2',
        'file-word': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 13l3 5l3-5',
        'file-excel': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 13l6 6 M15 13l-6 6',
        'file-powerpoint': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 15a3 3 0 1 0 0-6a3 3 0 0 0 0 6z M9 12h8',
        'file-pdf': 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z M14 2v6h6 M9 13h6 M9 17h6 M9 9h1',
    };

    const iconPath = iconPaths[iconName] || iconPaths.file;

    return (
        <div className="file-icon" data-type={fileType} data-extension={extension || ''}>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                width="24"
                height="24"
            >
                <path d={iconPath} />
            </svg>
        </div>
    );
};

export default FileIcon;