import React, { useState, useEffect } from 'react';
import { Button, Modal, ModalFooter } from '../common';
import TemplateItem from './TemplateItem';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useAppState } from '../../providers/AppStateProvider';

/**
 * Komponente zur Anzeige und Verwaltung von Templates
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {string} [props.currentPath] - Aktueller Pfad für das Anwenden von Templates
 */
const TemplateList = ({ currentPath }) => {
    const { getTemplates, applyTemplate } = useFileSystem();
    const { actions } = useAppState();

    const [templates, setTemplates] = useState([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);
    const [categories, setCategories] = useState([]);
    const [selectedCategory, setSelectedCategory] = useState('all');
    const [searchTerm, setSearchTerm] = useState('');

    // Template, das angewendet werden soll
    const [templateToApply, setTemplateToApply] = useState(null);
    const [isApplyModalOpen, setIsApplyModalOpen] = useState(false);
    const [applyPath, setApplyPath] = useState('');
    const [isApplying, setIsApplying] = useState(false);

    // Template, das gelöscht werden soll
    const [templateToDelete, setTemplateToDelete] = useState(null);
    const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
    const [isDeleting, setIsDeleting] = useState(false);

    // Lade Templates
    useEffect(() => {
        const loadTemplates = async () => {
            setIsLoading(true);
            setError(null);

            try {
                // [Backend Integration] - Templates vom Backend abrufen
                // /* BACKEND_INTEGRATION: Templates laden */

                // Beispieldaten
                const mockTemplates = [
                    {
                        id: '1',
                        name: 'Projektordner',
                        description: 'Standardordnerstruktur für neue Projekte',
                        category: 'Projekte',
                        type: 'directory',
                        createdAt: '2023-01-15T10:30:00Z',
                        path: '/templates/projektordner'
                    },
                    {
                        id: '2',
                        name: 'Dokumentvorlage.docx',
                        description: 'Vorlage für Projektdokumentation',
                        category: 'Dokumente',
                        type: 'file',
                        createdAt: '2023-02-20T14:45:00Z',
                        path: '/templates/dokumentvorlage.docx'
                    },
                    {
                        id: '3',
                        name: 'Zeiterfassung.xlsx',
                        description: 'Excel-Vorlage für Zeiterfassung',
                        category: 'Tabellen',
                        type: 'file',
                        createdAt: '2023-03-10T09:15:00Z',
                        path: '/templates/zeiterfassung.xlsx'
                    },
                    {
                        id: '4',
                        name: 'Präsentation.pptx',
                        description: 'Standardvorlage für Präsentationen',
                        category: 'Dokumente',
                        type: 'file',
                        createdAt: '2023-04-05T11:20:00Z',
                        path: '/templates/präsentation.pptx'
                    },
                ];

                setTemplates(mockTemplates);

                // Extrahiere eindeutige Kategorien
                const uniqueCategories = [...new Set(mockTemplates.map(template => template.category))].filter(Boolean);
                setCategories(uniqueCategories);
            } catch (err) {
                console.error('Error loading templates:', err);
                setError('Fehler beim Laden der Templates: ' + err.message);
            } finally {
                setIsLoading(false);
            }
        };

        loadTemplates();
    }, [getTemplates]);

    // Öffne den Anwenden-Dialog
    const handleApplyTemplate = (template) => {
        setTemplateToApply(template);
        setApplyPath(currentPath || '');
        setIsApplyModalOpen(true);
    };

    // Wende das Template an
    const confirmApplyTemplate = async () => {
        if (!templateToApply || !applyPath) return;

        setIsApplying(true);

        try {
            // [Backend Integration] - Template im Backend anwenden
            // /* BACKEND_INTEGRATION: Template anwenden */

            const result = await applyTemplate(templateToApply.path, applyPath);

            if (result.success) {
                // Schließe den Dialog und aktualisiere die Ansicht
                setIsApplyModalOpen(false);
                setTemplateToApply(null);

                // Navigiere zum Zielpfad
                actions.setCurrentPath(applyPath);
            } else {
                setError('Fehler beim Anwenden des Templates: ' + (result.error || 'Unbekannter Fehler'));
            }
        } catch (err) {
            console.error('Error applying template:', err);
            setError('Fehler beim Anwenden des Templates: ' + err.message);
        } finally {
            setIsApplying(false);
        }
    };

    // Öffne den Löschen-Dialog
    const handleDeleteTemplate = (template) => {
        setTemplateToDelete(template);
        setIsDeleteModalOpen(true);
    };

    // Lösche das Template
    const confirmDeleteTemplate = async () => {
        if (!templateToDelete) return;

        setIsDeleting(true);

        try {
            // [Backend Integration] - Template im Backend löschen
            // /* BACKEND_INTEGRATION: Template löschen */

            // Simuliere Erfolg
            const success = true;

            if (success) {
                // Entferne das Template aus der Liste
                setTemplates(prev => prev.filter(template => template.id !== templateToDelete.id));

                // Schließe den Dialog
                setIsDeleteModalOpen(false);
                setTemplateToDelete(null);
            } else {
                setError('Fehler beim Löschen des Templates');
            }
        } catch (err) {
            console.error('Error deleting template:', err);
            setError('Fehler beim Löschen des Templates: ' + err.message);
        } finally {
            setIsDeleting(false);
        }
    };

    // Filtere Templates basierend auf Kategorie und Suchbegriff
    const filteredTemplates = templates.filter(template => {
        const matchesCategory = selectedCategory === 'all' || template.category === selectedCategory;
        const matchesSearch = !searchTerm ||
            template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            (template.description && template.description.toLowerCase().includes(searchTerm.toLowerCase()));

        return matchesCategory && matchesSearch;
    });

    return (
        <div className="template-list-container">
            <div className="template-list-header">
                <div className="template-list-title">
                    <h2>Templates</h2>
                    <span className="template-count">{templates.length} Template(s)</span>
                </div>

                <div className="template-list-actions">
                    <div className="template-search">
                        <input
                            type="text"
                            className="template-search-input"
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            placeholder="Templates durchsuchen..."
                        />
                    </div>

                    <div className="template-filter">
                        <select
                            className="template-category-select"
                            value={selectedCategory}
                            onChange={(e) => setSelectedCategory(e.target.value)}
                        >
                            <option value="all">Alle Kategorien</option>
                            {categories.map(category => (
                                <option key={category} value={category}>{category}</option>
                            ))}
                        </select>
                    </div>
                </div>
            </div>

            {error && (
                <div className="template-list-error">
                    {error}
                </div>
            )}

            <div className="template-list">
                {isLoading ? (
                    <div className="template-list-loading">
                        <div className="loading-spinner"></div>
                        <p>Templates werden geladen...</p>
                    </div>
                ) : filteredTemplates.length === 0 ? (
                    <div className="template-list-empty">
                        {searchTerm || selectedCategory !== 'all' ? (
                            <p>Keine Templates gefunden, die den Filterkriterien entsprechen.</p>
                        ) : (
                            <p>Keine Templates vorhanden. Speichere Dateien oder Ordner als Templates, um sie hier zu sehen.</p>
                        )}
                    </div>
                ) : (
                    filteredTemplates.map(template => (
                        <TemplateItem
                            key={template.id}
                            template={template}
                            onApply={handleApplyTemplate}
                            onDelete={handleDeleteTemplate}
                            onClick={() => handleApplyTemplate(template)}
                        />
                    ))
                )}
            </div>

            {/* Modal zum Anwenden eines Templates */}
            <Modal
                isOpen={isApplyModalOpen}
                onClose={() => setIsApplyModalOpen(false)}
                title="Template anwenden"
                footer={
                    <ModalFooter
                        onCancel={() => setIsApplyModalOpen(false)}
                        onConfirm={confirmApplyTemplate}
                        confirmText="Anwenden"
                        isConfirmLoading={isApplying}
                        isConfirmDisabled={!applyPath.trim()}
                    />
                }
            >
                <div className="apply-template-form">
                    <p>
                        Du bist dabei, das Template <strong>{templateToApply?.name}</strong> anzuwenden.
                        Wähle den Zielpfad, in dem das Template erstellt werden soll:
                    </p>

                    <div className="form-group">
                        <label htmlFor="apply-path" className="form-label">Zielpfad *</label>
                        <input
                            id="apply-path"
                            type="text"
                            className="form-input"
                            value={applyPath}
                            onChange={(e) => setApplyPath(e.target.value)}
                            placeholder="Pfad eingeben"
                            required
                        />
                    </div>

                    <div className="apply-template-info">
                        <div className="form-info-title">Template-Info:</div>
                        <div className="form-info-row">
                            <span className="form-info-label">Typ:</span>
                            <span className="form-info-value">
                {templateToApply?.type === 'directory' ? 'Ordner' : 'Datei'}
              </span>
                        </div>
                        {templateToApply?.description && (
                            <div className="form-info-row">
                                <span className="form-info-label">Beschreibung:</span>
                                <span className="form-info-value">{templateToApply.description}</span>
                            </div>
                        )}
                    </div>
                </div>
            </Modal>

            {/* Modal zum Löschen eines Templates */}
            <Modal
                isOpen={isDeleteModalOpen}
                onClose={() => setIsDeleteModalOpen(false)}
                title="Template löschen"
                footer={
                    <ModalFooter
                        onCancel={() => setIsDeleteModalOpen(false)}
                        onConfirm={confirmDeleteTemplate}
                        confirmText="Löschen"
                        confirmVariant="danger"
                        isConfirmLoading={isDeleting}
                    />
                }
            >
                <div className="delete-template-confirmation">
                    <p>
                        Bist du sicher, dass du das Template <strong>{templateToDelete?.name}</strong> löschen möchtest?
                    </p>
                    <p className="delete-warning">
                        Diese Aktion kann nicht rückgängig gemacht werden.
                    </p>
                </div>
            </Modal>
        </div>
    );
};

export default TemplateList;