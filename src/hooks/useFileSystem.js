import { useContext } from 'react';
import { FileSystemContext } from '../providers/FileSystemProvider';

/**
 * Hook for accessing file system operations.
 * @returns {Object} File system state and functions.
 */
const useFileSystem = () => {
    const context = useContext(FileSystemContext);

    if (!context) {
        throw new Error('useFileSystem must be used within a FileSystemProvider');
    }

    return context;
};

export default useFileSystem;