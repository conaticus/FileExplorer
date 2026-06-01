import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { formatFileSize } from '../../utils/formatters';
import './thisPc.css';
import {showConfirm, showError, showSuccess} from "../../utils/NotificationSystem.js";
import zhCN from '../../locales/zh-CN.js';

/**
 * ThisPCView component - Displays system information, user folders, and storage drives
 * Similar to "This PC" or "My Computer" in Windows Explorer
 *
 * @returns {React.ReactElement} ThisPCView component
 */
const ThisPCView = () => {
    const [systemInfo, setSystemInfo] = useState(null);
    const [isLoading, setIsLoading] = useState(true);
    const { volumes, loadDirectory, loadVolumes } = useFileSystem();
    const { navigateTo } = useHistory();

    // Common folders for different operating systems
    const [userFolders, setUserFolders] = useState([]);

    /**
     * Load system information on component mount
     */
    useEffect(() => {
        loadSystemInfo();
    }, []);

    /**
     * Load user folders when systemInfo is available
     */
    useEffect(() => {
        if (systemInfo) {
            loadUserFolders();
        }
    }, [systemInfo]);

    /**
     * Loads system information from the backend
     * @async
     */
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

    /**
     * Loads user folders based on the current operating system
     * Checks which standard folders exist and adds them to the userFolders state
     * @async
     */
    const loadUserFolders = async () => {
        if (!systemInfo) return;

        console.log('Loading user folders for OS:', systemInfo.current_running_os);
        console.log('User home directory:', systemInfo.user_home_dir);

        const folders = [];

        try {
            // Get all possible paths for each folder type
            const videoPaths = getVideosPaths();
            console.log('Video paths to check:', videoPaths);

            // Common user folder paths - some folders might have multiple possible names
            const folderConfigs = [
                {
                    name: zhCN.folders.desktop,
                    paths: [getDesktopPath()],
                    icon: 'desktop'
                },
                {
                    name: zhCN.folders.documents,
                    paths: [getDocumentsPath()],
                    icon: 'documents'
                },
                {
                    name: zhCN.folders.downloads,
                    paths: [getDownloadsPath()],
                    icon: 'downloads'
                },
                {
                    name: zhCN.folders.pictures,
                    paths: [getPicturesPath()],
                    icon: 'pictures'
                },
                {
                    name: zhCN.folders.music,
                    paths: [getMusicPath()],
                    icon: 'music'
                },
                {
                    name: zhCN.folders.videos,
                    paths: videoPaths,
                    icon: 'videos'
                }
            ];

            // Check which folders exist for each configuration
            for (const config of folderConfigs) {
                let foundPath = null;

                console.log(`Checking ${config.name} with paths:`, config.paths);

                // Try each possible path for this folder type
                for (const path of config.paths) {
                    console.log(`Trying path: ${path}`);
                    try {
                        await invoke('open_directory', { path });
                        console.log(`Path exists and is accessible: ${path}`);
                        foundPath = path;
                        break; // Use the first path that works
                    } catch (error) {
                        console.log(`Path failed: ${path}`, error.message || error);
                        // This path doesn't exist or isn't accessible, try the next one
                        continue;
                    }
                }

                // If we found a working path, add it to the folders list
                if (foundPath) {
                    console.log(`Adding ${config.name} folder with path: ${foundPath}`);
                    folders.push({
                        name: config.name,
                        path: foundPath,
                        icon: config.icon
                    });
                } else {
                    console.log(`No accessible path found for ${config.name}`);
                }
            }
        } catch (error) {
            console.error('Failed to load user folders:', error);
        }

        console.log('Final user folders:', folders);
        setUserFolders(folders);
    };

    /**
     * Gets the desktop folder path based on the current OS
     * @returns {string} Desktop folder path
     */
    const getDesktopPath = () => {
        if (systemInfo.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Desktop`;
        }
        return `${systemInfo.user_home_dir}/Desktop`;
    };

    /**
     * Gets the documents folder path based on the current OS
     * @returns {string} Documents folder path
     */
    const getDocumentsPath = () => {
        if (systemInfo.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Documents`;
        }
        return `${systemInfo.user_home_dir}/Documents`;
    };

    /**
     * Gets the downloads folder path based on the current OS
     * @returns {string} Downloads folder path
     */
    const getDownloadsPath = () => {
        if (systemInfo.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Downloads`;
        }
        return `${systemInfo.user_home_dir}/Downloads`;
    };

    /**
     * Gets the pictures folder path based on the current OS
     * @returns {string} Pictures folder path
     */
    const getPicturesPath = () => {
        if (systemInfo.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Pictures`;
        }
        return `${systemInfo.user_home_dir}/Pictures`;
    };

    /**
     * Gets the music folder path based on the current OS
     * @returns {string} Music folder path
     */
    const getMusicPath = () => {
        if (systemInfo.current_running_os === 'windows') {
            return `${systemInfo.user_home_dir}\\Music`;
        }
        return `${systemInfo.user_home_dir}/Music`;
    };

    /**
     * Gets possible paths for videos folder based on the current OS
     * Handles different naming conventions across operating systems
     * @returns {Array<string>} Array of possible video folder paths
     */
    const getVideosPaths = () => {
        if (!systemInfo) {
            console.warn('getVideosPaths called without systemInfo');
            return [];
        }

        // Return an array of possible video folder paths
        let paths;
        if (systemInfo.current_running_os === 'windows') {
            paths = [
                `${systemInfo.user_home_dir}\\Videos`,
                `${systemInfo.user_home_dir}\\Movies`
            ];
        } else {
            paths = [
                `${systemInfo.user_home_dir}/Movies`,
                `${systemInfo.user_home_dir}/Videos`
            ];
        }

        console.log('Generated video paths:', paths);
        return paths;
    };

    /**
     * Handles clicking on a folder item
     * Navigates to the selected folder
     * @param {string} path - The path to navigate to
     * @async
     */
    const handleFolderClick = async (path) => {
        console.log('Clicking folder with path:', path);
        try {
            await loadDirectory(path);
            navigateTo(path);
        } catch (error) {
            console.error('Failed to navigate to folder:', error);
            showError(`无法访问 ${path}。文件夹可能不存在或无法访问。`);
        }
    };

    /**
     * Handles clicking on a volume/drive item
     * @param {Object} volume - The volume object to navigate to
     */
    const handleVolumeClick = (volume) => {
        handleFolderClick(volume.mount_point);
    };

    /**
     * Safely ejects a removable volume
     * @param {Object} volume - The volume to eject
     * @async
     */
    const ejectVolume = async (volume) => {
        if (!volume.is_removable) return;

        const confirmEject = await showConfirm(`确定要安全弹出 ${volume.volume_name} 吗？`);
        if (!confirmEject) return;

        try {
            let command;
            const os = systemInfo?.current_running_os?.toLowerCase();
            
            if (os === 'windows') {
                command = `eject ${volume.mount_point}`;
            } else if (os === 'macos' || os === 'darwin') {
                // Use diskutil for proper ejection on macOS
                command = `diskutil eject "${volume.mount_point}"`;
            } else {
                // Linux and other Unix-like systems
                command = `umount "${volume.mount_point}"`;
            }

            const result = await invoke('execute_command', { command });
            
            // Parse the command result to check for success
            const commandResponse = JSON.parse(result);
            
            if (commandResponse.status === 0) {
                showSuccess(`${volume.volume_name} 已安全弹出。`);
                // Reload volumes to update the UI after ejection
                setTimeout(() => {
                    loadVolumes();
                }, 1000);
            } else {
                throw new Error(commandResponse.stderr || commandResponse.stdout || '弹出失败');
            }
        } catch (error) {
            console.error('Failed to eject volume:', error);
            let errorMessage = error.message || error;
            
            // Parse error message if it's JSON
            try {
                const parsedError = JSON.parse(errorMessage);
                errorMessage = parsedError.custom_message || parsedError.error_message || errorMessage;
            } catch (e) {
                // If not JSON, use as-is
            }
            
            showError(`弹出 ${volume.volume_name} 失败: ${errorMessage}`);
        }
    };

    if (isLoading) {
        return (
            <div className="this-pc-loading">
                <div className="spinner"></div>
                <p>{zhCN.thisPC.loadingSystemInfo}</p>
            </div>
        );
    }

    return (
        <div className="this-pc-view">
            <div className="this-pc-header">
                <h2>{zhCN.thisPC.title}</h2>
                {systemInfo && (
                    <div className="system-info">
                        <span>{zhCN.thisPC.runningOn.replace('{os}', systemInfo.current_running_os)}</span>
                        {systemInfo.current_cpu_architecture && (
                            <span> • {systemInfo.current_cpu_architecture}</span>
                        )}
                    </div>
                )}
            </div>

            {/* User Folders Section */}
            {userFolders.length > 0 && (
                <div className="pc-section">
                    <h3>{zhCN.thisPC.folders}</h3>
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
                <h3>{zhCN.thisPC.drives}</h3>
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
                                            <span>{formatFileSize(volume.available_space)} {zhCN.thisPC.freeOf} {formatFileSize(volume.size)}</span>
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
                                            title="安全弹出"
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
                    <h3>{zhCN.thisPC.systemInfo}</h3>
                    <div className="system-details">
                        <div className="detail-row">
                            <span className="detail-label">{zhCN.thisPC.operatingSystem}</span>
                            <span className="detail-value">{systemInfo.current_running_os}</span>
                        </div>
                        {systemInfo.current_cpu_architecture && (
                            <div className="detail-row">
                                <span className="detail-label">{zhCN.thisPC.architecture}</span>
                                <span className="detail-value">{systemInfo.current_cpu_architecture}</span>
                            </div>
                        )}
                        {systemInfo.user_home_dir && (
                            <div className="detail-row">
                                <span className="detail-label">{zhCN.thisPC.userDirectory}</span>
                                <span className="detail-value">{systemInfo.user_home_dir}</span>
                            </div>
                        )}
                        {systemInfo.version && (
                            <div className="detail-row">
                                <span className="detail-label">{zhCN.thisPC.version}</span>
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

