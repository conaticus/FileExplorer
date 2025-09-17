import React, { useRef, useEffect } from 'react';
import Modal from '../common/Modal';
import Button from '../common/Button';
import { showSuccess, showError } from '../../utils/NotificationSystem';

/**
 * Modal component for displaying a generated hash with copy functionality
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Controls whether the modal is visible
 * @param {Function} props.onClose - Handler called when the modal is closed
 * @param {string} props.hash - The hash value to display
 * @param {string} props.fileName - The name of the file the hash was generated for
 * @returns {React.ReactElement|null} Hash display modal or null if no hash provided
 */
const HashDisplayModal = ({ isOpen, onClose, hash, fileName }) => {
    const textAreaRef = useRef(null);

    /**
     * Focus and select the hash text when modal opens
     */
    useEffect(() => {
        if (isOpen && hash && textAreaRef.current) {
            setTimeout(() => {
                textAreaRef.current.focus();
                textAreaRef.current.select();
            }, 100);
        }
    }, [isOpen, hash]);

    /**
     * Copy hash to clipboard using fallback method
     */
    const copyToClipboard = async () => {
        if (!hash) return;

        try {
            // Try modern clipboard API first
            await navigator.clipboard.writeText(hash);
            showSuccess('Hash copied to clipboard!');
        } catch (error) {
            try {
                // Fallback to document.execCommand
                if (textAreaRef.current) {
                    textAreaRef.current.select();
                    document.execCommand('copy');
                    showSuccess('Hash copied to clipboard!');
                }
            } catch (fallbackError) {
                showError('Failed to copy to clipboard. Please copy manually.');
            }
        }
    };

    /**
     * Handle keyboard events
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleKeyDown = (e) => {
        if (e.key === 'Escape') {
            onClose();
        } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
            copyToClipboard();
        }
    };

    if (!hash) return null;

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={`Hash for "${fileName}"`}
            size="md"
            footer={
                <>
                    <Button
                        variant="ghost"
                        onClick={onClose}
                    >
                        Close
                    </Button>
                    <Button
                        variant="primary"
                        onClick={copyToClipboard}
                    >
                        Copy to Clipboard
                    </Button>
                </>
            }
        >
            <div className="form-group">
                <label htmlFor="hash-display">
                    Generated Hash:
                </label>
                <textarea
                    ref={textAreaRef}
                    id="hash-display"
                    value={hash}
                    readOnly
                    className="input"
                    style={{
                        height: '120px',
                        resize: 'vertical',
                        fontFamily: 'monospace',
                        fontSize: '12px',
                        wordBreak: 'break-all'
                    }}
                    onKeyDown={handleKeyDown}
                />
                <div className="input-hint">
                    The hash has been generated successfully. Use Ctrl+C (Cmd+C on Mac) or click the button to copy.
                </div>
            </div>
        </Modal>
    );
};

export default HashDisplayModal;
