import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import Button from '../common/Button';
import IconButton from '../common/IconButton';
import Modal from '../common/Modal';
import TemplateItem from './TemplateItem';
import EmptyState from '../explorer/EmptyState';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { getTemplatePaths, useTemplate, removeTemplate } from '../../utils/fileOperations';
import './templates.css';

const TemplateList = ({ onClose }) => {
    const [templates, setTemplates] = useState([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);
    const [selectedTemplate, setSelectedTemplate] = useState(null);
    const [isUseModalOpen, setIsUseModalOpen] = useState(false);
    const [destinationPath, setDestinationPath] = useState('');
    const { currentPath } = useHistory();
    const { loadDirectory } = useFileSystem();

    // Load templates on mount
    useEffect(() => {
        const loadTemplates = async () => {
            setIsLoading(true);
            setError(null);

            try {
                const templatePaths = await getTemplatePaths();
                setTemplates(templatePaths);
            } catch (err) {
                console.error('Failed to load templates:', err);
                setError('Failed to load templates. Please try again.');

                // Mock data for development
                setTemplates([
                    {
                        name: 'Project Template',
                        path: '/templates/project-template',
                        type: 'folder',
                        size: 2048,
                        createdAt: '2023-04-15'
                    },
                    {
                        name: 'Document Template.docx',
                        path: '/templates/document-template.docx',
                        type: 'file',
                        size: 1024,
                        createdAt: '2023-03-20'
                    },
                    {
                        name: 'Web Project',
                        path: '/templates/web-project',
                        type: 'folder',
                        size: 4096,
                        createdAt: '2023-05-10'
                    }
                ]);
            } finally {
                setIsLoading(false);
            }
        };

        loadTemplates();
    }, []);

    // Open modal to use template
    const handleUseTemplate = (template) => {
        setSelectedTemplate(template);
        setDestinationPath(currentPath || '');
        setIsUseModalOpen(true);
    };

    // Apply template
    const applyTemplate = async () => {
        if (!selectedTemplate || !destinationPath) return;

        try {
            await useTemplate(selectedTemplate.path, destinationPath);

            // Reload the directory to show the new content
            await loadDirectory(currentPath);

            // Close the modal
            setIsUseModalOpen(false);

            // Optionally close the template list
            if (onClose) onClose();
        } catch (err) {
            console.error('Failed to apply template:', err);
            setError('Failed to apply template. Please try again.');
        }
    };

    // Remove template
    const handleRemoveTemplate = async (template) => {
        try {
            await removeTemplate(template.path);

            // Update the template list
            setTemplates(prev => prev.filter(t => t.path !== template.path));
        } catch (err) {
            console.error('Failed to remove template:', err);
            setError('Failed to remove template. Please try again.');
        }
    };

    // Add new template
    const handleAddTemplate = async () => {
        // This would normally open a file picker dialog
        // For simplicity, we'll just log a message
        console.log('Add new template...');
    };

    return (
        <div className="template-list-container">
            <div className="template-list-header">
                <h2>Templates</h2>
                <div className="template-list-actions">
                    <Button
                        variant="primary"
                        onClick={handleAddTemplate}
                    >
                        Add Template
                    </Button>
                </div>
            </div>

            <div className="template-list-content">
                {isLoading ? (
                    <div className="template-list-loading">
                        <div className="spinner"></div>
                        <p>Loading templates...</p>
                    </div>
                ) : error ? (
                    <div className="template-list-error">
                        <div className="alert alert-error">
                            <div className="alert-content">
                                <p>{error}</p>
                            </div>
                        </div>
                    </div>
                ) : templates.length === 0 ? (
                    <EmptyState
                        type="no-templates"
                        title="No Templates"
                        message="You haven't saved any templates yet. Templates help you create files and folders with predefined structures."
                    />
                ) : (
                    <div className="template-grid">
                        {templates.map((template, index) => (
                            <TemplateItem
                                key={template.path || index}
                                template={template}
                                onUse={() => handleUseTemplate(template)}
                                onRemove={() => handleRemoveTemplate(template)}
                            />
                        ))}
                    </div>
                )}
            </div>

            {/* Modal for using template */}
            <Modal
                isOpen={isUseModalOpen}
                onClose={() => setIsUseModalOpen(false)}
                title="Use Template"
                size="sm"
                footer={
                    <>
                        <Button
                            variant="ghost"
                            onClick={() => setIsUseModalOpen(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            variant="primary"
                            onClick={applyTemplate}
                        >
                            Apply Template
                        </Button>
                    </>
                }
            >
                <div className="template-use-form">
                    <div className="form-group">
                        <label htmlFor="template-name">Template</label>
                        <input
                            type="text"
                            id="template-name"
                            className="input"
                            value={selectedTemplate?.name || ''}
                            disabled
                        />
                    </div>

                    <div className="form-group">
                        <label htmlFor="destination-path">Destination</label>
                        <input
                            type="text"
                            id="destination-path"
                            className="input"
                            value={destinationPath}
                            onChange={(e) => setDestinationPath(e.target.value)}
                            placeholder="Enter destination path"
                        />
                        <div className="input-hint">
                            This is where the template will be applied.
                        </div>
                    </div>
                </div>
            </Modal>
        </div>
    );
};

export default TemplateList;