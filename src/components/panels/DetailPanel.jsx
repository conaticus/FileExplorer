import React, { useState, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';

const DetailPanel = ({ isOpen, selectedItems, onClose }) => {
    const { getItemProperties } = useFileSystem();
    const [itemDetails, setItemDetails] = useState(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const [activeTab, setActiveTab] = useState('properties');

    // Lade Details für das ausgewählte Element
    useEffect(() => {
        const loadDetails = async () => {
            if (!selectedItems || selectedItems.length === 0) {
                setItemDetails(null);
                return;
            }

            // Wenn mehrere Elemente ausgewählt sind, zeige zusammenfassende Informationen
            if (selectedItems.length > 1) {
                setItemDetails({
                    multipleSelection: true,
                    count: selectedItems.length,
                    paths: selectedItems
                });
                return;
            }

            setIsLoading(true);
            setError(null);

            try {
                // [Backend Integration] - Elementeigenschaften vom Backend abrufen
                // /* BACKEND_INTEGRATION: Elementeigenschaften abrufen */

                // Beispieldaten für ein ausgewähltes Element
                const path = selectedItems[0];
                const mockDetails = {
                    name: path.split(/[\\/]/).pop(),
                    path: path,
                    type: path.includes('.') ? 'file' : 'directory',
                    size: '1.2 MB',
                    created: new Date().toISOString(),
                    modified: new Date().toISOString(),
                    accessed: new Date().toISOString(),
                    attributes: {
                        readonly: false,
                        hidden: false,
                        system: false,
                    },
                    // Zusätzliche Metadaten für verschiedene Dateitypen
                    metadata: {
                        dimensions: path.match(/\.(jpg|jpeg|png|gif|bmp)$/i) ? '1920 x 1080' : null,
                        duration: path.match(/\.(mp3|wav|mp4|avi|mov)$/i) ? '3:45' : null,
                        artist: path.match(/\.(mp3|wav)$/i) ? 'Beispielkünstler' : null,
                        album: path.match(/\.(mp3|wav)$/i) ? 'Beispielalbum' : null,
                    }
                };

                setItemDetails(mockDetails);
            } catch (err) {
                console.error('Error loading item details:', err);
                setError('Fehler beim Laden der Details: ' + err.message);
            } finally {
                setIsLoading(false);
            }
        };

        loadDetails();
    }, [selectedItems, getItemProperties]);

    // Formatiere ein Datum
    const formatDate = (dateString) => {
        if (!dateString) return 'Unbekannt';
        const date = new Date(dateString);
        return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
    };

    // Berechne die Größe mit Einheit
    const formatSize = (size) => {
        if (!size) return 'Unbekannt';
        if (typeof size === 'string') return size;

        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let formattedSize = size;
        let unitIndex = 0;

        while (formattedSize >= 1024 && unitIndex < units.length - 1) {
            formattedSize /= 1024;
            unitIndex++;
        }

        return `${formattedSize.toFixed(2)} ${units[unitIndex]}`;
    };

    return (
        <div className={`detail-panel ${isOpen ? 'open' : 'collapsed'}`}>
            <div className="detail-panel-header">
                <h3 className="detail-panel-title">Details</h3>
                <button
                    className="detail-panel-close"
                    onClick={onClose}
                    aria-label="Details schließen"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        width="18"
                        height="18"
                    >
                        <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <div className="detail-panel-tabs">
                <button
                    className={`detail-panel-tab ${activeTab === 'properties' ? 'active' : ''}`}
                    onClick={() => setActiveTab('properties')}
                >
                    Eigenschaften
                </button>
                <button
                    className={`detail-panel-tab ${activeTab === 'preview' ? 'active' : ''}`}
                    onClick={() => setActiveTab('preview')}
                >
                    Vorschau
                </button>
            </div>

            <div className="detail-panel-content">
                {isLoading ? (
                    <div className="detail-panel-loading">
                        <span className="loading-spinner"></span>
                        <p>Lade Details...</p>
                    </div>
                ) : error ? (
                    <div className="detail-panel-error">
                        <p>{error}</p>
                    </div>
                ) : !itemDetails ? (
                    <div className="detail-panel-empty">
                        <p>Kein Element ausgewählt</p>
                    </div>
                ) : itemDetails.multipleSelection ? (
                    <div className="detail-panel-multiple">
                        <div className="detail-panel-section">
                            <h4 className="detail-panel-section-title">Mehrfachauswahl</h4>
                            <p>{itemDetails.count} Elemente ausgewählt</p>
                        </div>
                    </div>
                ) : activeTab === 'properties' ? (
                    <div className="detail-panel-properties">
                        {/* Grundinformationen */}
                        <div className="detail-panel-section">
                            <h4 className="detail-panel-section-title">Allgemein</h4>
                            <div className="detail-panel-property">
                                <span className="detail-panel-property-label">Name:</span>
                                <span className="detail-panel-property-value">{itemDetails.name}</span>
                            </div>
                            <div className="detail-panel-property">
                                <span className="detail-panel-property-label">Typ:</span>
                                <span className="detail-panel-property-value">
                  {itemDetails.type === 'directory' ? 'Ordner' : itemDetails.name.split('.').pop().toUpperCase() + '-Datei'}
                </span>
                            </div>
                            <div className="detail-panel-property">
                                <span className="detail-panel-property-label">Pfad:</span>
                                <span className="detail-panel-property-value text-truncate" title={itemDetails.path}>
                  {itemDetails.path}
                </span>
                            </div>
                            {itemDetails.type === 'file' && (
                                <div className="detail-panel-property">
                                    <span className="detail-panel-property-label">Größe:</span>
                                    <span className="detail-panel-property-value">{formatSize(itemDetails.size)}</span>
                                </div>
                            )}
                        </div>

                        {/* Zeitstempel */}
                        <div className="detail-panel-section">
                            <h4 className="detail-panel-section-title">Zeitstempel</h4>
                            <div className="detail-panel-property">
                                <span className="detail-panel-property-label">Erstellt:</span>
                                <span className="detail-panel-property-value">{formatDate(itemDetails.created)}</span>
                            </div>
                            <div className="detail-panel-property">
                                <span className="detail-panel-property-label">Geändert:</span>
                                <span className="detail-panel-property-value">{formatDate(itemDetails.modified)}</span>
                            </div>
                            <div className="detail-panel-property">
                                <span className="detail-panel-property-label">Zugriff:</span>
                                <span className="detail-panel-property-value">{formatDate(itemDetails.accessed)}</span>
                            </div>
                        </div>

                        {/* Attribute */}
                        <div className="detail-panel-section">
                            <h4 className="detail-panel-section-title">Attribute</h4>
                            <div className="detail-panel-attributes">
                                <label className="detail-panel-attribute">
                                    <input
                                        type="checkbox"
                                        checked={itemDetails.attributes?.readonly || false}
                                        onChange={() => {
                                            // [Backend Integration] - Attribute ändern
                                            // /* BACKEND_INTEGRATION: Attribut ändern */
                                            console.log('Read-only attribute changed');
                                        }}
                                    />
                                    <span>Schreibgeschützt</span>
                                </label>
                                <label className="detail-panel-attribute">
                                    <input
                                        type="checkbox"
                                        checked={itemDetails.attributes?.hidden || false}
                                        onChange={() => {
                                            // [Backend Integration] - Attribute ändern
                                            // /* BACKEND_INTEGRATION: Attribut ändern */
                                            console.log('Hidden attribute changed');
                                        }}
                                    />
                                    <span>Versteckt</span>
                                </label>
                                <label className="detail-panel-attribute">
                                    <input
                                        type="checkbox"
                                        checked={itemDetails.attributes?.system || false}
                                        onChange={() => {
                                            // [Backend Integration] - Attribute ändern
                                            // /* BACKEND_INTEGRATION: Attribut ändern */
                                            console.log('System attribute changed');
                                        }}
                                    />
                                    <span>System</span>
                                </label>
                            </div>
                        </div>

                        {/* Zusätzliche Metadaten für spezifische Dateitypen */}
                        {itemDetails.metadata && Object.values(itemDetails.metadata).some(value => value) && (
                            <div className="detail-panel-section">
                                <h4 className="detail-panel-section-title">Zusätzliche Informationen</h4>
                                {itemDetails.metadata.dimensions && (
                                    <div className="detail-panel-property">
                                        <span className="detail-panel-property-label">Abmessungen:</span>
                                        <span className="detail-panel-property-value">{itemDetails.metadata.dimensions}</span>
                                    </div>
                                )}
                                {itemDetails.metadata.duration && (
                                    <div className="detail-panel-property">
                                        <span className="detail-panel-property-label">Dauer:</span>
                                        <span className="detail-panel-property-value">{itemDetails.metadata.duration}</span>
                                    </div>
                                )}
                                {itemDetails.metadata.artist && (
                                    <div className="detail-panel-property">
                                        <span className="detail-panel-property-label">Künstler:</span>
                                        <span className="detail-panel-property-value">{itemDetails.metadata.artist}</span>
                                    </div>
                                )}
                                {itemDetails.metadata.album && (
                                    <div className="detail-panel-property">
                                        <span className="detail-panel-property-label">Album:</span>
                                        <span className="detail-panel-property-value">{itemDetails.metadata.album}</span>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                ) : (
                    <div className="detail-panel-preview">
                        {/* Vorschau für verschiedene Dateitypen */}
                        {itemDetails.type === 'file' && (
                            <div className="detail-panel-preview-content">
                                {itemDetails.name.match(/\.(jpg|jpeg|png|gif|bmp)$/i) ? (
                                    <div className="image-preview">
                                        <p className="preview-placeholder">Bildvorschau würde hier angezeigt</p>
                                        {/* [Backend Integration] - Bildvorschau laden */}
                                        {/* BACKEND_INTEGRATION: Bildvorschau laden */}
                                            </div>
                                            ) : itemDetails.name.match(/\.(txt|md|json|xml|html|css|js)$/i) ? (
                                            <div className="text-preview">
                                            <p className="preview-placeholder">Textvorschau würde hier angezeigt</p>
                                        {/* [Backend Integration] - Textvorschau laden */}
                                        {/* BACKEND_INTEGRATION: Textvorschau laden */}
                                            </div>
                                            ) : itemDetails.name.match(/\.(mp3|wav)$/i) ? (
                                            <div className="audio-preview">
                                            <p className="preview-placeholder">Audiovorschau würde hier angezeigt</p>
                                        {/* [Backend Integration] - Audiovorschau laden */}
                                        {/* BACKEND_INTEGRATION: Audiovorschau laden */}
                                            </div>
                                            ) : itemDetails.name.match(/\.(mp4|webm)$/i) ? (
                                            <div className="video-preview">
                                            <p className="preview-placeholder">Videovorschau würde hier angezeigt</p>
                                        {/* [Backend Integration] - Videovorschau laden */}
                                        {/* BACKEND_INTEGRATION: Videovorschau laden */}
                                            </div>
                                            ) : (
                                            <div className="no-preview">
                                            <p>Keine Vorschau verfügbar für diesen Dateityp.</p>
                                            </div>
                                            )}
                                    </div>
                                )}

                                {itemDetails.type === 'directory' && (
                                    <div className="directory-preview">
                                        <p>Ordnervorschau: {itemDetails.name}</p>
                                        {/* [Backend Integration] - Ordnerinhalt für Vorschau laden */}
                                        {/* BACKEND_INTEGRATION: Ordnerinhalt laden */}
                                            </div>
                                            )}
                                    </div>
                                )}
                            </div>
                            </div>
                            );
                        };

export default DetailPanel;