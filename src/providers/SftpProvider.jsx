import React, { createContext, useContext, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { showError, showSuccess } from '../utils/NotificationSystem';

const SftpContext = createContext({
    sftpConnections: [],
    currentSftpConnection: null,
    currentSftpPath: null,
    loadSftpDirectory: () => {},
    createSftpFile: () => {},
    createSftpDirectory: () => {},
    deleteSftpItem: () => {},
    renameSftpItem: () => {},
    copySftpItem: () => {},
    moveSftpItem: () => {},
    openSftpFile: () => {},
    downloadAndOpenSftpFile: () => {},
    isSftpPath: () => false,
    parseSftpPath: () => null,
    navigateToSftpConnection: () => {},
    disconnectSftp: () => {},
    createSftpUrl: () => null,
    createSftpPath: () => null
});

export const useSftp = () => useContext(SftpContext);

export default function SftpProvider({ children }) {
    const [sftpConnections, setSftpConnections] = useState([]);
    const [currentSftpConnection, setCurrentSftpConnection] = useState(null);
    const [currentSftpPath, setCurrentSftpPath] = useState(null);
    const connectionCache = useRef(new Map());

    // Load SFTP connections from localStorage
    const loadSftpConnections = useCallback(() => {
        try {
            const saved = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
            setSftpConnections(saved);
            return saved;
        } catch (err) {
            console.error('Failed to load SFTP connections:', err);
            return [];
        }
    }, []);

    // Initialize SFTP connections on mount
    React.useEffect(() => {
        loadSftpConnections();
        const handler = () => loadSftpConnections();
        window.addEventListener('sftp-connections-updated', handler);
        return () => window.removeEventListener('sftp-connections-updated', handler);
    }, [loadSftpConnections]);

    // Check if a path is an SFTP path
    const isSftpPath = useCallback((path) => {
        return typeof path === 'string' && (path.startsWith('sftp://') || path.startsWith('sftp:'));
    }, []);

    // Parse SFTP path to extract connection details and remote path
    const parseSftpPath = useCallback((path) => {
        console.log('parseSftpPath called with:', path);
        
        if (!isSftpPath(path)) {
            console.log('Path is not SFTP path');
            return null;
        }

        try {
            // Get fresh connections from localStorage if our state is empty
            let connections = sftpConnections;
            if (connections.length === 0) {
                try {
                    connections = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
                    console.log('Loaded fresh connections from localStorage:', connections);
                } catch (err) {
                    connections = [];
                }
            }
            console.log('Available SFTP connections:', connections);

            // Handle different SFTP path formats:
            // sftp://user@host:port/path/to/file
            // sftp:connectionName:/path/to/file
            
            if (path.includes('@')) {
                // Full URL format: sftp://user@host:port/remote/path
                const url = new URL(path);
                const connection = connections.find(conn => 
                    conn.username === url.username && 
                    conn.host === url.hostname && 
                    conn.port === (url.port || '22')
                );
                console.log('Found connection for URL format:', connection);
                return {
                    connection,
                    remotePath: url.pathname || '/',
                    connectionName: connection?.name
                };
            } else {
                // Connection name format: sftp:connectionName:/remote/path
                const withoutPrefix = path.replace('sftp:', '');
                const colonIndex = withoutPrefix.indexOf(':');
                
                if (colonIndex === -1) {
                    // No colon found, treat entire string as connection name with root path
                    const connectionName = withoutPrefix;
                    const remotePath = '.';
                    const connection = connections.find(conn => conn.name === connectionName);
                    console.log('Found connection for simple format:', connection, 'connectionName:', connectionName, 'available connections:', connections.map(c => c.name));
                    return {
                        connection,
                        remotePath,
                        connectionName
                    };
                } else {
                    // Colon found, split connection name and path
                    const connectionName = withoutPrefix.substring(0, colonIndex);
                    const remotePath = withoutPrefix.substring(colonIndex + 1) || '.';
                    const connection = connections.find(conn => conn.name === connectionName);
                    console.log('Found connection for colon format:', connection, 'connectionName:', connectionName, 'remotePath:', remotePath);
                    return {
                        connection,
                        remotePath,
                        connectionName
                    };
                }
            }
        } catch (err) {
            console.error('Failed to parse SFTP path:', path, err);
            return null;
        }
    }, [sftpConnections, isSftpPath]);

    // Create SFTP URL for a connection and path
    const createSftpUrl = useCallback((connection, remotePath = '/') => {
        if (!connection) return null;
        return `sftp:${connection.name}:${remotePath}`;
    }, []);

    // Create SFTP path for a connection and path (internal format)
    const createSftpPath = useCallback((connection, remotePath = '.') => {
        if (!connection) return null;
        return `sftp:${connection.name}:${remotePath}`;
    }, []);

    // Navigate to an SFTP connection
    const navigateToSftpConnection = useCallback(async (connection, remotePath = '.') => {
        console.log('navigateToSftpConnection called with:', connection, remotePath);
        
        if (!connection) {
            console.error('No connection provided to navigateToSftpConnection');
            return null;
        }
        
        try {
            setCurrentSftpConnection(connection);
            setCurrentSftpPath(remotePath);
            
            // Ensure remotePath is valid for SFTP command
            const sftpPath = remotePath || '.';
            console.log('Using SFTP path:', sftpPath);
            
            // Load directory using existing SFTP command
            const result = await invoke('load_dir', {
                host: connection.host,
                port: parseInt(connection.port, 10),
                username: connection.username,
                password: connection.password,
                directory: sftpPath
            });
            
            console.log('SFTP load_dir result:', result);
            const dirData = JSON.parse(result);
            console.log('Parsed SFTP directory data:', dirData);
            
            // Transform data to match FileSystemProvider format
            const transformedData = {
                directory: createSftpUrl(connection, remotePath),
                directories: (dirData.directories || []).map(dirPath => {
                    const name = dirPath.split('/').pop() || dirPath.replace(/^\.\//, '');
                    const fullPath = dirPath.startsWith('./') ? dirPath.substring(2) : dirPath;
                    return {
                        name: name || 'Directory',
                        path: createSftpUrl(connection, fullPath),
                        isDirectory: true,
                        size: 0,
                        modified: null
                    };
                }),
                files: (dirData.files || []).map(filePath => {
                    const name = filePath.split('/').pop() || filePath.replace(/^\.\//, '');
                    const fullPath = filePath.startsWith('./') ? filePath.substring(2) : filePath;
                    return {
                        name: name || 'File',
                        path: createSftpUrl(connection, fullPath),
                        isDirectory: false,
                        size: 0,
                        modified: null
                    };
                })
            };
            
            console.log('Transformed SFTP data:', transformedData);
            return transformedData;
        } catch (error) {
            console.error('Failed to load SFTP directory:', error);
            showError(`Failed to connect to ${connection.name}: ${error.message || error}`);
            return null;
        }
    }, [createSftpUrl]);

    // Load SFTP directory
    const loadSftpDirectory = useCallback(async (sftpPath) => {
        console.log('loadSftpDirectory called with:', sftpPath);
        const parsed = parseSftpPath(sftpPath);
        console.log('Parsed SFTP path:', parsed);
        
        if (!parsed || !parsed.connection) {
            console.error('Invalid SFTP path or connection not found', { parsed, sftpPath });
            showError('Invalid SFTP path or connection not found');
            return null;
        }
        
        try {
            const result = await navigateToSftpConnection(parsed.connection, parsed.remotePath);
            console.log('navigateToSftpConnection result:', result);
            return result;
        } catch (error) {
            console.error('Error in loadSftpDirectory:', error);
            showError(`Failed to load SFTP directory: ${error.message || error}`);
            return null;
        }
    }, [parseSftpPath, navigateToSftpConnection]);

    // SFTP file operations
    const createSftpFile = useCallback(async (sftpPath, fileName) => {
        const parsed = parseSftpPath(sftpPath);
        if (!parsed || !parsed.connection) return false;

        try {
            const filePath = `${parsed.remotePath}/${fileName}`.replace(/\/+/g, '/');
            await invoke('create_file_sftp', {
                host: parsed.connection.host,
                port: parseInt(parsed.connection.port, 10),
                username: parsed.connection.username,
                password: parsed.connection.password,
                filePath: filePath
            });
            
            showSuccess(`File "${fileName}" created successfully`);
            return true;
        } catch (error) {
            console.error('Failed to create SFTP file:', error);
            showError(`Failed to create file: ${error.message || error}`);
            return false;
        }
    }, [parseSftpPath]);

    const createSftpDirectory = useCallback(async (sftpPath, dirName) => {
        const parsed = parseSftpPath(sftpPath);
        if (!parsed || !parsed.connection) return false;

        try {
            const dirPath = `${parsed.remotePath}/${dirName}`.replace(/\/+/g, '/');
            await invoke('create_directory_sftp', {
                host: parsed.connection.host,
                port: parseInt(parsed.connection.port, 10),
                username: parsed.connection.username,
                password: parsed.connection.password,
                directoryPath: dirPath
            });
            
            showSuccess(`Directory "${dirName}" created successfully`);
            return true;
        } catch (error) {
            console.error('Failed to create SFTP directory:', error);
            showError(`Failed to create directory: ${error.message || error}`);
            return false;
        }
    }, [parseSftpPath]);

    const deleteSftpItem = useCallback(async (sftpPath) => {
        const parsed = parseSftpPath(sftpPath);
        if (!parsed || !parsed.connection) return false;

        try {
            // Determine if it's a file or directory by checking the current directory data
            // For now, try both and handle errors
            try {
                await invoke('delete_file_sftp', {
                    host: parsed.connection.host,
                    port: parseInt(parsed.connection.port, 10),
                    username: parsed.connection.username,
                    password: parsed.connection.password,
                    filePath: parsed.remotePath
                });
                showSuccess(`File deleted successfully`);
                return true;
            } catch (fileError) {
                // Try as directory if file deletion failed
                await invoke('delete_directory_sftp', {
                    host: parsed.connection.host,
                    port: parseInt(parsed.connection.port, 10),
                    username: parsed.connection.username,
                    password: parsed.connection.password,
                    directoryPath: parsed.remotePath
                });
                showSuccess(`Directory deleted successfully`);
                return true;
            }
        } catch (error) {
            console.error('Failed to delete SFTP item:', error);
            showError(`Failed to delete item: ${error.message || error}`);
            return false;
        }
    }, [parseSftpPath]);

    const renameSftpItem = useCallback(async (sftpPath, newName) => {
        const parsed = parseSftpPath(sftpPath);
        if (!parsed || !parsed.connection) return false;

        try {
            const pathParts = parsed.remotePath.split('/');
            pathParts[pathParts.length - 1] = newName;
            const newPath = pathParts.join('/');

            // Try both file and directory rename
            try {
                await invoke('rename_file_sftp', {
                    host: parsed.connection.host,
                    port: parseInt(parsed.connection.port, 10),
                    username: parsed.connection.username,
                    password: parsed.connection.password,
                    oldPath: parsed.remotePath,
                    newPath: newPath
                });
            } catch (fileError) {
                await invoke('rename_directory_sftp', {
                    host: parsed.connection.host,
                    port: parseInt(parsed.connection.port, 10),
                    username: parsed.connection.username,
                    password: parsed.connection.password,
                    oldPath: parsed.remotePath,
                    newPath: newPath
                });
            }
            
            showSuccess(`Item renamed to "${newName}" successfully`);
            return true;
        } catch (error) {
            console.error('Failed to rename SFTP item:', error);
            showError(`Failed to rename item: ${error.message || error}`);
            return false;
        }
    }, [parseSftpPath]);

    const copySftpItem = useCallback(async (sftpPath, targetPath) => {
        const sourceParsed = parseSftpPath(sftpPath);
        const targetParsed = parseSftpPath(targetPath);
        
        if (!sourceParsed || !targetParsed || 
            !sourceParsed.connection || !targetParsed.connection) {
            return false;
        }

        // Only support copying within the same connection for now
        if (sourceParsed.connection.name !== targetParsed.connection.name) {
            showError('Copying between different SFTP connections is not yet supported');
            return false;
        }

        try {
            // Try both file and directory copy
            try {
                await invoke('copy_file_sftp', {
                    host: sourceParsed.connection.host,
                    port: parseInt(sourceParsed.connection.port, 10),
                    username: sourceParsed.connection.username,
                    password: sourceParsed.connection.password,
                    sourcePath: sourceParsed.remotePath,
                    destinationPath: targetParsed.remotePath
                });
            } catch (fileError) {
                await invoke('copy_directory_sftp', {
                    host: sourceParsed.connection.host,
                    port: parseInt(sourceParsed.connection.port, 10),
                    username: sourceParsed.connection.username,
                    password: sourceParsed.connection.password,
                    sourcePath: sourceParsed.remotePath,
                    destinationPath: targetParsed.remotePath
                });
            }
            
            showSuccess(`Item copied successfully`);
            return true;
        } catch (error) {
            console.error('Failed to copy SFTP item:', error);
            showError(`Failed to copy item: ${error.message || error}`);
            return false;
        }
    }, [parseSftpPath]);

    const moveSftpItem = useCallback(async (sftpPath, targetPath) => {
        const sourceParsed = parseSftpPath(sftpPath);
        const targetParsed = parseSftpPath(targetPath);
        
        if (!sourceParsed || !targetParsed || 
            !sourceParsed.connection || !targetParsed.connection) {
            return false;
        }

        // Only support moving within the same connection for now
        if (sourceParsed.connection.name !== targetParsed.connection.name) {
            showError('Moving between different SFTP connections is not yet supported');
            return false;
        }

        try {
            // Try both file and directory move
            try {
                await invoke('move_file_sftp', {
                    host: sourceParsed.connection.host,
                    port: parseInt(sourceParsed.connection.port, 10),
                    username: sourceParsed.connection.username,
                    password: sourceParsed.connection.password,
                    sourcePath: sourceParsed.remotePath,
                    destinationPath: targetParsed.remotePath
                });
            } catch (fileError) {
                await invoke('move_directory_sftp', {
                    host: sourceParsed.connection.host,
                    port: parseInt(sourceParsed.connection.port, 10),
                    username: sourceParsed.connection.username,
                    password: sourceParsed.connection.password,
                    sourcePath: sourceParsed.remotePath,
                    destinationPath: targetParsed.remotePath
                });
            }
            
            showSuccess(`Item moved successfully`);
            return true;
        } catch (error) {
            console.error('Failed to move SFTP item:', error);
            showError(`Failed to move item: ${error.message || error}`);
            return false;
        }
    }, [parseSftpPath]);

    const openSftpFile = useCallback(async (sftpPath) => {
        const parsed = parseSftpPath(sftpPath);
        if (!parsed || !parsed.connection) return null;

        try {
            const content = await invoke('open_file_sftp', {
                host: parsed.connection.host,
                port: parseInt(parsed.connection.port, 10),
                username: parsed.connection.username,
                password: parsed.connection.password,
                filePath: parsed.remotePath
            });
            
            return content;
        } catch (error) {
            console.error('Failed to open SFTP file:', error);
            showError(`Failed to open file: ${error.message || error}`);
            return null;
        }
    }, [parseSftpPath]);

    const downloadAndOpenSftpFile = useCallback(async (sftpPath, openFile = true) => {
        const parsed = parseSftpPath(sftpPath);
        if (!parsed || !parsed.connection) {
            showError('Invalid SFTP path or connection not found');
            return null;
        }

        try {
            const result = await invoke('download_and_open_sftp_file', {
                host: parsed.connection.host,
                port: parseInt(parsed.connection.port, 10),
                username: parsed.connection.username,
                password: parsed.connection.password,
                filePath: parsed.remotePath,
                openFile: openFile
            });
            
            if (openFile) {
                showSuccess('File opened successfully');
            } else {
                showSuccess('File downloaded successfully');
            }
            
            return result;
        } catch (error) {
            console.error('Failed to download SFTP file:', error);
            showError(`Failed to download file: ${error.message || error}`);
            return null;
        }
    }, [parseSftpPath]);

    const disconnectSftp = useCallback(() => {
        setCurrentSftpConnection(null);
        setCurrentSftpPath(null);
        connectionCache.current.clear();
    }, []);

    const value = {
        sftpConnections,
        currentSftpConnection,
        currentSftpPath,
        loadSftpDirectory,
        createSftpFile,
        createSftpDirectory,
        deleteSftpItem,
        renameSftpItem,
        copySftpItem,
        moveSftpItem,
        openSftpFile,
        downloadAndOpenSftpFile,
        isSftpPath,
        parseSftpPath,
        navigateToSftpConnection,
        disconnectSftp,
        createSftpUrl,
        createSftpPath
    };

    return (
        <SftpContext.Provider value={value}>
            {children}
        </SftpContext.Provider>
    );
}