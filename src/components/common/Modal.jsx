import React, { useEffect, useRef } from 'react';
import ReactDOM from 'react-dom';
import Icon from './Icon';
import './common.css';

/**
 * Modal component
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Whether the modal is open
 * @param {Function} props.onClose - Function to call when modal should close
 * @param {string} [props.title] - Modal title
 * @param {React.ReactNode} props.children - Modal content
 * @param {React.ReactNode} [props.footer] - Modal footer content
 * @param {string} [props.size='md'] - Modal size (sm, md, lg, xl, full)
 * @param {boolean} [props.closeOnEsc=true] - Whether to close modal on Escape key
 * @param {boolean} [props.closeOnOverlayClick=true] - Whether to close modal when clicking the overlay
 * @param {boolean} [props.showCloseButton=true] - Whether to show the close button
 * @param {string} [props.className] - Additional CSS class names
 * @returns {React.ReactElement|null} Modal component or null if closed
 */
const Modal = ({
                   isOpen,
                   onClose,
                   title,
                   children,
                   footer,
                   size = 'md',
                   closeOnEsc = true,
                   closeOnOverlayClick = true,
                   showCloseButton = true,
                   className = '',
                   ...rest
               }) => {
    const modalRef = useRef(null);

    // Close modal when Escape key is pressed
    useEffect(() => {
        const handleKeyDown = (event) => {
            if (closeOnEsc && event.key === 'Escape' && isOpen) {
                onClose();
            }
        };

        if (isOpen) {
            document.addEventListener('keydown', handleKeyDown);
            // Prevent scrolling of the body when modal is open
            document.body.style.overflow = 'hidden';
        }

        return () => {
            document.removeEventListener('keydown', handleKeyDown);
            // Restore scrolling when modal is closed
            document.body.style.overflow = '';
        };
    }, [isOpen, onClose, closeOnEsc]);

    // Focus the modal when it opens
    useEffect(() => {
        if (isOpen && modalRef.current) {
            // Save the currently focused element
            const previouslyFocused = document.activeElement;

            // Focus the modal
            modalRef.current.focus();

            // Restore focus when modal closes
            return () => {
                if (previouslyFocused) {
                    previouslyFocused.focus();
                }
            };
        }
    }, [isOpen]);

    /**
     * Behandelt Klicks auf den Modal-Overlay
     * @param {React.MouseEvent} event - Maus-Event
     */
    const handleOverlayClick = (event) => {
        if (
            closeOnOverlayClick &&
            modalRef.current &&
            !modalRef.current.contains(event.target)
        ) {
            onClose();
        }
    };

    // Don't render anything if modal is closed
    if (!isOpen) return null;

    // Build class names
    const modalClasses = [
        'modal',
        `modal-${size}`,
        className
    ].filter(Boolean).join(' ');

    // Create portal to render modal at the body level
    return ReactDOM.createPortal(
        <div className="modal-overlay" onClick={handleOverlayClick}>
            <div
                className={modalClasses}
                ref={modalRef}
                tabIndex={-1}
                role="dialog"
                aria-modal="true"
                aria-labelledby={title ? 'modal-title' : undefined}
                {...rest}
            >
                {/* Modal Header */}
                {(title || showCloseButton) && (
                    <div className="modal-header">
                        {title && (
                            <h2 className="modal-title" id="modal-title">
                                {title}
                            </h2>
                        )}

                        {showCloseButton && (
                            <button
                                className="modal-close"
                                onClick={onClose}
                                aria-label="Close modal"
                            >
                                <Icon name="x" />
                            </button>
                        )}
                    </div>
                )}

                {/* Modal Content */}
                <div className="modal-content">
                    {children}
                </div>

                {/* Modal Footer */}
                {footer && (
                    <div className="modal-footer">
                        {footer}
                    </div>
                )}
            </div>
        </div>,
        document.body
    );
};

export default Modal;