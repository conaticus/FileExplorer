import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { showError, showSuccess } from '../../utils/NotificationSystem';
import Modal from '../common/Modal';
import Button from '../common/Button';

const HashFileModal = ({ isOpen, onClose, item }) => {
    const [fileName, setFileName] = useState('');
    const [isGenerating, setIsGenerating] = useState(false);
    const inputRef = useRef(null);
    const { currentPath } = useHistory();
    const { loadDirectory } = useFileSystem();

    // Initialize filename when modal opens
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

    // Handle form submission
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

    // Handle input change
    const handleChange = (e) => {
        setFileName(e.target.value);
    };

    // Handle key down events
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