import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { showError, showSuccess } from '../../utils/NotificationSystem';
import Modal from '../common/Modal';
import Button from '../common/Button';

/**
 * Modal component for generating a hash file for a selected file or directory
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Controls whether the modal is visible
 * @param {Function} props.onClose - Handler called when the modal is closed
 * @param {Object} props.item - File or directory item to generate hash for
 * @param {string} props.item.path - Path to the file or directory
 * @param {string} props.item.name - Name of the file or directory
 * @returns {React.ReactElement|null} Hash file generation modal or null if no item provided
 */
const HashFileModal = ({ isOpen, onClose, item }) => {
    const [fileName, setFileName] = useState('');
    const [isGenerating, setIsGenerating] = useState(false);
    const inputRef = useRef(null);
    const { currentPath } = useHistory();
    const { loadDirectory } = useFileSystem();

    /**
     * Initialize filename when modal opens and focus the input field
     */
    useEffect(() => {
        if (isOpen && item) {
            setFileName(`${item.name}.hash`);
            // Focus and select filename without extension
            setTimeout(() => {
                if (inputRef.current) {
                    inputRef.current.focus();
                    const lastDotIndex = fileName.lastIndexOf('.');
                    if (lastDotIndex > 0) {
                        inputRef.current.setSelectionRange(0, lastDotIndex);
                    } else {
                        inputRef.current.select();
                    }
                }
            }, 100);
        }
    }, [isOpen, item, fileName]);

    /**
     * Handle form submission to generate hash file
     * @param {React.FormEvent} e - Form event
     */
    const handleSubmit = async (e) => {
        e.preventDefault();
        if (!fileName.trim() || !item) return;

        setIsGenerating(true);
        try {
            const separator = currentPath.includes('\\') ? '\\' : '/';
            const outputPath = `${currentPath}${separator}${fileName.trim()}`;

            const hash = await invoke('gen_hash_and_save_to_file', {
                source_path: item.path,
                output_path: outputPath
            });

            await loadDirectory(currentPath);
            showSuccess(`Hash generated and saved to ${fileName}: ${hash.substring(0, 16)}...`);
            onClose();
        } catch (error) {
            console.error('Hash generation to file failed:', error);
            showError(`Failed to generate hash file: ${error.message || error}`);
        } finally {
            setIsGenerating(false);
        }
    };

    /**
     * Handle changes to the filename input field
     * @param {React.ChangeEvent<HTMLInputElement>} e - Input change event
     */
    const handleChange = (e) => {
        setFileName(e.target.value);
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

    if (!item) return null;

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Generate Hash to File"
            size="sm"
            footer={
                <>
                    <Button
                        variant="ghost"
                        onClick={onClose}
                        disabled={isGenerating}
                    >
                        Cancel
                    </Button>
                    <Button
                        type="submit"
                        variant="primary"
                        disabled={!fileName.trim() || isGenerating}
                        onClick={handleSubmit}
                    >
                        {isGenerating ? 'Generating...' : 'Generate Hash'}
                    </Button>
                </>
            }
        >
            <form onSubmit={handleSubmit}>
                <div className="form-group">
                    <label htmlFor="hash-filename">
                        Hash file name:
                    </label>
                    <input
                        ref={inputRef}
                        type="text"
                        id="hash-filename"
                        value={fileName}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        className="input"
                        placeholder="Enter hash file name"
                        disabled={isGenerating}
                    />
                    <div className="input-hint">
                        The hash will be generated for "{item.name}" and saved to this file.
                    </div>
                </div>
            </form>
        </Modal>
    );
};

export default HashFileModal;