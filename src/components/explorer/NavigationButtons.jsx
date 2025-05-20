import React from 'react';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import './navigationButtons.css';

const NavigationButtons = () => {
    const { canGoBack, canGoForward, goBack, goForward, currentPath } = useHistory();
    const { loadDirectory } = useFileSystem();

    // Handle refresh - only refresh the current directory, not the whole page
    const handleRefresh = async () => {
        if (currentPath) {
            try {
                await loadDirectory(currentPath);
                console.log('Directory refreshed successfully');

                // Optionally show a brief success indicator
                const notification = document.createElement('div');
                notification.textContent = 'Directory refreshed';
                notification.style.cssText = `
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    background: var(--success);
                    color: white;
                    padding: 8px 16px;
                    border-radius: 4px;
                    z-index: 10000;
                    font-size: 14px;
                    animation: slideIn 0.3s ease-out;
                `;
                document.body.appendChild(notification);
                setTimeout(() => {
                    notification.remove();
                }, 2000);
            } catch (error) {
                console.error('Failed to refresh directory:', error);

                // Show error notification
                const notification = document.createElement('div');
                notification.textContent = 'Failed to refresh directory';
                notification.style.cssText = `
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    background: var(--error);
                    color: white;
                    padding: 8px 16px;
                    border-radius: 4px;
                    z-index: 10000;
                    font-size: 14px;
                    animation: slideIn 0.3s ease-out;
                `;
                document.body.appendChild(notification);
                setTimeout(() => {
                    notification.remove();
                }, 3000);
            }
        }
    };

    return (
        <div className="navigation-buttons">
            <button
                className="nav-button"
                onClick={goBack}
                disabled={!canGoBack}
                aria-label="Go back"
                title="Go back"
            >
                <span className="icon icon-arrow-left"></span>
            </button>

            <button
                className="nav-button"
                onClick={goForward}
                disabled={!canGoForward}
                aria-label="Go forward"
                title="Go forward"
            >
                <span className="icon icon-arrow-right"></span>
            </button>

            <button
                className="nav-button"
                onClick={handleRefresh}
                aria-label="Refresh"
                title="Refresh current directory"
            >
                <span className="icon icon-refresh"></span>
            </button>
        </div>
    );
};

export default NavigationButtons;