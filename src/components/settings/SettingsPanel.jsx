import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettings } from '../../providers/SettingsProvider';
import Modal from '../common/Modal';
import Button from '../common/Button';
import './settings.css';
import zhCN from '../../locales/zh-CN.js';

/**
 * SettingsPanel component - Provides a comprehensive settings interface
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Whether the settings panel is open
 * @param {Function} props.onClose - Callback function when panel is closed
 * @returns {React.ReactElement} Settings panel modal component
 */
const SettingsPanel = ({ isOpen, onClose }) => {
    const { settings, error, updateSetting, resetSettings, reloadSettings } = useSettings();
    const [isResetting, setIsResetting] = useState(false);
    const [activeTab, setActiveTab] = useState('appearance');
    const [localError, setLocalError] = useState(null);

    /**
     * Available tabs configuration
     * @type {Array<{id: string, label: string, icon: string}>}
     */
    const tabs = [
        { id: 'appearance', label: zhCN.settings.appearance, icon: 'palette' },
        { id: 'behavior', label: zhCN.settings.behavior, icon: 'settings' },
        { id: 'search', label: zhCN.settings.search, icon: 'search' },
        { id: 'advanced', label: zhCN.settings.advanced, icon: 'cog' }
    ];

    /**
     * Theme options configuration
     * @type {Array<{id: boolean, label: string}>}
     */
    const themes = [
        { id: false, label: zhCN.settings.light },
        { id: true, label: zhCN.settings.dark }
    ];

    /**
     * View mode options configuration
     * @type {Array<{id: string, label: string}>}
     */
    const viewModes = [
        { id: 'grid', label: zhCN.settings.gridView },
        { id: 'list', label: zhCN.settings.listView },
        { id: 'details', label: zhCN.settings.detailsView }
    ];

    /**
     * Font size options configuration
     * @type {Array<{id: string, label: string}>}
     */
    const fontSizes = [
        { id: 'Small', label: zhCN.settings.small },
        { id: 'Medium', label: zhCN.settings.medium },
        { id: 'Large', label: zhCN.settings.large }
    ];

    /**
     * Sort options configuration
     * @type {Array<{id: string, label: string}>}
     */
    const sortOptions = [
        { id: 'Name', label: zhCN.settings.sortByOptions.name },
        { id: 'Size', label: zhCN.settings.sortByOptions.size },
        { id: 'Modified', label: zhCN.settings.sortByOptions.modified },
        { id: 'Type', label: zhCN.settings.sortByOptions.type }
    ];

    /**
     * Sort direction options configuration
     * @type {Array<{id: string, label: string}>}
     */
    const sortDirections = [
        { id: 'Ascending', label: zhCN.settings.ascending },
        { id: 'Descending', label: zhCN.settings.descending }
    ];

    /**
     * Double-click behavior options configuration
     * @type {Array<{id: string, label: string}>}
     */
    const doubleClickOptions = [
        { id: 'OpenFilesAndFolders', label: zhCN.settings.openFilesAndFolders },
        { id: 'SelectFilesAndFolders', label: zhCN.settings.selectFilesAndFolders }
    ];

    /**
     * Hash algorithm options configuration
     * @type {Array<{id: string, label: string}>}
     */
    const hashAlgorithms = [
        { id: 'MD5', label: 'MD5' },
        { id: 'SHA256', label: 'SHA-256' },
        { id: 'SHA384', label: 'SHA-384' },
        { id: 'SHA512', label: 'SHA-512' },
        { id: 'CRC32', label: 'CRC32' }
    ];

    /**
     * Effect to clear local error after timeout
     */
    useEffect(() => {
        if (localError) {
            const timer = setTimeout(() => setLocalError(null), 5000);
            return () => clearTimeout(timer);
        }
    }, [localError]);

    /**
     * Handles resetting all settings to default values
     * Confirms with user before proceeding
     * @async
     */
    const handleReset = async () => {
        if (!confirm(zhCN.settings.confirmResetSettings)) {
            return;
        }

        setIsResetting(true);
        try {
            await resetSettings();
            setLocalError(null);
            alert(zhCN.settings.settingsResetSuccess);
        } catch (error) {
            console.error('Failed to reset settings:', error);
            setLocalError(zhCN.settings.settingsResetFailed);
        } finally {
            setIsResetting(false);
        }
    };

    /**
     * Handles clearing the search index
     * @async
     */
    const handleClearSearchIndex = async () => {
        try {
            await invoke('clear_search_engine');
            alert(zhCN.settings.searchIndexCleared);
        } catch (error) {
            console.error('Failed to clear search index:', error);
            setLocalError('清除搜索索引失败');
        }
    };

    /**
     * Renders the appearance settings tab content
     * @returns {React.ReactElement} Appearance tab content
     */
    const renderAppearanceTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>{zhCN.settings.theme}</h3>
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
                <h3>{zhCN.settings.defaultView}</h3>
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
                <h3>{zhCN.settings.fontSize}</h3>
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
                <h3>{zhCN.settings.accentColor}</h3>
                <div className="form-group">
                    <input
                        type="color"
                        value={settings.accent_color || '#0672ef'}
                        onChange={(e) => updateSetting('accent_color', e.target.value)}
                        className="color-picker"
                    />
                    <div className="input-hint">
                        {zhCN.settings.accentColorHint}
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
                    <span>{zhCN.settings.showDetailsPanel}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.show_file_extensions !== false}
                        onChange={(e) => updateSetting('show_file_extensions', e.target.checked)}
                    />
                    <span>{zhCN.settings.showFileExtensions}</span>
                </label>
            </div>
        </div>
    );

    /**
     * Renders the behavior settings tab content
     * @returns {React.ReactElement} Behavior tab content
     */
    const renderBehaviorTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>{zhCN.settings.fileOperations}</h3>
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.confirm_delete !== false}
                        onChange={(e) => updateSetting('confirm_delete', e.target.checked)}
                    />
                    <span>{zhCN.settings.confirmBeforeDelete}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.show_hidden_files_and_folders || false}
                        onChange={(e) => updateSetting('show_hidden_files_and_folders', e.target.checked)}
                    />
                    <span>{zhCN.settings.showHiddenFiles}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.auto_refresh_dir !== false}
                        onChange={(e) => updateSetting('auto_refresh_dir', e.target.checked)}
                    />
                    <span>{zhCN.settings.autoRefreshDirs}</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>{zhCN.settings.defaultSort}</h3>
                <div className="form-row">
                    <div className="form-group">
                        <label>{zhCN.settings.sortBy}</label>
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
                        <label>{zhCN.settings.direction}</label>
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
                <h3>{zhCN.settings.doubleClickBehavior}</h3>
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

    /**
     * Renders the search settings tab content
     * @returns {React.ReactElement} Search tab content
     */
    const renderSearchTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>{zhCN.settings.searchBehavior}</h3>
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.case_sensitive_search || false}
                        onChange={(e) => updateSetting('case_sensitive_search', e.target.checked)}
                    />
                    <span>{zhCN.settings.caseSensitiveSearch}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.index_hidden_files || false}
                        onChange={(e) => updateSetting('index_hidden_files', e.target.checked)}
                    />
                    <span>{zhCN.settings.indexHiddenFiles}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.fuzzy_search_enabled !== false}
                        onChange={(e) => updateSetting('fuzzy_search_enabled', e.target.checked)}
                    />
                    <span>{zhCN.settings.enableFuzzySearch}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enable_suggestions !== false}
                        onChange={(e) => updateSetting('enable_suggestions', e.target.checked)}
                    />
                    <span>{zhCN.settings.enableSuggestions}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.highlight_matches !== false}
                        onChange={(e) => updateSetting('highlight_matches', e.target.checked)}
                    />
                    <span>{zhCN.settings.highlightMatches}</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>{zhCN.settings.searchIndex}</h3>
                <p>{zhCN.settings.searchIndexHint}</p>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.search_engine_enabled !== false}
                        onChange={(e) => updateSetting('search_engine_enabled', e.target.checked)}
                    />
                    <span>{zhCN.settings.enableSearchIndex}</span>
                </label>

                <Button
                    variant="secondary"
                    onClick={handleClearSearchIndex}
                >
                    {zhCN.settings.clearSearchIndex}
                </Button>
            </div>
        </div>
    );

    /**
     * Renders the advanced settings tab content
     * @returns {React.ReactElement} Advanced tab content
     */
    const renderAdvancedTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>{zhCN.settings.performance}</h3>
                <div className="form-group">
                    <label>{zhCN.settings.terminalHeight}</label>
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
                    <span>{zhCN.settings.enableAnimations}</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enable_virtual_scroll_for_large_directories || false}
                        onChange={(e) => updateSetting('enable_virtual_scroll_for_large_directories', e.target.checked)}
                    />
                    <span>{zhCN.settings.enableVirtualScroll}</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>{zhCN.settings.hashAlgorithm}</h3>
                <div className="form-group">
                    <label>{zhCN.settings.defaultHashAlgorithm}</label>
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
                        {zhCN.settings.hashAlgorithmHint}
                    </div>
                </div>
            </div>

            <div className="settings-section">
                <h3>{zhCN.settings.defaultPaths}</h3>
                <div className="form-group">
                    <label>{zhCN.settings.defaultFolderOnOpen}</label>
                    <input
                        type="text"
                        value={settings.default_folder_path_on_opening || ''}
                        onChange={(e) => updateSetting('default_folder_path_on_opening', e.target.value)}
                        className="settings-input"
                        placeholder={zhCN.settings.leaveEmptyForDefault}
                    />
                    <div className="input-hint">
                        {zhCN.settings.defaultFolderHint}
                    </div>
                </div>
            </div>

            <div className="settings-section danger-zone">
                <h3>{zhCN.settings.dangerZone}</h3>
                <p>{zhCN.settings.dangerZoneHint}</p>
                <Button
                    variant="danger"
                    onClick={handleReset}
                    disabled={isResetting}
                >
                    {isResetting ? zhCN.settings.resetting : zhCN.settings.resetAllSettings}
                </Button>
            </div>
        </div>
    );

    /**
     * Renders the appropriate tab content based on activeTab state
     * @returns {React.ReactElement} The content for the active tab
     */
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
            title={zhCN.settings.title}
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
                        <Button variant="primary" onClick={reloadSettings}>
                            {zhCN.settings.saveAndApply}
                        </Button>
                        <Button variant="ghost" onClick={onClose}>
                            {zhCN.common.close}
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

