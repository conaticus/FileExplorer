import React from 'react';
import { useHistory } from '../../providers/HistoryProvider';
import './navigationButtons.css';

const NavigationButtons = () => {
    const { canGoBack, canGoForward, goBack, goForward } = useHistory();

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
                onClick={() => window.location.reload()}
                aria-label="Refresh"
                title="Refresh"
            >
                <span className="icon icon-refresh"></span>
            </button>
        </div>
    );
};

export default NavigationButtons;