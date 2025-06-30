/**
 * Simple notification system to replace browser alerts
 */

let notificationContainer = null;

// Initialize notification container
const initNotificationContainer = () => {
    if (!notificationContainer) {
        notificationContainer = document.createElement('div');
        notificationContainer.id = 'notification-container';
        notificationContainer.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            z-index: 10000;
            pointer-events: none;
        `;
        document.body.appendChild(notificationContainer);
    }
};

// Show notification
export const showNotification = (message, type = 'info', duration = 3000) => {
    initNotificationContainer();

    const notification = document.createElement('div');
    notification.textContent = message;

    const colors = {
        info: { bg: '#0891b2', text: 'white' },
        success: { bg: '#16a34a', text: 'white' },
        error: { bg: '#dc2626', text: 'white' },
        warning: { bg: '#f59e0b', text: 'white' }
    };

    const color = colors[type] || colors.info;

    notification.style.cssText = `
        background: ${color.bg};
        color: ${color.text};
        padding: 12px 20px;
        border-radius: 6px;
        margin-bottom: 10px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        font-size: 14px;
        max-width: 300px;
        word-wrap: break-word;
        animation: slideIn 0.3s ease-out;
        pointer-events: auto;
        cursor: pointer;
    `;

    // Add CSS animation if not already added
    if (!document.getElementById('notification-styles')) {
        const style = document.createElement('style');
        style.id = 'notification-styles';
        style.textContent = `
            @keyframes slideIn {
                from {
                    opacity: 0;
                    transform: translateX(100%);
                }
                to {
                    opacity: 1;
                    transform: translateX(0);
                }
            }
            @keyframes fadeOut {
                from {
                    opacity: 1;
                    transform: translateX(0);
                }
                to {
                    opacity: 0;
                    transform: translateX(100%);
                }
            }
        `;
        document.head.appendChild(style);
    }

    // Click to dismiss
    notification.addEventListener('click', () => {
        removeNotification(notification);
    });

    notificationContainer.appendChild(notification);

    // Auto remove after duration
    if (duration > 0) {
        setTimeout(() => {
            removeNotification(notification);
        }, duration);
    }

    return notification;
};

// Remove notification
const removeNotification = (notification) => {
    if (notification && notification.parentNode) {
        notification.style.animation = 'fadeOut 0.3s ease-out';
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 300);
    }
};

// Convenience methods
export const showSuccess = (message, duration) => showNotification(message, 'success', duration);
export const showError = (message, duration) => showNotification(message, 'error', duration);
export const showWarning = (message, duration) => showNotification(message, 'warning', duration);
export const showInfo = (message, duration) => showNotification(message, 'info', duration);

// Simple confirm dialog replacement
export const showConfirm = (message, title = 'Confirm') => {
    return new Promise((resolve) => {
        const modal = document.createElement('div');
        modal.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 10001;
        `;

        const dialog = document.createElement('div');
        dialog.style.cssText = `
            background: #ffffff;
            border-radius: 8px;
            padding: 24px;
            max-width: 400px;
            width: 90%;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        `;

        const titleEl = document.createElement('h3');
        titleEl.textContent = title;
        titleEl.style.cssText = `
            margin: 0 0 16px 0;
            color: #1a1c1e;
            font-size: 18px;
        `;

        const messageEl = document.createElement('p');
        messageEl.textContent = message;
        messageEl.style.cssText = `
            margin: 0 0 24px 0;
            color: #4b5563;
            line-height: 1.5;
        `;

        const buttonContainer = document.createElement('div');
        buttonContainer.style.cssText = `
            display: flex;
            gap: 12px;
            justify-content: flex-end;
        `;

        const cancelBtn = document.createElement('button');
        cancelBtn.textContent = 'Cancel';
        cancelBtn.style.cssText = `
            padding: 8px 16px;
            border: 1px solid #e0e4e8;
            background: transparent;
            color: #1a1c1e;
            border-radius: 6px;
            cursor: pointer;
        `;

        const confirmBtn = document.createElement('button');
        confirmBtn.textContent = 'Confirm';
        confirmBtn.style.cssText = `
            padding: 8px 16px;
            border: none;
            background: #0672ef;
            color: white;
            border-radius: 6px;
            cursor: pointer;
        `;

        const cleanup = () => {
            if (modal.parentNode) {
                modal.parentNode.removeChild(modal);
            }
        };

        cancelBtn.addEventListener('click', () => {
            cleanup();
            resolve(false);
        });

        confirmBtn.addEventListener('click', () => {
            cleanup();
            resolve(true);
        });

        modal.addEventListener('click', (e) => {
            if (e.target === modal) {
                cleanup();
                resolve(false);
            }
        });

        // ESC key to cancel
        const handleKeyDown = (e) => {
            if (e.key === 'Escape') {
                cleanup();
                resolve(false);
                document.removeEventListener('keydown', handleKeyDown);
            }
        };
        document.addEventListener('keydown', handleKeyDown);

        buttonContainer.appendChild(cancelBtn);
        buttonContainer.appendChild(confirmBtn);
        dialog.appendChild(titleEl);
        dialog.appendChild(messageEl);
        dialog.appendChild(buttonContainer);
        modal.appendChild(dialog);
        document.body.appendChild(modal);

        // Focus confirm button
        confirmBtn.focus();
    });
};