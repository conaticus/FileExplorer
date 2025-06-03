import React, { useState, useRef, useEffect } from 'react';
import Modal from '../common/Modal';
import Button from '../common/Button';

const RenameModal = ({ isOpen, onClose, item, onRename }) => {
    const [newName, setNewName] = useState('');
    const inputRef = useRef(null);

    // Initialize name when modal opens
    useEffect(() => {
        if (isOpen && item) {
            setNewName(item.name);
            // Focus and select name without extension for files
            setTimeout(() => {
                if (inputRef.current) {
                    inputRef.current.focus();
                    const isDirectory = item.isDirectory || 'sub_file_count' in item;
                    if (!isDirectory && item.name.includes('.')) {
                        const lastDotIndex = item.name.lastIndexOf('.');
                        inputRef.current.setSelectionRange(0, lastDotIndex);
                    } else {
                        inputRef.current.select();
                    }
                }
            }, 100);
        }
    }, [isOpen, item]);

    // Handle form submission
    const handleSubmit = (e) => {
        e.preventDefault();
        if (newName.trim() && newName !== item.name) {
            onRename(item, newName.trim());
        }
        onClose();
    };

    // Handle input change
    const handleChange = (e) => {
        setNewName(e.target.value);
    };

    // Handle key down events
    const handleKeyDown = (e) => {
        if (e.key === 'Escape') {
            onClose();
        }
    };

    if (!item) return null;

    const isDirectory = item.isDirectory || 'sub_file_count' in item;

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={`Rename ${isDirectory ? 'Folder' : 'File'}`}
            size="sm"
            footer={
                <>
                    <Button
                        variant="ghost"
                        onClick={onClose}
                    >
                        Cancel
                    </Button>
                    <Button
                        type="submit"
                        variant="primary"
                        disabled={!newName.trim() || newName === item.name}
                        onClick={handleSubmit}
                    >
                        Rename
                    </Button>
                </>
            }
        >
            <form onSubmit={handleSubmit}>
                <div className="form-group">
                    <label htmlFor="new-name">
                        {isDirectory ? 'Folder name:' : 'File name:'}
                    </label>
                    <input
                        ref={inputRef}
                        type="text"
                        id="new-name"
                        value={newName}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        className="input"
                        placeholder={`Enter new ${isDirectory ? 'folder' : 'file'} name`}
                    />
                    <div className="input-hint">
                        {isDirectory
                            ? 'Choose a descriptive name for your folder'
                            : 'Include the file extension (e.g., .txt, .jpg, .pdf)'
                        }
                    </div>
                </div>
            </form>
        </Modal>
    );
};

export default RenameModal;