import React, { useState, useEffect } from 'react';
import { useTheme } from '../../providers/ThemeProvider';
import { Modal } from '../common';
import './Settings.css';

// Create a component for the settings button
export const SettingsButton = ({ onClick }) => {
    return (
        <button
            className="settings-trigger-button"
            onClick={onClick}
            title="Settings"
        >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z" />
            </svg>
        </button>
    );
};

// Icons
const ColorIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="10" />
        <path d="M12 18a6 6 0 1 0 0-12 4 4 0 0 1 0 8" />
    </svg>
);

const LayoutIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <path d="M3 9h18" />
        <path d="M9 21V9" />
    </svg>
);

const TextIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M4 7V4h16v3" />
        <path d="M9 20h6" />
        <path d="M12 4v16" />
    </svg>
);

const BehaviorIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 2v4" />
        <path d="M12 18v4" />
        <path d="M4.93 4.93l2.83 2.83" />
        <path d="M16.24 16.24l2.83 2.83" />
        <path d="M2 12h4" />
        <path d="M18 12h4" />
        <path d="M4.93 19.07l2.83-2.83" />
        <path d="M16.24 7.76l2.83-2.83" />
    </svg>
);

const AboutIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="10" />
        <path d="M12 16v-4" />
        <path d="M12 8h.01" />
    </svg>
);

