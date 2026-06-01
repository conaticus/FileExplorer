import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import Modal from './Modal';
import Button from './Button';
import { showSuccess, showError } from '../../utils/NotificationSystem';
import zhCN from '../../locales/zh-CN.js';

/**
 * PermissionHelper - Component to help users grant necessary permissions or browse to folder
 */
const PermissionHelper = ({ isOpen, onClose, directoryPath, directoryName, onDirectorySelected }) => {
    const [isChecking, setIsChecking] = useState(false);
    const [hasAccess, setHasAccess] = useState(false);
    const [isBrowsing, setIsBrowsing] = useState(false);

    // Check if we have access to the directory
    const checkAccess = async () => {
        if (!directoryPath) return;
        
        setIsChecking(true);
        try {
            const hasAccess = await invoke('check_directory_access', { path: directoryPath });
            setHasAccess(hasAccess);
            if (hasAccess) {
                showSuccess(zhCN.permission.accessGrantedMessage.replace('{name}', directoryName));
                setTimeout(onClose, 1000); // Close modal after success
            }
        } catch (error) {
            setHasAccess(false);
            console.log('Access check failed:', error);
        } finally {
            setIsChecking(false);
        }
    };

    // Auto-check when modal opens
    useEffect(() => {
        if (isOpen && directoryPath) {
            checkAccess();
        }
    }, [isOpen, directoryPath]);

    const handleGrantAccess = async () => {
        try {
            await invoke('request_full_disk_access');
        } catch (error) {
            console.error('Failed to open System Preferences:', error);
            showError(zhCN.permission.failedToOpenPreferences);
        }
    };

    const handleTryAgain = () => {
        checkAccess();
    };

    const handleBrowseToFolder = async () => {
        setIsBrowsing(true);
        try {
            // Try to open the specific directory path first
            let selectedPath = null;
            
            // Try to open folder picker starting from the parent directory
            const parentPath = directoryPath ? directoryPath.substring(0, directoryPath.lastIndexOf('/')) : null;
            
            selectedPath = await open({
                directory: true,
                title: zhCN.permission.browseToFolder.replace('{name}', directoryName),
                defaultPath: parentPath || undefined
            });
            
            if (selectedPath) {
                showSuccess(zhCN.permission.browseToFolder.replace('{name}', directoryName) + ': ' + selectedPath);
                onDirectorySelected && onDirectorySelected(selectedPath);
                onClose();
            }
        } catch (error) {
            console.error('Failed to open folder picker:', error);
            showError(zhCN.permission.failedToOpenPicker);
        } finally {
            setIsBrowsing(false);
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={zhCN.permission.accessRequired.replace('{name}', directoryName)}
            size="md"
            footer={
                <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
                    <Button variant="ghost" onClick={onClose}>
                        {zhCN.permission.cancel}
                    </Button>
                    {hasAccess ? (
                        <Button variant="success" onClick={onClose}>
                            <span className="icon icon-check"></span>
                            {zhCN.permission.accessGranted}
                        </Button>
                    ) : (
                        <>
                            <Button variant="ghost" onClick={handleBrowseToFolder} disabled={isBrowsing}>
                                {isBrowsing ? zhCN.permission.opening : zhCN.permission.browseToFolder}
                            </Button>
                            <Button variant="secondary" onClick={handleTryAgain} disabled={isChecking}>
                                {isChecking ? zhCN.permission.checking : zhCN.permission.tryAgain}
                            </Button>
                            <Button variant="primary" onClick={handleGrantAccess}>
                                {zhCN.permission.grantPermission}
                            </Button>
                        </>
                    )}
                </div>
            }
        >
            <div className="permission-helper">
                <div className="permission-icon">
                    {hasAccess ? (
                        <span className="icon icon-check" style={{ color: 'var(--success)', fontSize: '48px' }}></span>
                    ) : (
                        <span className="icon icon-lock" style={{ color: 'var(--warning)', fontSize: '48px' }}></span>
                    )}
                </div>
                
                <div className="permission-content">
                    {hasAccess ? (
                        <div className="success-message">
                            <h3>{zhCN.permission.accessGranted}!</h3>
                            <p>{zhCN.permission.accessGrantedMessage.replace('{name}', directoryName)}</p>
                        </div>
                    ) : (
                        <div className="permission-request">
                            <h3>{zhCN.permission.permissionRequired}</h3>
                            <p dangerouslySetInnerHTML={{ __html: zhCN.permission.permissionNeeded.replace('{name}', directoryName) }} />
                            
                            <div className="instructions">
                                <h4>{zhCN.permission.optionsToAccess.replace('{name}', directoryName)}</h4>
                                <div className="option-section">
                                    <h5>{zhCN.permission.option1Title}</h5>
                                    <p>{zhCN.permission.option1Desc.replace('{name}', directoryName)}</p>
                                </div>
                                <div className="option-section">
                                    <h5>{zhCN.permission.option2Title}</h5>
                                    <ol>
                                        <li>{zhCN.permission.option2Step1}</li>
                                        <li dangerouslySetInnerHTML={{ __html: zhCN.permission.option2Step2 }} />
                                        <li dangerouslySetInnerHTML={{ __html: zhCN.permission.option2Step3.replace('{name}', directoryName) }} />
                                        <li>{zhCN.permission.option2Step4}</li>
                                    </ol>
                                </div>
                            </div>
                            
                            <div className="help-note">
                                <p dangerouslySetInnerHTML={{ __html: zhCN.permission.securityNote.replace('{name}', directoryName) }} />
                            </div>
                        </div>
                    )}
                </div>
            </div>
            
            <style jsx>{`
                .permission-helper {
                    text-align: center;
                    padding: 20px 0;
                }
                
                .permission-icon {
                    margin-bottom: 24px;
                }
                
                .permission-content {
                    text-align: left;
                }
                
                .permission-content h3 {
                    color: var(--text-primary);
                    margin: 0 0 16px 0;
                    font-size: 1.4rem;
                    font-weight: 600;
                }
                
                .permission-content p {
                    color: var(--text-secondary);
                    line-height: 1.6;
                    margin: 0 0 16px 0;
                }
                
                .instructions {
                    background: var(--surface);
                    border: 1px solid var(--border);
                    border-radius: 8px;
                    padding: 20px;
                    margin: 20px 0;
                }
                
                .instructions h4 {
                    color: var(--text-primary);
                    margin: 0 0 12px 0;
                    font-size: 1.1rem;
                    font-weight: 600;
                }
                
                .instructions ol {
                    margin: 0;
                    padding-left: 20px;
                    color: var(--text-primary);
                }
                
                .instructions li {
                    margin: 8px 0;
                    line-height: 1.5;
                }
                
                .option-section {
                    margin: 16px 0;
                    padding: 16px;
                    background: var(--background);
                    border-radius: 6px;
                    border: 1px solid var(--border);
                }
                
                .option-section h5 {
                    color: var(--text-primary);
                    margin: 0 0 8px 0;
                    font-size: 1rem;
                    font-weight: 600;
                }
                
                .option-section p {
                    color: var(--text-secondary);
                    margin: 0 0 8px 0;
                    font-size: 0.9rem;
                }
                
                .help-note {
                    background: var(--info-bg, #e3f2fd);
                    border: 1px solid var(--info-border, #90caf9);
                    border-radius: 6px;
                    padding: 16px;
                    margin-top: 20px;
                }
                
                .help-note p {
                    margin: 0;
                    color: var(--info, #1565c0);
                    font-size: 0.9rem;
                }
                
                .success-message {
                    text-align: center;
                }
                
                .success-message h3 {
                    color: var(--success);
                }
            `}</style>
        </Modal>
    );
};

export default PermissionHelper;