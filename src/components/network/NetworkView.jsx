import React, { useState, useEffect } from 'react';
import { useSftp } from '../../providers/SftpProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { showSuccess, showError } from '../../utils/NotificationSystem';
import AddSftpConnectionView from '../sidebar/AddSftpConnectionView';
import './networkView.css';
import zhCN from '../../locales/zh-CN.js';

/**
 * NetworkView component that displays available network connections
 * and provides management for SFTP connections
 */
const NetworkView = () => {
    const { sftpConnections, navigateToSftpConnection, createSftpUrl } = useSftp();
    const { loadDirectory } = useFileSystem();
    const { navigateTo } = useHistory();
    const [isAddSftpModalOpen, setIsAddSftpModalOpen] = useState(false);
    const [localSftpConnections, setLocalSftpConnections] = useState([]);

    // Load SFTP connections
    useEffect(() => {
        const loadConnections = () => {
            try {
                const saved = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
                setLocalSftpConnections(saved);
            } catch (err) {
                setLocalSftpConnections([]);
            }
        };

        loadConnections();
        
        // Listen for connection updates
        const handler = () => loadConnections();
        window.addEventListener('sftp-connections-updated', handler);
        window.addEventListener('storage', (e) => {
            if (e.key === 'fileExplorerSftpConnections') loadConnections();
        });
        
        return () => {
            window.removeEventListener('sftp-connections-updated', handler);
        };
    }, []);

    // Add SFTP connection
    const addSftpConnection = (conn) => {
        try {
            const existing = JSON.parse(localStorage.getItem('fileExplorerSftpConnections') || '[]');
            const newConnections = [...existing, conn];
            localStorage.setItem('fileExplorerSftpConnections', JSON.stringify(newConnections));
            window.dispatchEvent(new CustomEvent('sftp-connections-updated'));
            window.dispatchEvent(new StorageEvent('storage', {
                key: 'fileExplorerSftpConnections',
                newValue: JSON.stringify(newConnections)
            }));
            showSuccess(zhCN.network.connectionAdded.replace('{name}', conn.name));
        } catch (err) {
            showError(zhCN.network.connectionFailed);
        }
        setIsAddSftpModalOpen(false);
    };

    // Connect to SFTP server
    const handleConnectToSftp = async (connection) => {
        try {
            const sftpData = await navigateToSftpConnection(connection);
            if (sftpData) {
                const sftpPath = createSftpUrl(connection, '.');
                await loadDirectory(sftpPath);
                navigateTo(sftpPath);
                showSuccess(zhCN.network.connected.replace('{name}', connection.name));
            }
        } catch (error) {
            console.error('Failed to connect to SFTP:', error);
            showError(`${zhCN.network.connectionFailed}: ${error.message || error}`);
        }
    };

    return (
        <div className="network-view">
            <div className="network-view-header">
                <h2 className="network-view-title">{zhCN.network.title}</h2>
                <p className="network-view-subtitle">{zhCN.network.subtitle}</p>
            </div>

            <div className="network-connections-section">
                <div className="section-header">
                    <h3 className="section-title">{zhCN.network.sftpConnections}</h3>
                    <button
                        className="add-connection-button"
                        onClick={() => setIsAddSftpModalOpen(true)}
                        title={zhCN.network.addConnection}
                    >
                        <span className="icon icon-plus"></span>
                        <span>{zhCN.network.addConnection}</span>
                    </button>
                </div>

                <div className="connections-grid">
                    {localSftpConnections.length === 0 ? (
                        <div className="empty-connections">
                            <div className="empty-connections-icon">
                                <span className="icon icon-network"></span>
                            </div>
                            <h4>{zhCN.network.noConnections}</h4>
                            <p>{zhCN.network.addConnectionToStart}</p>
                            <button
                                className="add-first-connection-button"
                                onClick={() => setIsAddSftpModalOpen(true)}
                            >
                                <span className="icon icon-plus"></span>
                                {zhCN.network.addFirstConnection}
                            </button>
                        </div>
                    ) : (
                        localSftpConnections.map((connection) => (
                            <div key={connection.name} className="connection-card">
                                <div className="connection-icon">
                                    <span className="icon icon-network"></span>
                                </div>
                                <div className="connection-details">
                                    <h4 className="connection-name">{connection.name}</h4>
                                    <p className="connection-address">
                                        {connection.username}@{connection.host}:{connection.port}
                                    </p>
                                </div>
                                <div className="connection-actions">
                                    <button
                                        className="connect-button"
                                        onClick={() => handleConnectToSftp(connection)}
                                        title={zhCN.network.connect}
                                    >
                                        <span className="icon icon-play"></span>
                                        {zhCN.network.connect}
                                    </button>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>

            <div className="network-info-section">
                <h3 className="section-title">{zhCN.network.aboutNetwork}</h3>
                <div className="info-cards">
                    <div className="info-card">
                        <div className="info-icon">
                            <span className="icon icon-shield"></span>
                        </div>
                        <div className="info-content">
                            <h4>{zhCN.network.secureConnections}</h4>
                            <p>{zhCN.network.secureConnectionsDesc}</p>
                        </div>
                    </div>
                    <div className="info-card">
                        <div className="info-icon">
                            <span className="icon icon-folder"></span>
                        </div>
                        <div className="info-content">
                            <h4>{zhCN.network.fileOperations}</h4>
                            <p>{zhCN.network.fileOperationsDesc}</p>
                        </div>
                    </div>
                    <div className="info-card">
                        <div className="info-icon">
                            <span className="icon icon-sync"></span>
                        </div>
                        <div className="info-content">
                            <h4>{zhCN.network.seamlessIntegration}</h4>
                            <p>{zhCN.network.seamlessIntegrationDesc}</p>
                        </div>
                    </div>
                </div>
            </div>

            {/* Add SFTP Connection Modal */}
            <AddSftpConnectionView
                isOpen={isAddSftpModalOpen}
                onClose={() => setIsAddSftpModalOpen(false)}
                onAdd={addSftpConnection}
            />
        </div>
    );
};

export default NetworkView;