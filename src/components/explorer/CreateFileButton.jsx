import React, { useState, useRef, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import './createFileButton.css';

const CreateFileButton = () => {
    const [isDropdownOpen, setIsDropdownOpen] = useState(false);
    const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
    const [creationType, setCreationType] = useState(''); // 'file' or 'folder'
    const [itemName, setItemName] = useState('');
    const dropdownRef = useRef(null);
    const modalRef = useRef(null);
    const inputRef = useRef(null);

    const { createFile, createDirectory } = useFileSystem();
    const { currentPath } = useHistory();

    // Toggle dropdown
    const toggleDropdown = () => {
        setIsDropdownOpen(prev => !prev);
    };

    // Close dropdown
    const closeDropdown = () => {
        setIsDropdownOpen(false);
    };

    // Handle option click
    const handleOptionClick = (type) => {
        setCreationType(type);
        setItemName(type === 'file' ? 'New File.txt' : 'New Folder');
        setIsCreateModalOpen(true);
        closeDropdown();

        // Focus input on next render
        setTimeout(() => {
            if (inputRef.current) {
                inputRef.current.focus();
                inputRef.current.select();
            }
        }, 0);
    };

    // Handle input change
    const handleNameChange = (e) => {
        setItemName(e.target.value);
    };

    // Handle create
    const handleCreate = async () => {
        if (!itemName.trim()) return;

        try {
            if (creationType === 'file') {
                await createFile(currentPath, itemName);
            } else if (creationType === 'folder') {
                await createDirectory(currentPath, itemName);
            }

            setIsCreateModalOpen(false);
            setItemName('');
        } catch (error) {
            console.error('Failed to create item:', error);
            // In a production app, show user feedback about the error
        }
    };

    // Handle form submission
    const handleSubmit = (e) => {
        e.preventDefault();
        handleCreate();
    };

    // Close dropdown when clicking outside
    useEffect(() => {
        const handleClickOutside = (event) => {
            if (
                dropdownRef.current &&
                !dropdownRef.current.contains(event.target)
            ) {
                closeDropdown();
            }
        };

        document.addEventListener('mousedown', handleClickOutside);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);

    // Close modal when clicking outside
    useEffect(() => {
        const handleClickOutside = (event) => {
            if (
                modalRef.current &&
                !modalRef.current.contains(event.target)
            ) {
                setIsCreateModalOpen(false);
            }
        };

        if (isCreateModalOpen) {
            document.addEventListener('mousedown', handleClickOutside);
        }

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isCreateModalOpen]);

    // Close modal with Escape key
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.key === 'Escape') {
                setIsCreateModalOpen(false);
            }
        };

        if (isCreateModalOpen) {
            document.addEventListener('keydown', handleKeyDown);
        }

        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [isCreateModalOpen]);

    return (
        <div className="create-file-container">
            <button
                className="create-button"
                onClick={toggleDropdown}
                aria-label="Create new item"
                aria-expanded={isDropdownOpen}
                aria-haspopup="true"
            >
                <span className="icon icon-plus"></span>
                <span>Create New</span>
                <span className="icon icon-chevron-down"></span>
            </button>

            {isDropdownOpen && (
                <div className="create-dropdown" ref={dropdownRef}>
                    <ul className="create-options">
                        <li>
                            <button
                                className="create-option"
                                onClick={() => handleOptionClick('file')}
                            >
                                <span className="icon icon-file"></span>
                                <span>Text File</span>
                            </button>
                        </li>
                        <li>
                            <button
                                className="create-option"
                                onClick={() => handleOptionClick('folder')}
                            >
                                <span className="icon icon-folder"></span>
                                <span>Folder</span>
                            </button>
                        </li>
                        <li className="create-divider"></li>
                        <li>
                            <button
                                className="create-option"
                                onClick={() => {
                                    // This would normally open a template selection modal
                                    console.log('Open template selection');
                                    closeDropdown();
                                }}
                            >
                                <span className="icon icon-template"></span>
                                <span>From Template...</span>
                            </button>
                        </li>
                    </ul>
                </div>
            )}

            {isCreateModalOpen && (
                <div className="modal-backdrop">
                    <div className="create-modal" ref={modalRef}>
                        <div className="modal-header">
                            <h3>Create New {creationType === 'file' ? 'File' : 'Folder'}</h3>
                            <button
                                className="modal-close"
                                onClick={() => setIsCreateModalOpen(false)}
                                aria-label="Close"
                            >
                                <span className="icon icon-x"></span>
                            </button>
                        </div>

                        <form onSubmit={handleSubmit} className="modal-content">
                            <div className="form-group">
                                <label htmlFor="item-name">Name:</label>
                                <input
                                    ref={inputRef}
                                    type="text"
                                    id="item-name"
                                    value={itemName}
                                    onChange={handleNameChange}
                                    className="input-field"
                                />
                            </div>

                            <div className="modal-footer">
                                <button
                                    type="button"
                                    className="button button-secondary"
                                    onClick={() => setIsCreateModalOpen(false)}
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    className="button button-primary"
                                    disabled={!itemName.trim()}
                                >
                                    Create
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}
        </div>
    );
};

export default CreateFileButton;