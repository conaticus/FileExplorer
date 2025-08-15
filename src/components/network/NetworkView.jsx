import React, { useState, useEffect } from 'react';
import { useSftp } from '../../providers/SftpProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import { showSuccess, showError } from '../../utils/NotificationSystem';
import AddSftpConnectionView from '../sidebar/AddSftpConnectionView';
import './networkView.css';

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
            showSuccess(`SFTP connection "${conn.name}" added successfully`);
        } catch (err) {
            showError('Failed to add SFTP connection');
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
                showSuccess(`Connected to ${connection.name}`);
            }
        } catch (error) {
            console.error('Failed to connect to SFTP:', error);
            showError(`Failed to connect to ${connection.name}: ${error.message || error}`);
        }
    };

    return (
        <div className="network-view">
            <div className="network-view-header">
                <h2 className="network-view-title">Network</h2>
                <p className="network-view-subtitle">Manage and connect to remote servers</p>
            </div>

            <div className="network-connections-section">
                <div className="section-header">
                    <h3 className="section-title">SFTP Connections</h3>
                    <button
                        className="add-connection-button"
                        onClick={() => setIsAddSftpModalOpen(true)}
                        title="Add SFTP Connection"
                    >
                        <span className="icon icon-plus"></span>
                        <span>Add Connection</span>
                    </button>
                </div>

                <div className="connections-grid">
                    {localSftpConnections.length === 0 ? (
                        <div className="empty-connections">
                            <div className="empty-connections-icon">
                                <span className="icon icon-network"></span>
                            </div>
                            <h4>No SFTP connections</h4>
                            <p>Add an SFTP connection to get started with remote file access</p>
                            <button
                                className="add-first-connection-button"
                                onClick={() => setIsAddSftpModalOpen(true)}
                            >
                                <span className="icon icon-plus"></span>
                                Add Your First Connection
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
                                        title="Connect to this SFTP server"
                                    >
                                        <span className="icon icon-play"></span>
                                        Connect
                                    </button>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>

            <div className="network-info-section">
                <h3 className="section-title">About Network Features</h3>
                <div className="info-cards">
                    <div className="info-card">
                        <div className="info-icon">
                            <span className="icon icon-shield"></span>
                        </div>
                        <div className="info-content">
                            <h4>Secure Connections</h4>
                            <p>All SFTP connections use secure SSH protocols to protect your data during transfer.</p>
                        </div>
                    </div>
                    <div className="info-card">
                        <div className="info-icon">
                            <span className="icon icon-folder"></span>
                        </div>
                        <div className="info-content">
                            <h4>File Operations</h4>
                            <p>Perform all standard file operations on remote files just like local ones.</p>
                        </div>
                    </div>
                    <div className="info-card">
                        <div className="info-icon">
                            <span className="icon icon-sync"></span>
                        </div>
                        <div className="info-content">
                            <h4>Seamless Integration</h4>
                            <p>Remote files appear alongside local files with full explorer functionality.</p>
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