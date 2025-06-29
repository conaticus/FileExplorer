import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettings } from '../../providers/SettingsProvider';
import Modal from '../common/Modal';
import Button from '../common/Button';
import './settings.css';

const SettingsPanel = ({ isOpen, onClose }) => {
    const { settings, error, updateSetting, resetSettings, reloadSettings } = useSettings();
    const [isResetting, setIsResetting] = useState(false);
    const [activeTab, setActiveTab] = useState('appearance');
    const [localError, setLocalError] = useState(null);

    const tabs = [
        { id: 'appearance', label: 'Appearance', icon: 'palette' },
        { id: 'behavior', label: 'Behavior', icon: 'settings' },
        { id: 'search', label: 'Search', icon: 'search' },
        { id: 'advanced', label: 'Advanced', icon: 'cog' }
    ];

    const themes = [
        { id: false, label: 'Light' },
        { id: true, label: 'Dark' }
    ];

    const viewModes = [
        { id: 'Grid', label: 'Grid View' },
        { id: 'List', label: 'List View' },
        { id: 'Details', label: 'Details View' }
    ];

    const fontSizes = [
        { id: 'Small', label: 'Small' },
        { id: 'Medium', label: 'Medium' },
        { id: 'Large', label: 'Large' }
    ];

    const sortOptions = [
        { id: 'Name', label: 'Name' },
        { id: 'Size', label: 'Size' },
        { id: 'Modified', label: 'Date Modified' },
        { id: 'Type', label: 'Type' }
    ];

    const sortDirections = [
        { id: 'Ascending', label: 'Ascending' },
        { id: 'Descending', label: 'Descending' }
    ];

    const doubleClickOptions = [
        { id: 'OpenFilesAndFolders', label: 'Open files and folders' },
        { id: 'SelectFilesAndFolders', label: 'Select files and folders' }
    ];

    const hashAlgorithms = [
        { id: 'MD5', label: 'MD5' },
        { id: 'SHA256', label: 'SHA-256' },
        { id: 'SHA384', label: 'SHA-384' },
        { id: 'SHA512', label: 'SHA-512' },
        { id: 'CRC32', label: 'CRC32' }
    ];

    // Clear local error after some time
    useEffect(() => {
        if (localError) {
            const timer = setTimeout(() => setLocalError(null), 5000);
            return () => clearTimeout(timer);
        }
    }, [localError]);

    const handleReset = async () => {
        if (!confirm('Are you sure you want to reset all settings to default? This cannot be undone.')) {
            return;
        }

        setIsResetting(true);
        try {
            await resetSettings();
            setLocalError(null);
            alert('Settings have been reset to default.');
        } catch (error) {
            console.error('Failed to reset settings:', error);
            setLocalError('Failed to reset settings. Please try again.');
        } finally {
            setIsResetting(false);
        }
    };

    const handleClearSearchIndex = async () => {
        try {
            await invoke('clear_search_engine');
            alert('Search index has been cleared.');
        } catch (error) {
            console.error('Failed to clear search index:', error);
            setLocalError('Failed to clear search index.');
        }
    };

    const renderAppearanceTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>Theme</h3>
                <div className="radio-group">
                    {themes.map(theme => (
                        <label key={theme.id.toString()} className="radio-option">
                            <input
                                type="radio"
                                name="darkmode"
                                value={theme.id}
                                checked={settings.darkmode === theme.id}
                                onChange={(e) => updateSetting('darkmode', e.target.value === 'true')}
                            />
                            <span>{theme.label}</span>
                        </label>
                    ))}
                </div>
            </div>

            <div className="settings-section">
                <h3>Default View</h3>
                <div className="radio-group">
                    {viewModes.map(mode => (
                        <label key={mode.id} className="radio-option">
                            <input
                                type="radio"
                                name="default_view"
                                value={mode.id}
                                checked={settings.default_view === mode.id}
                                onChange={(e) => updateSetting('default_view', e.target.value)}
                            />
                            <span>{mode.label}</span>
                        </label>
                    ))}
                </div>
            </div>

            <div className="settings-section">
                <h3>Font Size</h3>
                <select
                    value={settings.font_size || 'Medium'}
                    onChange={(e) => updateSetting('font_size', e.target.value)}
                    className="settings-select"
                >
                    {fontSizes.map(size => (
                        <option key={size.id} value={size.id}>{size.label}</option>
                    ))}
                </select>
            </div>

            <div className="settings-section">
                <h3>Accent Color</h3>
                <div className="form-group">
                    <input
                        type="color"
                        value={settings.accent_color || '#0672ef'}
                        onChange={(e) => updateSetting('accent_color', e.target.value)}
                        className="color-picker"
                    />
                    <div className="input-hint">
                        Choose your preferred accent color for the interface.
                    </div>
                </div>
            </div>

            <div className="settings-section">
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.show_details_panel || false}
                        onChange={(e) => updateSetting('show_details_panel', e.target.checked)}
                    />
                    <span>Show details panel by default</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.show_file_extensions !== false}
                        onChange={(e) => updateSetting('show_file_extensions', e.target.checked)}
                    />
                    <span>Show file extensions</span>
                </label>
            </div>
        </div>
    );

    const renderBehaviorTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>File Operations</h3>
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.confirm_delete !== false}
                        onChange={(e) => updateSetting('confirm_delete', e.target.checked)}
                    />
                    <span>Confirm before deleting files</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.show_hidden_files_and_folders || false}
                        onChange={(e) => updateSetting('show_hidden_files_and_folders', e.target.checked)}
                    />
                    <span>Show hidden files and folders</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.auto_refresh_dir !== false}
                        onChange={(e) => updateSetting('auto_refresh_dir', e.target.checked)}
                    />
                    <span>Auto-refresh directories</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>Default Sort</h3>
                <div className="form-row">
                    <div className="form-group">
                        <label>Sort by:</label>
                        <select
                            value={settings.sort_by || 'Name'}
                            onChange={(e) => updateSetting('sort_by', e.target.value)}
                            className="settings-select"
                        >
                            {sortOptions.map(option => (
                                <option key={option.id} value={option.id}>{option.label}</option>
                            ))}
                        </select>
                    </div>

                    <div className="form-group">
                        <label>Direction:</label>
                        <select
                            value={settings.sort_direction || 'Ascending'}
                            onChange={(e) => updateSetting('sort_direction', e.target.value)}
                            className="settings-select"
                        >
                            {sortDirections.map(option => (
                                <option key={option.id} value={option.id}>{option.label}</option>
                            ))}
                        </select>
                    </div>
                </div>
            </div>

            <div className="settings-section">
                <h3>Double-click Behavior</h3>
                <div className="radio-group">
                    {doubleClickOptions.map(option => (
                        <label key={option.id} className="radio-option">
                            <input
                                type="radio"
                                name="double_click"
                                value={option.id}
                                checked={settings.double_click === option.id}
                                onChange={(e) => updateSetting('double_click', e.target.value)}
                            />
                            <span>{option.label}</span>
                        </label>
                    ))}
                </div>
            </div>
        </div>
    );

    const renderSearchTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>Search Behavior</h3>
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.case_sensitive_search || false}
                        onChange={(e) => updateSetting('case_sensitive_search', e.target.checked)}
                    />
                    <span>Case-sensitive search by default</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.index_hidden_files || false}
                        onChange={(e) => updateSetting('index_hidden_files', e.target.checked)}
                    />
                    <span>Include hidden files in search index</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.fuzzy_search_enabled !== false}
                        onChange={(e) => updateSetting('fuzzy_search_enabled', e.target.checked)}
                    />
                    <span>Enable fuzzy search</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enable_suggestions !== false}
                        onChange={(e) => updateSetting('enable_suggestions', e.target.checked)}
                    />
                    <span>Enable search suggestions</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.highlight_matches !== false}
                        onChange={(e) => updateSetting('highlight_matches', e.target.checked)}
                    />
                    <span>Highlight search matches</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>Search Index</h3>
                <p>Indexing improves search performance but uses disk space.</p>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.search_engine_enabled !== false}
                        onChange={(e) => updateSetting('search_engine_enabled', e.target.checked)}
                    />
                    <span>Enable search indexing</span>
                </label>

                <Button
                    variant="secondary"
                    onClick={handleClearSearchIndex}
                >
                    Clear Search Index
                </Button>
            </div>
        </div>
    );

    const renderAdvancedTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>Performance</h3>
                <div className="form-group">
                    <label>Terminal height (pixels):</label>
                    <input
                        type="number"
                        min="200"
                        max="600"
                        value={settings.terminal_height || 240}
                        onChange={(e) => updateSetting('terminal_height', parseInt(e.target.value))}
                        className="settings-input"
                    />
                </div>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enable_animations_and_transitions !== false}
                        onChange={(e) => updateSetting('enable_animations_and_transitions', e.target.checked)}
                    />
                    <span>Enable animations and transitions</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enable_virtual_scroll_for_large_directories || false}
                        onChange={(e) => updateSetting('enable_virtual_scroll_for_large_directories', e.target.checked)}
                    />
                    <span>Enable virtual scrolling for large directories</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>Hash Algorithm</h3>
                <div className="form-group">
                    <label>Default hash algorithm:</label>
                    <select
                        value={settings.default_checksum_hash || 'SHA256'}
                        onChange={(e) => updateSetting('default_checksum_hash', e.target.value)}
                        className="settings-select"
                    >
                        {hashAlgorithms.map(algo => (
                            <option key={algo.id} value={algo.id}>{algo.label}</option>
                        ))}
                    </select>
                    <div className="input-hint">
                        Used for generating file hashes and integrity checks.
                    </div>
                </div>
            </div>

            <div className="settings-section">
                <h3>Default Paths</h3>
                <div className="form-group">
                    <label>Default folder on opening:</label>
                    <input
                        type="text"
                        value={settings.default_folder_path_on_opening || ''}
                        onChange={(e) => updateSetting('default_folder_path_on_opening', e.target.value)}
                        className="settings-input"
                        placeholder="Leave empty to use system default"
                    />
                    <div className="input-hint">
                        The folder to open when the application starts.
                    </div>
                </div>
            </div>

            <div className="settings-section danger-zone">
                <h3>Danger Zone</h3>
                <p>These actions cannot be undone.</p>
                <Button
                    variant="danger"
                    onClick={handleReset}
                    disabled={isResetting}
                >
                    {isResetting ? 'Resetting...' : 'Reset All Settings'}
                </Button>
            </div>
        </div>
    );

    const renderTabContent = () => {
        switch (activeTab) {
            case 'appearance': return renderAppearanceTab();
            case 'behavior': return renderBehaviorTab();
            case 'search': return renderSearchTab();
            case 'advanced': return renderAdvancedTab();
            default: return renderAppearanceTab();
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Settings"
            size="lg"
            footer={
                <div className="settings-footer">
                    {(error || localError) && (
                        <div className="settings-error">
                            <span className="icon-alert-triangle"></span>
                            <span>{error || localError}</span>
                        </div>
                    )}
                    <div className="settings-footer-buttons">
                        <Button variant="ghost" onClick={reloadSettings}>
                            Reload
                        </Button>
                        <Button variant="primary" onClick={onClose}>
                            Close
                        </Button>
                    </div>
                </div>
            }
        >
            <div className="settings-panel">
                <div className="settings-sidebar">
                    {tabs.map(tab => (
                        <button
                            key={tab.id}
                            className={`settings-tab ${activeTab === tab.id ? 'active' : ''}`}
                            onClick={() => setActiveTab(tab.id)}
                        >
                            <span className={`icon icon-${tab.icon}`}></span>
                            <span>{tab.label}</span>
                        </button>
                    ))}
                </div>

                <div className="settings-content">
                    {renderTabContent()}
                </div>
            </div>
        </Modal>
    );
};

export default SettingsPanel;