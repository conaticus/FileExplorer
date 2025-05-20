import React, { useState, useRef, useEffect } from 'react';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useHistory } from '../../providers/HistoryProvider';
import Modal from '../common/Modal';
import Button from '../common/Button';
import './createFileButton.css';

const CreateFileButton = () => {
    const [isDropdownOpen, setIsDropdownOpen] = useState(false);
    const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
    const [creationType, setCreationType] = useState(''); // 'file' or 'folder'
    const [itemName, setItemName] = useState('');
    const dropdownRef = useRef(null);
    const inputRef = useRef(null);

    const { createFile, createDirectory } = useFileSystem();
    const { currentPath } = useHistory();

    // Listen for custom events
    useEffect(() => {
        const handleCreateFile = () => {
            handleOptionClick('file');
        };

        const handleCreateFolder = () => {
            handleOptionClick('folder');
        };

        document.addEventListener('create-file', handleCreateFile);
        document.addEventListener('create-folder', handleCreateFolder);

        return () => {
            document.removeEventListener('create-file', handleCreateFile);
            document.removeEventListener('create-folder', handleCreateFolder);
        };
    }, []);

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
        setItemName(getDefaultName(type));
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

    // Get default name for new items
    const getDefaultName = (type) => {
        return type === 'file' ? 'New File.txt' : 'New Folder';
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

            // Check for specific error types
            if (error.message && error.message.includes('already exists')) {
                const shouldCreateCopy = confirm(
                    `An item named "${itemName}" already exists. Would you like to create a copy instead?`
                );

                if (shouldCreateCopy) {
                    const extension = itemName.includes('.') ? '.' + itemName.split('.').pop() : '';
                    const baseName = extension ? itemName.replace(extension, '') : itemName;
                    const copyName = `${baseName} - Copy${extension}`;
                    setItemName(copyName);
                    return; // Don't close modal, let user try again
                }
            } else {
                alert(`Failed to create ${creationType}: ${error.message || error}`);
            }
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

    return (
        <>
            <div className="create-file-container">
                <button
                    className="create-button"
                    onClick={toggleDropdown}
                    aria-label="Create new item"
                    aria-expanded={isDropdownOpen}
                    aria-haspopup="true"
                >
                    <span className="icon icon-plus"></span>
                    <span>New</span>
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
                                    <span className="shortcut">Ctrl+N</span>
                                </button>
                            </li>
                            <li>
                                <button
                                    className="create-option"
                                    onClick={() => handleOptionClick('folder')}
                                >
                                    <span className="icon icon-folder"></span>
                                    <span>Folder</span>
                                    <span className="shortcut">Ctrl+Shift+N</span>
                                </button>
                            </li>
                            <li className="create-divider"></li>
                            <li>
                                <button
                                    className="create-option"
                                    onClick={() => {
                                        document.dispatchEvent(new CustomEvent('open-templates'));
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
            </div>

            <Modal
                isOpen={isCreateModalOpen}
                onClose={() => setIsCreateModalOpen(false)}
                title={`Create New ${creationType === 'file' ? 'File' : 'Folder'}`}
                size="sm"
                footer={
                    <>
                        <Button
                            variant="ghost"
                            onClick={() => setIsCreateModalOpen(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            variant="primary"
                            disabled={!itemName.trim()}
                            onClick={handleCreate}
                        >
                            Create
                        </Button>
                    </>
                }
            >
                <form onSubmit={handleSubmit}>
                    <div className="form-group">
                        <label htmlFor="item-name">
                            {creationType === 'file' ? 'File name:' : 'Folder name:'}
                        </label>
                        <input
                            ref={inputRef}
                            type="text"
                            id="item-name"
                            value={itemName}
                            onChange={handleNameChange}
                            className="input"
                            placeholder={`Enter ${creationType} name`}
                        />
                        <div className="input-hint">
                            {creationType === 'file'
                                ? 'Include the file extension (e.g., .txt, .md, .json)'
                                : 'Choose a descriptive name for your folder'
                            }
                        </div>
                    </div>
                </form>
            </Modal>
        </>
    );
};

export default CreateFileButton;