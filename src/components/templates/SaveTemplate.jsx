import React, { useState } from 'react';
import { Modal, ModalFooter, Button } from '../common';
import { useFileSystem } from '../../providers/FileSystemProvider';

/**
 * Komponente zum Speichern eines Elements als Template
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {boolean} props.isOpen - Ob der Dialog geöffnet ist
 * @param {Function} props.onClose - Callback zum Schließen des Dialogs
 * @param {Object} props.item - Das zu speichernde Element (Datei oder Ordner)
 * @param {Function} [props.onSuccess] - Callback nach erfolgreicher Template-Erstellung
 */
const SaveTemplate = ({
                          isOpen,
                          onClose,
                          item,
                          onSuccess
                      }) => {
    const { saveAsTemplate } = useFileSystem();
    const [templateName, setTemplateName] = useState(item?.name || '');
    const [description, setDescription] = useState('');
    const [category, setCategory] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);

    // Zurücksetzen der Formularwerte beim Öffnen
    React.useEffect(() => {
        if (isOpen && item) {
            setTemplateName(item.name || '');
            setDescription('');
            setCategory('');
            setError(null);
        }
    }, [isOpen, item]);

    // Template speichern
    const handleSaveTemplate = async () => {
        if (!templateName.trim()) {
            setError('Bitte gib einen Namen für das Template ein.');
            return;
        }

        setIsLoading(true);
        setError(null);

        try {
            // [Backend Integration] - Template im Backend speichern
            // /* BACKEND_INTEGRATION: Template speichern */

            const templateData = {
                name: templateName,
                description,
                category,
                sourcePath: item.path,
                type: item.type,
                createdAt: new Date().toISOString()
            };

            const result = await saveAsTemplate(item.path, templateData);

            if (result.success) {
                if (onSuccess) {
                    onSuccess(result);
                }
                onClose();
            } else {
                setError('Fehler beim Speichern des Templates: ' + (result.error || 'Unbekannter Fehler'));
            }
        } catch (err) {
            console.error('Error saving template:', err);
            setError('Fehler beim Speichern des Templates: ' + err.message);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Als Template speichern"
            footer={
                <ModalFooter
                    onCancel={onClose}
                    onConfirm={handleSaveTemplate}
                    confirmText="Speichern"
                    isConfirmLoading={isLoading}
                    isConfirmDisabled={!templateName.trim()}
                />
            }
        >
            <div className="save-template-form">
                {error && (
                    <div className="form-error-message">{error}</div>
                )}

                <div className="form-group">
                    <label htmlFor="template-name" className="form-label">Name *</label>
                    <input
                        id="template-name"
                        type="text"
                        className="form-input"
                        value={templateName}
                        onChange={(e) => setTemplateName(e.target.value)}
                        placeholder="Template-Name"
                        required
                    />
                </div>

                <div className="form-group">
                    <label htmlFor="template-description" className="form-label">Beschreibung</label>
                    <textarea
                        id="template-description"
                        className="form-textarea"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="Beschreibe den Zweck und Inhalt des Templates"
                        rows={3}
                    />
                </div>

                <div className="form-group">
                    <label htmlFor="template-category" className="form-label">Kategorie</label>
                    <input
                        id="template-category"
                        type="text"
                        className="form-input"
                        value={category}
                        onChange={(e) => setCategory(e.target.value)}
                        placeholder="z.B. Projekte, Dokumente, etc."
                    />
                </div>

                <div className="form-info">
                    <div className="form-info-title">Template-Details:</div>
                    <div className="form-info-row">
                        <span className="form-info-label">Typ:</span>
                        <span className="form-info-value">{item?.type === 'directory' ? 'Ordner' : 'Datei'}</span>
                    </div>
                    <div className="form-info-row">
                        <span className="form-info-label">Pfad:</span>
                        <span className="form-info-value text-truncate" title={item?.path}>{item?.path}</span>
                    </div>
                </div>
            </div>
        </Modal>
    );
};

export default SaveTemplate;