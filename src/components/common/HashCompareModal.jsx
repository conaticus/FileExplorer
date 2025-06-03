import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { showError, showSuccess } from '../../utils/NotificationSystem';
import Modal from '../common/Modal';
import Button from '../common/Button';

const HashCompareModal = ({ isOpen, onClose, item }) => {
    const [hashValue, setHashValue] = useState('');
    const [isComparing, setIsComparing] = useState(false);
    const inputRef = useRef(null);

    // Initialize when modal opens
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

    // Handle form submission
    const handleSubmit = async (e) => {
        e.preventDefault();
        if (!hashValue.trim() || !item) return;

        setIsComparing(true);
        try {
            const matches = await invoke('compare_file_or_dir_with_hash', {
                path: item.path,
                hash_to_compare: hashValue.trim()
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

    // Handle input change
    const handleChange = (e) => {
        setHashValue(e.target.value);
    };

    // Handle key down events
    const handleKeyDown = (e) => {
        if (e.key === 'Escape') {
            onClose();
        }
    };

    // Handle paste event to clean up hash value
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