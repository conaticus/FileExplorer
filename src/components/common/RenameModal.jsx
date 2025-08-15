import React, { useState, useRef, useEffect } from 'react';
import Modal from '../common/Modal';
import Button from '../common/Button';

/**
 * Modal component for renaming files and folders
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Controls whether the modal is visible
 * @param {Function} props.onClose - Handler called when the modal is closed
 * @param {Object} props.item - File or folder item to rename
 * @param {string} props.item.name - Current name of the file or folder
 * @param {boolean} [props.item.isDirectory] - Whether the item is a directory
 * @param {Function} props.onRename - Handler called with (item, newName) when rename is confirmed
 * @returns {React.ReactElement|null} Rename modal or null if no item provided
 */
const RenameModal = ({ isOpen, onClose, item, onRename }) => {
    const [newName, setNewName] = useState('');
    const inputRef = useRef(null);

    /**
     * Initialize name when modal opens and select appropriate portion of text
     */
    useEffect(() => {
        if (isOpen && item && item.name) {
            setNewName(item.name);
            // Focus and select name without extension for files
            setTimeout(() => {
                if (inputRef.current) {
                    inputRef.current.focus();
                    const isDirectory = item.isDirectory || (typeof item === 'object' && 'sub_file_count' in item);
                    if (!isDirectory && item.name.includes('.')) {
                        const lastDotIndex = item.name.lastIndexOf('.');
                        inputRef.current.setSelectionRange(0, lastDotIndex);
                    } else {
                        inputRef.current.select();
                    }
                }
            }, 100);
        } else if (!isOpen) {
            // Reset when modal closes
            setNewName('');
        }
    }, [isOpen, item]);

    /**
     * Handle form submission to rename the item
     * @param {React.FormEvent} e - Form event
     */
    const handleSubmit = (e) => {
        e.preventDefault();
        console.log('RenameModal handleSubmit called with:', { newName, itemName: item?.name });
        
        if (newName && newName.trim() && newName.trim() !== item.name) {
            console.log('RenameModal: Calling onRename with:', item, newName.trim());
            onRename(item, newName.trim());
        } else {
            console.log('RenameModal: Not calling onRename - conditions not met');
        }
        onClose();
    };

    /**
     * Handle changes to the name input field
     * @param {React.ChangeEvent<HTMLInputElement>} e - Input change event
     */
    const handleChange = (e) => {
        setNewName(e.target.value || '');
    };

    /**
     * Handle keyboard events, specifically for modal escape
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleKeyDown = (e) => {
        if (e.key === 'Escape') {
            onClose();
        }
    };

    if (!item || typeof item !== 'object' || !item.name) return null;

    const isDirectory = item.isDirectory || ('sub_file_count' in item);

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
                        disabled={!newName || !newName.trim() || newName.trim() === item.name}
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
                        value={newName || ''}
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