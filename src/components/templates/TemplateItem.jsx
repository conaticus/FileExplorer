import React from 'react';
import { Tooltip, Button } from '../common';
import FileIcon from '../file-view/FileIcon';

/**
 * Komponente zur Darstellung eines einzelnen Templates
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {Object} props.template - Das Template-Objekt
 * @param {Function} [props.onApply] - Callback zum Anwenden des Templates
 * @param {Function} [props.onDelete] - Callback zum Löschen des Templates
 * @param {Function} [props.onClick] - Callback bei Klick auf das Template
 */
const TemplateItem = ({
                          template,
                          onApply,
                          onDelete,
                          onClick,
                      }) => {
    const fileExtension = template.name?.split('.').pop() || '';
    const templateType = template.type || 'file';
    const formattedDate = template.createdAt
        ? new Date(template.createdAt).toLocaleDateString()
        : 'Unbekannt';

    return (
        <div className="template-item" onClick={() => onClick?.(template)}>
            <div className="template-icon">
                <FileIcon fileType={templateType} extension={fileExtension} />
            </div>

            <div className="template-details">
                <div className="template-name" title={template.name}>
                    {template.name}
                </div>

                {template.description && (
                    <div className="template-description" title={template.description}>
                        {template.description}
                    </div>
                )}

                <div className="template-meta">
                    {template.category && (
                        <span className="template-category">{template.category}</span>
                    )}
                    <span className="template-date">Erstellt am: {formattedDate}</span>
                </div>
            </div>

            <div className="template-actions">
                <Tooltip content="Template anwenden">
                    <Button
                        variant="icon"
                        icon="M3 4h18M3 8h18M3 12h18M3 16h12M3 20h12" /* Anwenden-Icon */
                        onClick={(e) => {
                            e.stopPropagation();
                            onApply?.(template);
                        }}
                    />
                </Tooltip>

                <Tooltip content="Template löschen">
                    <Button
                        variant="icon"
                        icon="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /* Löschen-Icon */
                        onClick={(e) => {
                            e.stopPropagation();
                            onDelete?.(template);
                        }}
                    />
                </Tooltip>
            </div>
        </div>
    );
};

export default TemplateItem;