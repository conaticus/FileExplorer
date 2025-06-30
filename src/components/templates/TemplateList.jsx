import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Button from '../common/Button';
import IconButton from '../common/IconButton';
import Modal from '../common/Modal';
import TemplateItem from './TemplateItem';
import EmptyState from '../explorer/EmptyState';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { getTemplatePaths, useTemplate, removeTemplate, addTemplate } from '../../utils/fileOperations';
import { showError, showSuccess } from '../../utils/NotificationSystem';
import './templates.css';

/**
 * TemplateList component - Displays and manages file and folder templates
 * Allows users to save, apply, and remove templates for reuse
 *
 * @param {Object} props - Component props
 * @param {Function} props.onClose - Callback function when the template list is closed
 * @returns {React.ReactElement} TemplateList component
 */
const TemplateList = ({ onClose }) => {
    const [templates, setTemplates] = useState([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);
    const [selectedTemplate, setSelectedTemplate] = useState(null);
    const [isUseModalOpen, setIsUseModalOpen] = useState(false);
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [destinationPath, setDestinationPath] = useState('');
    const [newTemplatePath, setNewTemplatePath] = useState('');
    const { currentPath } = useHistory();
    const { loadDirectory } = useFileSystem();
    const addTemplateInputRef = useRef(null);

    /**
     * Load templates on component mount
     * Also sets up event listener for template updates
     */
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

        // Listen for template updates from context menu
        const handleTemplatesUpdated = () => {
            loadTemplates();
        };

        window.addEventListener('templates-updated', handleTemplatesUpdated);

        return () => {
            window.removeEventListener('templates-updated', handleTemplatesUpdated);
        };
    }, []);

    /**
     * Opens the modal to use/apply a template
     * @param {Object} template - The template to use
     */
    const handleUseTemplate = (template) => {
        setSelectedTemplate(template);
        setDestinationPath(currentPath || '');
        setIsUseModalOpen(true);
    };

    /**
     * Applies the selected template to the specified destination path
     * @async
     */
    const applyTemplate = async () => {
        if (!selectedTemplate || !destinationPath) return;

        try {
            await useTemplate(selectedTemplate.path, destinationPath);

            // Reload the directory to show the new content
            await loadDirectory(currentPath);

            // Close the modal
            setIsUseModalOpen(false);

            // Show success message
            showSuccess(`Template "${selectedTemplate.name}" applied successfully!`);
        } catch (err) {
            console.error('Failed to apply template:', err);
            setError('Failed to apply template. Please try again.');
        }
    };

    /**
     * Removes a template from the saved templates
     * @param {Object} template - The template to remove
     * @async
     */
    const handleRemoveTemplate = async (template) => {
        try {
            await removeTemplate(template.path);

            // Update the template list
            setTemplates(prev => prev.filter(t => t.path !== template.path));
            showSuccess(`Template "${template.name}" removed successfully.`);
        } catch (err) {
            console.error('Failed to remove template:', err);
            showError('Failed to remove template. Please try again.');
        }
    };

    /**
     * Opens the modal to add a new template
     */
    const handleAddTemplate = () => {
        setNewTemplatePath('');
        setIsAddModalOpen(true);

        // Focus input after modal opens
        setTimeout(() => {
            if (addTemplateInputRef.current) {
                addTemplateInputRef.current.focus();
            }
        }, 100);
    };

    /**
     * Saves a new template from the specified path
     * @async
     */
    const saveNewTemplate = async () => {
        if (!newTemplatePath.trim()) return;

        try {
            await addTemplate(newTemplatePath.trim());

            // Reload templates
            const templatePaths = await getTemplatePaths();
            setTemplates(templatePaths);

            setIsAddModalOpen(false);
            setNewTemplatePath('');

            showSuccess('Template added successfully!');
        } catch (err) {
            console.error('Failed to add template:', err);
            showError(`Failed to add template: ${err.message || err}`);
        }
    };

    /**
     * Handles form submission for adding a template
     * @param {React.FormEvent} e - Form submit event
     */
    const handleAddTemplateSubmit = (e) => {
        e.preventDefault();
        saveNewTemplate();
    };

    /**
     * Handles input change for the template path
     * @param {React.ChangeEvent<HTMLInputElement>} e - Input change event
     */
    const handleAddTemplateInputChange = (e) => {
        setNewTemplatePath(e.target.value);
    };

    /**
     * Handles key down events for the add template input
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleAddTemplateKeyDown = (e) => {
        if (e.key === 'Escape') {
            setIsAddModalOpen(false);
        }
    };

    return (
        <div className="template-list-container">
            <div className="template-list-header">
                <div className="template-header-left">
                    <h2>Templates</h2>
                    <button
                        className="template-close-btn"
                        onClick={onClose}
                        title="Close Templates"
                    >
                        <span className="icon icon-x"></span>
                    </button>
                </div>
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

            {/* Modal for adding template */}
            <Modal
                isOpen={isAddModalOpen}
                onClose={() => setIsAddModalOpen(false)}
                title="Add Template"
                size="sm"
                footer={
                    <>
                        <Button
                            variant="ghost"
                            onClick={() => setIsAddModalOpen(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            variant="primary"
                            onClick={saveNewTemplate}
                            disabled={!newTemplatePath.trim()}
                        >
                            Add Template
                        </Button>
                    </>
                }
            >
                <form onSubmit={handleAddTemplateSubmit}>
                    <div className="template-add-form">
                        <div className="form-group">
                            <label htmlFor="template-path">Template Path</label>
                            <input
                                ref={addTemplateInputRef}
                                type="text"
                                id="template-path"
                                className="input"
                                value={newTemplatePath}
                                onChange={handleAddTemplateInputChange}
                                onKeyDown={handleAddTemplateKeyDown}
                                placeholder="Enter path to file or folder to save as template"
                            />
                            <div className="input-hint">
                                Enter the full path to a file or folder that you want to save as a template.
                            </div>
                        </div>
                    </div>
                </form>
            </Modal>
        </div>
    );
};

export default TemplateList;

