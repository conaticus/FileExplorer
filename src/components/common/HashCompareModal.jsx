import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { showError, showSuccess } from '../../utils/NotificationSystem';
import Modal from '../common/Modal';
import Button from '../common/Button';

/**
 * Modal component for comparing file or directory hash with a provided hash value
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Controls whether the modal is visible
 * @param {Function} props.onClose - Handler called when the modal is closed
 * @param {Object} props.item - File or directory item to compare hash for
 * @param {string} props.item.path - Path to the file or directory
 * @param {string} props.item.name - Name of the file or directory
 * @returns {React.ReactElement|null} Hash comparison modal or null if no item provided
 */
const HashCompareModal = ({ isOpen, onClose, item }) => {
    const [hashValue, setHashValue] = useState('');
    const [isComparing, setIsComparing] = useState(false);
    const inputRef = useRef(null);

    /**
     * Initialize state when modal opens and focus the input field
     */
    useEffect(() => {
        if (isOpen) {
            setHashValue('');
            // Focus input after modal animation
            setTimeout(() => {
                if (inputRef.current) {
                    inputRef.current.focus();
                }
            }, 100);
        }
    }, [isOpen]);

    /**
     * Handle form submission to compare hash values
     * @param {React.FormEvent} e - Form event
     */
    const handleSubmit = async (e) => {
        e.preventDefault();
        if (!hashValue.trim() || !item) return;

        setIsComparing(true);
        try {
            const matches = await invoke('compare_file_or_dir_with_hash', {
                path: item.path,
                hashToCompare: hashValue.trim()
            });

            if (matches) {
                showSuccess('✓ Hash matches! File integrity verified.');
            } else {
                showError('✗ Hash does not match! File may be corrupted or modified.');
            }
            onClose();
        } catch (error) {
            console.error('Hash comparison failed:', error);
            showError(`Failed to compare hash: ${error.message || error}`);
        } finally {
            setIsComparing(false);
        }
    };

    /**
     * Handle changes to the hash input field
     * @param {React.ChangeEvent<HTMLTextAreaElement>} e - Input change event
     */
    const handleChange = (e) => {
        setHashValue(e.target.value);
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

    /**
     * Clean up pasted hash values by removing whitespace and common prefixes
     * @param {React.ClipboardEvent<HTMLTextAreaElement>} e - Paste event
     */
    const handlePaste = (e) => {
        // Allow the paste to happen, then clean it up
        setTimeout(() => {
            const pastedValue = e.target.value;
            // Remove any whitespace and common hash file prefixes
            const cleanedValue = pastedValue
                .replace(/^\s*([A-Fa-f0-9]+)\s*.*$/, '$1') // Extract just the hex part
                .replace(/\s+/g, '') // Remove all whitespace
                .toLowerCase();
            setHashValue(cleanedValue);
        }, 0);
    };

    if (!item) return null;

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Compare Hash"
            size="md"
            footer={
                <>
                    <Button
                        variant="ghost"
                        onClick={onClose}
                        disabled={isComparing}
                    >
                        Cancel
                    </Button>
                    <Button
                        type="submit"
                        variant="primary"
                        disabled={!hashValue.trim() || isComparing}
                        onClick={handleSubmit}
                    >
                        {isComparing ? 'Comparing...' : 'Compare Hash'}
                    </Button>
                </>
            }
        >
            <form onSubmit={handleSubmit}>
                <div className="form-group">
                    <label htmlFor="hash-value">
                        Hash value to compare:
                    </label>
                    <textarea
                        ref={inputRef}
                        id="hash-value"
                        value={hashValue}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        onPaste={handlePaste}
                        className="input"
                        placeholder="Enter or paste the hash value to compare against"
                        rows={4}
                        disabled={isComparing}
                        style={{ resize: 'vertical', fontFamily: 'monospace', fontSize: '13px' }}
                    />
                    <div className="input-hint">
                        Enter the hash value you want to compare "{item.name}" against.
                        The comparison will verify the file's integrity.
                    </div>
                </div>
            </form>
        </Modal>
    );
};

export default HashCompareModal;