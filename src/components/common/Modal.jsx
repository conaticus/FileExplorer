import React, { useEffect } from 'react';
import Button from './Button';

/**
 * Modal-Komponente für Dialoge und Popup-Fenster
 *
 * @param {Object} props - Die Komponenten-Props
 * @param {boolean} props.isOpen - Ob das Modal geöffnet ist
 * @param {Function} props.onClose - Callback zum Schließen des Modals
 * @param {string} props.title - Titel des Modals
 * @param {React.ReactNode} props.children - Inhalt des Modals
 * @param {React.ReactNode} [props.footer] - Optionaler Footer-Inhalt
 * @param {string} [props.size='medium'] - Größe des Modals (small, medium, large)
 * @param {boolean} [props.closeOnClickOutside=true] - Ob das Modal beim Klick außerhalb geschlossen werden soll
 * @param {boolean} [props.showCloseButton=true] - Ob der Schließen-Button angezeigt werden soll
 * @param {string} [props.className] - Zusätzliche CSS-Klassen
 */
const Modal = ({
                   isOpen,
                   onClose,
                   title,
                   children,
                   footer,
                   size = 'medium',
                   closeOnClickOutside = true,
                   showCloseButton = true,
                   className = '',
               }) => {
    // Blockiere das Scrollen im Hintergrund, wenn das Modal geöffnet ist
    useEffect(() => {
        if (isOpen) {
            document.body.style.overflow = 'hidden';
        } else {
            document.body.style.overflow = 'auto';
        }

        return () => {
            document.body.style.overflow = 'auto';
        };
    }, [isOpen]);

    // Schließe das Modal mit der Escape-Taste
    useEffect(() => {
        const handleKeyDown = (event) => {
            if (event.key === 'Escape' && isOpen) {
                onClose();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    // Wenn das Modal nicht geöffnet ist, rendere nichts
    if (!isOpen) return null;

    // CSS-Klassen für das Modal
    const modalClasses = [
        'modal',
        `modal-${size}`,
        className
    ].filter(Boolean).join(' ');

    // Behandle Klicks auf den Hintergrund
    const handleBackdropClick = (e) => {
        if (closeOnClickOutside && e.target === e.currentTarget) {
            onClose();
        }
    };

    return (
        <div className="modal-backdrop" onClick={handleBackdropClick}>
            <div className={modalClasses} onClick={(e) => e.stopPropagation()}>
                <div className="modal-header">
                    <h2 className="modal-title">{title}</h2>
                    {showCloseButton && (
                        <button
                            className="modal-close-button"
                            onClick={onClose}
                            aria-label="Schließen"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                width="18"
                                height="18"
                            >
                                <path d="M18 6L6 18M6 6l12 12" />
                            </svg>
                        </button>
                    )}
                </div>

                <div className="modal-body">
                    {children}
                </div>

                {footer && (
                    <div className="modal-footer">
                        {footer}
                    </div>
                )}
            </div>
        </div>
    );
};

/**
 * Standard-Footer für Modals mit Abbrechen- und Bestätigen-Buttons
 */
export const ModalFooter = ({
                                onCancel,
                                onConfirm,
                                cancelText = 'Abbrechen',
                                confirmText = 'Bestätigen',
                                confirmVariant = 'primary',
                                isConfirmDisabled = false,
                                isConfirmLoading = false,
                            }) => {
    return (
        <>
            <Button
                variant="tertiary"
                onClick={onCancel}
            >
                {cancelText}
            </Button>
            <Button
                variant={confirmVariant}
                onClick={onConfirm}
                isDisabled={isConfirmDisabled}
                icon={isConfirmLoading ? 'M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83' : undefined}
            >
                {confirmText}
            </Button>
        </>
    );
};

/**
 * Hilfsfunktion zum Erstellen eines Bestätigungsdialogs
 */
export const createConfirmDialog = ({
                                        title = 'Bestätigen',
                                        message,
                                        confirmText = 'Bestätigen',
                                        cancelText = 'Abbrechen',
                                        confirmVariant = 'primary',
                                        onConfirm,
                                        onCancel,
                                    }) => {
    // Diese Funktion erstellt ein Modal und rendert es dynamisch
    // In einer realen Implementierung würde hier ein separater Modal-Provider verwendet

    // Beispiel für die Nutzung im Code:
    console.log('createConfirmDialog wurde aufgerufen mit:', {
        title,
        message,
        confirmText,
        cancelText,
        confirmVariant,
        onConfirm,
        onCancel,
    });

    // Alert als Fallback
    if (window.confirm(message)) {
        onConfirm?.();
    } else {
        onCancel?.();
    }
};

export default Modal;