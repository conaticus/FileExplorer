import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import FileIcon from '../explorer/FileIcon';
import { formatFileSize, formatDate } from '../../utils/formatters';
import './thisPc.css';

const ThisPCView = () => {
    const [systemInfo, setSystemInfo] = useState(null);
    const [isLoading, setIsLoading] = useState(true);
    const { volumes, loadDirectory } = useFileSystem();
    const { navigateTo } = useHistory();

    // Common folders for different operating systems
    const [userFolders, setUserFolders] = useState([]);

    useEffect(() => {
        loadSystemInfo();
        loadUserFolders();
    }, []);

    const loadSystemInfo = async () => {
        try {
            const metaDataJson = await invoke('get_meta_data_as_json');
            const metaData = JSON.parse(metaDataJson);
            setSystemInfo(metaData);
        } catch (error) {
            console.error('Failed to load system info:', error);
        } finally {
            setIsLoading(false);
        }
    };

    const loadUserFolders = async () => {
        // Try to detect user folders based on OS
        const folders = [];

        try {
            // Common user folder paths
            const commonPaths = [
                { name: 'Desktop', path: getDesktopPath(), icon: 'desktop' },
                { name: 'Documents', path: getDocumentsPath(), icon: 'documents' },
                { name: 'Downloads', path: getDownloadsPath(), icon: 'downloads' },
                { name: 'Pictures', path: getPicturesPath(), icon: 'pictures' },
                { name: 'Music', path: getMusicPath(), icon: 'music' },
                { name: 'Videos', path: getVideosPath(), icon: 'videos' },
            ];

            // Check which folders exist
            for (const folder of commonPaths) {
                try {
                    await invoke('open_directory', { path: folder.path });
                    folders.push(folder);
                } catch {
                    // Folder doesn't exist or isn't accessible
                }
            }
        } catch (error) {
            console.error('Failed to load user folders:', error);
        }

        setUserFolders(folders);
    };

    const getDesktopPath = () => {
        if (systemInfo?.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Desktop`;
        }
        return `${systemInfo?.user_home_dir || '/home/user'}/Desktop`;
    };

    const getDocumentsPath = () => {
        if (systemInfo?.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Documents`;
        }
        return `${systemInfo?.user_home_dir || '/home/user'}/Documents`;
    };

    const getDownloadsPath = () => {
        if (systemInfo?.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Downloads`;
        }
        return `${systemInfo?.user_home_dir || '/home/user'}/Downloads`;
    };

    const getPicturesPath = () => {
        if (systemInfo?.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Pictures`;
        }
        return `${systemInfo?.user_home_dir || '/home/user'}/Pictures`;
    };

    const getMusicPath = () => {
        if (systemInfo?.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Music`;
        }
        return `${systemInfo?.user_home_dir || '/home/user'}/Music`;
    };

    const getVideosPath = () => {
        if (systemInfo?.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Videos`;
        }
        return `${systemInfo?.user_home_dir || '/home/user'}/Videos`;
    };

    const handleFolderClick = async (path) => {
        try {
            await loadDirectory(path);
            navigateTo(path);
        } catch (error) {
            console.error('Failed to navigate to folder:', error);
            alert(`Cannot access ${path}. The folder may not exist or is inaccessible.`);
        }
    };

    const handleVolumeClick = (volume) => {
        handleFolderClick(volume.mount_point);
    };

    const ejectVolume = async (volume) => {
        if (!volume.is_removable) return;

        const confirmEject = confirm(`Are you sure you want to safely eject ${volume.volume_name}?`);
        if (!confirmEject) return;

        try {
            // Use system command to eject the volume
            const command = systemInfo?.current_running_os === 'windows'
                ? `eject ${volume.mount_point}`
                : `umount ${volume.mount_point}`;

            await invoke('execute_command', { command });
            alert(`${volume.volume_name} has been safely ejected.`);
        } catch (error) {
            console.error('Failed to eject volume:', error);
            alert(`Failed to eject ${volume.volume_name}: ${error.message || error}`);
        }
    };

    if (isLoading) {
        return (
            <div className="this-pc-loading">
                <div className="spinner"></div>
                <p>Loading system information...</p>
            </div>
        );
    }

    return (
        <div className="this-pc-view">
            <div className="this-pc-header">
                <h2>This PC</h2>
                {systemInfo && (
                    <div className="system-info">
                        <span>Running {systemInfo.current_running_os}</span>
                        {systemInfo.current_cpu_architecture && (
                            <span> • {systemInfo.current_cpu_architecture}</span>
                        )}
                    </div>
                )}
            </div>

            {/* User Folders Section */}
            {userFolders.length > 0 && (
                <div className="pc-section">
                    <h3>Folders</h3>
                    <div className="folders-grid">
                        {userFolders.map((folder) => (
                            <div
                                key={folder.path}
                                className="folder-item"
                                onClick={() => handleFolderClick(folder.path)}
                            >
                                <div className="folder-icon">
                                    <span className={`icon icon-${folder.icon}`}></span>
                                </div>
                                <div className="folder-details">
                                    <div className="folder-name">{folder.name}</div>
                                    <div className="folder-path">{folder.path}</div>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Drives Section */}
            <div className="pc-section">
                <h3>Drives</h3>
                <div className="drives-grid">
                    {volumes.map((volume) => {
                        const usedSpace = volume.size - volume.available_space;
                        const usedPercentage = (usedSpace / volume.size) * 100;

                        return (
                            <div
                                key={volume.mount_point}
                                className="drive-item"
                                onClick={() => handleVolumeClick(volume)}
                            >
                                <div className="drive-icon">
                                    <span className={`icon icon-${volume.is_removable ? 'usb' : 'drive'}`}></span>
                                </div>

                                <div className="drive-details">
                                    <div className="drive-name">
                                        {volume.volume_name || volume.mount_point}
                                    </div>
                                    <div className="drive-path">
                                        {volume.mount_point} • {volume.file_system}
                                    </div>

                                    <div className="drive-storage">
                                        <div className="storage-info">
                                            <span>{formatFileSize(volume.available_space)} free of {formatFileSize(volume.size)}</span>
                                        </div>
                                        <div className="storage-bar">
                                            <div
                                                className="storage-used"
                                                style={{ width: `${Math.min(usedPercentage, 100)}%` }}
                                            ></div>
                                        </div>
                                    </div>
                                </div>

                                {volume.is_removable && (
                                    <div className="drive-actions">
                                        <button
                                            className="eject-button"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                ejectVolume(volume);
                                            }}
                                            title="Safely eject"
                                        >
                                            <span className="icon icon-eject"></span>
                                        </button>
                                    </div>
                                )}
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* System Information */}
            {systemInfo && (
                <div className="pc-section">
                    <h3>System Information</h3>
                    <div className="system-details">
                        <div className="detail-row">
                            <span className="detail-label">Operating System:</span>
                            <span className="detail-value">{systemInfo.current_running_os}</span>
                        </div>
                        {systemInfo.current_cpu_architecture && (
                            <div className="detail-row">
                                <span className="detail-label">Architecture:</span>
                                <span className="detail-value">{systemInfo.current_cpu_architecture}</span>
                            </div>
                        )}
                        {systemInfo.user_home_dir && (
                            <div className="detail-row">
                                <span className="detail-label">User Directory:</span>
                                <span className="detail-value">{systemInfo.user_home_dir}</span>
                            </div>
                        )}
                        {systemInfo.version && (
                            <div className="detail-row">
                                <span className="detail-label">Version:</span>
                                <span className="detail-value">{systemInfo.version}</span>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};

export default ThisPCView;