const Settings = ({ isOpen, onClose }) => {
    const {
        theme,
        activeTheme,
        setTheme,
        themes,
        themeSettings,
        updateThemeSettings,
        themePalettes
    } = useTheme();

    const [activeTab, setActiveTab] = useState('appearance');
    const [previewTheme, setPreviewTheme] = useState(activeTheme);
    const [selectedFont, setSelectedFont] = useState(themeSettings.fontFamily);
    const [fontSize, setFontSize] = useState(themeSettings.fontSize);
    const [defaultView, setDefaultView] = useState(themeSettings.defaultView);
    const [iconSize, setIconSize] = useState(themeSettings.iconSize);
    const [accentColor, setAccentColor] = useState(themeSettings.accentColor);
    const [enableGlassEffect, setEnableGlassEffect] = useState(themeSettings.enableGlassEffect || false);
    const [enableAnimations, setEnableAnimations] = useState(themeSettings.enableAnimations || true);
    const [density, setDensity] = useState(themeSettings.density || 'normal');
    const [borderRadius, setBorderRadius] = useState(themeSettings.borderRadius || 'medium');

    // Reset to defaults if panel reopened
    useEffect(() => {
        if (isOpen) {
            setPreviewTheme(activeTheme);
            setSelectedFont(themeSettings.fontFamily);
            setFontSize(themeSettings.fontSize);
            setDefaultView(themeSettings.defaultView);
            setIconSize(themeSettings.iconSize);
            setAccentColor(themeSettings.accentColor);
            setEnableGlassEffect(themeSettings.enableGlassEffect || false);
            setEnableAnimations(themeSettings.enableAnimations || true);
            setDensity(themeSettings.density || 'normal');
            setBorderRadius(themeSettings.borderRadius || 'medium');
        }
    }, [isOpen, activeTheme, themeSettings]);

    // Apply changes
    const applyChanges = () => {
        setTheme(previewTheme);
        updateThemeSettings({
            fontFamily: selectedFont,
            fontSize: fontSize,
            defaultView: defaultView,
            iconSize: iconSize,
            accentColor: accentColor,
            enableGlassEffect: enableGlassEffect,
            enableAnimations: enableAnimations,
            density: density,
            borderRadius: borderRadius
        });
        onClose();
    };

    // Cancel changes
    const cancelChanges = () => {
        onClose();
    };

    // Available font families
    const fontFamilies = [
        { name: 'System UI', value: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif' },
        { name: 'Inter', value: '"Inter", sans-serif' },
        { name: 'Roboto', value: '"Roboto", sans-serif' },
        { name: 'Open Sans', value: '"Open Sans", sans-serif' },
        { name: 'SF Pro', value: '"SF Pro Text", "SF Pro", "Helvetica Neue", Helvetica, Arial, sans-serif' },
        { name: 'Montserrat', value: '"Montserrat", sans-serif' },
        { name: 'Consolas', value: 'Consolas, "Courier New", monospace' }
    ];

    // Preview panel
    const ThemePreview = ({ themeName, useGlass }) => {
        const palette = themePalettes[themeName] || themePalettes.light;
        const isActive = previewTheme === themeName;

        const previewStyle = {
            backgroundColor: palette.surface,
            color: palette.text,
            border: isActive ? `2px solid ${palette.primary}` : `1px solid ${palette.border}`,
            ...(useGlass && {
                backgroundColor: 'rgba(255, 255, 255, 0.15)',
                backdropFilter: 'blur(12px)',
                boxShadow: '0 4px 15px rgba(0, 0, 0, 0.1)'
            })
        };

        return (
            <div
                className={`theme-preview ${isActive ? 'active' : ''}`}
                style={previewStyle}
                onClick={() => setPreviewTheme(themeName)}
            >
                <div className="theme-preview-content">
                    <div className="theme-preview-header" style={{ backgroundColor: palette.primary, color: palette.textOnPrimary }}>
                        {themeName.charAt(0).toUpperCase() + themeName.slice(1)}
                    </div>
                    <div className="theme-preview-body">
                        <div className="theme-preview-item" style={{ backgroundColor: palette.background }}></div>
                        <div className="theme-preview-item" style={{ backgroundColor: palette.surface }}></div>
                        <div className="theme-preview-item" style={{ backgroundColor: palette.primary }}></div>
                    </div>
                </div>
            </div>
        );
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={cancelChanges}
            title="Settings"
            size="large"
            className="settings-modal"
            showCloseButton={true}
        >
            <div className="settings-container">
                <div className="settings-sidebar">
                    <nav className="settings-nav">
                        <button
                            className={`settings-nav-item ${activeTab === 'appearance' ? 'active' : ''}`}
                            onClick={() => setActiveTab('appearance')}
                        >
                            <ColorIcon /> Appearance
                        </button>
                        <button
                            className={`settings-nav-item ${activeTab === 'layout' ? 'active' : ''}`}
                            onClick={() => setActiveTab('layout')}
                        >
                            <LayoutIcon /> Layout
                        </button>
                        <button
                            className={`settings-nav-item ${activeTab === 'text' ? 'active' : ''}`}
                            onClick={() => setActiveTab('text')}
                        >
                            <TextIcon /> Text
                        </button>
                        <button
                            className={`settings-nav-item ${activeTab === 'behavior' ? 'active' : ''}`}
                            onClick={() => setActiveTab('behavior')}
                        >
                            <BehaviorIcon /> Behavior
                        </button>
                        <button
                            className={`settings-nav-item ${activeTab === 'about' ? 'active' : ''}`}
                            onClick={() => setActiveTab('about')}
                        >
                            <AboutIcon /> About
                        </button>
                    </nav>
                </div>

                <div className="settings-content">
                    {activeTab === 'appearance' && (
                        <div className="settings-section">
                            <h2>Themes</h2>
                            <div className="theme-previews">
                                {Object.keys(themePalettes).map(themeName => (
                                    <ThemePreview
                                        key={themeName}
                                        themeName={themeName}
                                        useGlass={themeName.includes('glass') || (themeName === previewTheme && enableGlassEffect)}
                                    />
                                ))}
                            </div>

                            <div className="settings-option">
                                <label>
                                    <input
                                        type="checkbox"
                                        checked={enableGlassEffect}
                                        onChange={(e) => setEnableGlassEffect(e.target.checked)}
                                    />
                                    Enable glass effect
                                </label>
                                <small>Adds a modern, translucent look to UI elements (works best with dark themes)</small>
                            </div>

                            <h2>Accent Color</h2>
                            <div className="color-picker">
                                {['#0078d4', '#0078D7', '#7B61FF', '#00CC6A', '#E81123', '#F7630C', '#FACF37', '#FF7CBB'].map(color => (
                                    <button
                                        key={color}
                                        className={`color-option ${accentColor === color ? 'active' : ''}`}
                                        style={{ backgroundColor: color }}
                                        onClick={() => setAccentColor(color)}
                                        aria-label={`Select color ${color}`}
                                    />
                                ))}
                            </div>

                            <h2>UI Density</h2>
                            <div className="settings-option-group">
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="density"
                                        value="compact"
                                        checked={density === 'compact'}
                                        onChange={() => setDensity('compact')}
                                    />
                                    Compact
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="density"
                                        value="normal"
                                        checked={density === 'normal'}
                                        onChange={() => setDensity('normal')}
                                    />
                                    Normal
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="density"
                                        value="comfortable"
                                        checked={density === 'comfortable'}
                                        onChange={() => setDensity('comfortable')}
                                    />
                                    Comfortable
                                </label>
                            </div>

                            <h2>Corner Roundness</h2>
                            <div className="settings-option-group">
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="borderRadius"
                                        value="none"
                                        checked={borderRadius === 'none'}
                                        onChange={() => setBorderRadius('none')}
                                    />
                                    None
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="borderRadius"
                                        value="small"
                                        checked={borderRadius === 'small'}
                                        onChange={() => setBorderRadius('small')}
                                    />
                                    Small
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="borderRadius"
                                        value="medium"
                                        checked={borderRadius === 'medium'}
                                        onChange={() => setBorderRadius('medium')}
                                    />
                                    Medium
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="borderRadius"
                                        value="large"
                                        checked={borderRadius === 'large'}
                                        onChange={() => setBorderRadius('large')}
                                    />
                                    Large
                                </label>
                            </div>

                            <div className="settings-option">
                                <label>
                                    <input
                                        type="checkbox"
                                        checked={enableAnimations}
                                        onChange={(e) => setEnableAnimations(e.target.checked)}
                                    />
                                    Enable animations
                                </label>
                                <small>Smooth transitions between UI states</small>
                            </div>
                        </div>
                    )}

                    {activeTab === 'layout' && (
                        <div className="settings-section">
                            <h2>Default View</h2>
                            <div className="settings-option-group">
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="view"
                                        value="list"
                                        checked={defaultView === 'list'}
                                        onChange={() => setDefaultView('list')}
                                    />
                                    List
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="view"
                                        value="grid"
                                        checked={defaultView === 'grid'}
                                        onChange={() => setDefaultView('grid')}
                                    />
                                    Grid
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="view"
                                        value="details"
                                        checked={defaultView === 'details'}
                                        onChange={() => setDefaultView('details')}
                                    />
                                    Details
                                </label>
                            </div>

                            <h2>Icon Size</h2>
                            <div className="settings-option-group">
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="iconSize"
                                        value="small"
                                        checked={iconSize === 'small'}
                                        onChange={() => setIconSize('small')}
                                    />
                                    Small
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="iconSize"
                                        value="medium"
                                        checked={iconSize === 'medium'}
                                        onChange={() => setIconSize('medium')}
                                    />
                                    Medium
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="iconSize"
                                        value="large"
                                        checked={iconSize === 'large'}
                                        onChange={() => setIconSize('large')}
                                    />
                                    Large
                                </label>
                            </div>
                        </div>
                    )}

                    {activeTab === 'text' && (
                        <div className="settings-section">
                            <h2>Font</h2>
                            <div className="settings-option">
                                <label htmlFor="font-select">Font Family</label>
                                <select
                                    id="font-select"
                                    value={selectedFont}
                                    onChange={(e) => setSelectedFont(e.target.value)}
                                    className="settings-select"
                                >
                                    {fontFamilies.map(font => (
                                        <option key={font.name} value={font.value}>{font.name}</option>
                                    ))}
                                </select>
                            </div>

                            <div className="font-preview" style={{ fontFamily: selectedFont }}>
                                <p>The quick brown fox jumps over the lazy dog</p>
                                <p>1234567890</p>
                                <p style={{ fontWeight: 'bold' }}>Bold text example</p>
                            </div>

                            <h2>Font Size</h2>
                            <div className="settings-option-group">
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="fontSize"
                                        value="small"
                                        checked={fontSize === 'small'}
                                        onChange={() => setFontSize('small')}
                                    />
                                    Small
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="fontSize"
                                        value="medium"
                                        checked={fontSize === 'medium'}
                                        onChange={() => setFontSize('medium')}
                                    />
                                    Medium
                                </label>
                                <label className="radio-label">
                                    <input
                                        type="radio"
                                        name="fontSize"
                                        value="large"
                                        checked={fontSize === 'large'}
                                        onChange={() => setFontSize('large')}
                                    />
                                    Large
                                </label>
                            </div>
                        </div>
                    )}

                    {activeTab === 'behavior' && (
                        <div className="settings-section">
                            <h2>File Operations</h2>
                            <div className="settings-option">
                                <label>
                                    <input
                                        type="checkbox"
                                        checked={themeSettings.confirmDeletion || true}
                                        onChange={(e) => updateThemeSettings({ ...themeSettings, confirmDeletion: e.target.checked })}
                                    />
                                    Confirm before deleting files
                                </label>
                            </div>

                            <div className="settings-option">
                                <label>
                                    <input
                                        type="checkbox"
                                        checked={themeSettings.showHiddenFiles || false}
                                        onChange={(e) => updateThemeSettings({ ...themeSettings, showHiddenFiles: e.target.checked })}
                                    />
                                    Show hidden files
                                </label>
                            </div>

                            <h2>File Previews</h2>
                            <div className="settings-option">
                                <label>
                                    <input
                                        type="checkbox"
                                        checked={themeSettings.showThumbnails || true}
                                        onChange={(e) => updateThemeSettings({ ...themeSettings, showThumbnails: e.target.checked })}
                                    />
                                    Show file thumbnails
                                </label>
                            </div>
                        </div>
                    )}

                    {activeTab === 'about' && (
                        <div className="settings-section about-section">
                            <div className="app-info">
                                <h1>Fast File Explorer</h1>
                                <p className="app-version">Version 0.2.0</p>
                                <p>A modern, fast file explorer built with Rust, Tauri, and React.</p>

                                <h2>Features</h2>
                                <ul>
                                    <li>Lightning-fast file searching</li>
                                    <li>Modern, customizable UI</li>
                                    <li>Multiple view modes</li>
                                    <li>Advanced file operations</li>
                                    <li>Template system</li>
                                </ul>

                                <div className="copyright">
                                    <p>&copy; 2025 Fast File Explorer</p>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            <div className="settings-footer">
                <button className="settings-button cancel" onClick={cancelChanges}>
                    Cancel
                </button>
                <button className="settings-button primary" onClick={applyChanges}>
                    Apply Changes
                </button>
            </div>
        </Modal>
    );
};

export default Settings;