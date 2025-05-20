import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettings } from '../../providers/SettingsProvider';
import Modal from '../common/Modal';
import Button from '../common/Button';
import './settings.css';

const SettingsPanel = ({ isOpen, onClose }) => {
    const { settings, updateSetting, resetSettings } = useSettings();
    const [isResetting, setIsResetting] = useState(false);
    const [activeTab, setActiveTab] = useState('appearance');

    const tabs = [
        { id: 'appearance', label: 'Appearance', icon: 'palette' },
        { id: 'behavior', label: 'Behavior', icon: 'settings' },
        { id: 'search', label: 'Search', icon: 'search' },
        { id: 'advanced', label: 'Advanced', icon: 'cog' }
    ];

    const themes = [
        { id: 'light', label: 'Light' },
        { id: 'dark', label: 'Dark' },
        { id: 'system', label: 'System' }
    ];

    const viewModes = [
        { id: 'grid', label: 'Grid View' },
        { id: 'list', label: 'List View' },
        { id: 'details', label: 'Details View' }
    ];

    const fontSizes = [
        { id: 'small', label: 'Small' },
        { id: 'medium', label: 'Medium' },
        { id: 'large', label: 'Large' }
    ];

    const sortOptions = [
        { id: 'name', label: 'Name' },
        { id: 'size', label: 'Size' },
        { id: 'modified', label: 'Date Modified' },
        { id: 'type', label: 'Type' }
    ];

    const handleReset = async () => {
        if (!confirm('Are you sure you want to reset all settings to default? This cannot be undone.')) {
            return;
        }

        setIsResetting(true);
        try {
            await resetSettings();
            alert('Settings have been reset to default.');
        } catch (error) {
            console.error('Failed to reset settings:', error);
            alert('Failed to reset settings. Please try again.');
        } finally {
            setIsResetting(false);
        }
    };

    const renderAppearanceTab = () => (
        <div className="settings-tab-content">
            <div className="settings-section">
                <h3>Theme</h3>
                <div className="radio-group">
                    {themes.map(theme => (
                        <label key={theme.id} className="radio-option">
                            <input
                                type="radio"
                                name="theme"
                                value={theme.id}
                                checked={settings.theme === theme.id}
                                onChange={(e) => updateSetting('theme', e.target.value)}
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
                                name="defaultView"
                                value={mode.id}
                                checked={settings.defaultView === mode.id}
                                onChange={(e) => updateSetting('defaultView', e.target.value)}
                            />
                            <span>{mode.label}</span>
                        </label>
                    ))}
                </div>
            </div>

            <div className="settings-section">
                <h3>Font Size</h3>
                <select
                    value={settings.fontSize || 'medium'}
                    onChange={(e) => updateSetting('fontSize', e.target.value)}
                    className="settings-select"
                >
                    {fontSizes.map(size => (
                        <option key={size.id} value={size.id}>{size.label}</option>
                    ))}
                </select>
            </div>

            <div className="settings-section">
                <h3>Accent Color</h3>
                <div className="color-options">
                    {['blue', 'green', 'purple', 'orange', 'pink'].map(color => (
                        <button
                            key={color}
                            className={`color-option color-${color} ${settings.accentColor === color ? 'selected' : ''}`}
                            onClick={() => updateSetting('accentColor', color)}
                            aria-label={`${color} theme`}
                        />
                    ))}
                </div>
            </div>

            <div className="settings-section">
                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.showDetailsPanel || false}
                        onChange={(e) => updateSetting('showDetailsPanel', e.target.checked)}
                    />
                    <span>Show details panel by default</span>
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
                        checked={settings.confirmDelete !== false}
                        onChange={(e) => updateSetting('confirmDelete', e.target.checked)}
                    />
                    <span>Confirm before deleting files</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.showHiddenFiles || false}
                        onChange={(e) => updateSetting('showHiddenFiles', e.target.checked)}
                    />
                    <span>Show hidden files and folders</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.autoRefresh !== false}
                        onChange={(e) => updateSetting('autoRefresh', e.target.checked)}
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
                            value={settings.sortBy || 'name'}
                            onChange={(e) => updateSetting('sortBy', e.target.value)}
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
                            value={settings.sortDirection || 'asc'}
                            onChange={(e) => updateSetting('sortDirection', e.target.value)}
                            className="settings-select"
                        >
                            <option value="asc">Ascending</option>
                            <option value="desc">Descending</option>
                        </select>
                    </div>
                </div>
            </div>

            <div className="settings-section">
                <h3>Double-click Behavior</h3>
                <div className="radio-group">
                    <label className="radio-option">
                        <input
                            type="radio"
                            name="doubleClickBehavior"
                            value="open"
                            checked={(settings.doubleClickBehavior || 'open') === 'open'}
                            onChange={(e) => updateSetting('doubleClickBehavior', e.target.value)}
                        />
                        <span>Open files and folders</span>
                    </label>
                    <label className="radio-option">
                        <input
                            type="radio"
                            name="doubleClickBehavior"
                            value="select"
                            checked={settings.doubleClickBehavior === 'select'}
                            onChange={(e) => updateSetting('doubleClickBehavior', e.target.value)}
                        />
                        <span>Select files and folders</span>
                    </label>
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
                        checked={settings.searchCaseSensitive || false}
                        onChange={(e) => updateSetting('searchCaseSensitive', e.target.checked)}
                    />
                    <span>Case-sensitive search by default</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.searchIncludeHidden || false}
                        onChange={(e) => updateSetting('searchIncludeHidden', e.target.checked)}
                    />
                    <span>Include hidden files in search</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.searchWholeWords || false}
                        onChange={(e) => updateSetting('searchWholeWords', e.target.checked)}
                    />
                    <span>Match whole words only</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>Search Index</h3>
                <p>Indexing improves search performance but uses disk space.</p>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enableSearchIndex !== false}
                        onChange={(e) => updateSetting('enableSearchIndex', e.target.checked)}
                    />
                    <span>Enable search indexing</span>
                </label>

                <Button
                    variant="secondary"
                    onClick={async () => {
                        try {
                            await invoke('clear_search_engine');
                            alert('Search index has been cleared.');
                        } catch (error) {
                            console.error('Failed to clear search index:', error);
                            alert('Failed to clear search index.');
                        }
                    }}
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
                        value={settings.terminalHeight || 240}
                        onChange={(e) => updateSetting('terminalHeight', parseInt(e.target.value))}
                        className="settings-input"
                    />
                </div>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enableAnimations !== false}
                        onChange={(e) => updateSetting('enableAnimations', e.target.checked)}
                    />
                    <span>Enable animations and transitions</span>
                </label>

                <label className="checkbox-option">
                    <input
                        type="checkbox"
                        checked={settings.enableVirtualScrolling || false}
                        onChange={(e) => updateSetting('enableVirtualScrolling', e.target.checked)}
                    />
                    <span>Enable virtual scrolling for large directories</span>
                </label>
            </div>

            <div className="settings-section">
                <h3>File Associations</h3>
                <p>Configure which applications open specific file types.</p>
                <Button variant="secondary">
                    Manage File Associations
                </Button>
            </div>

            <div className="settings-section">
                <h3>Data Sources</h3>
                <p>Add additional locations to appear in the sidebar.</p>
                <Button variant="secondary">
                    Manage Data Sources
                </Button>
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
                <Button variant="primary" onClick={onClose}>
                    Close
                </Button>
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