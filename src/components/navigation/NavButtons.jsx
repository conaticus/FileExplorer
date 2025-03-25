import React from 'react';

const NavButtons = ({ canGoBack, canGoForward, onGoBack, onGoForward }) => {
    return (
        <div className="nav-buttons">
            {/* Zurück-Button */}
            <button
                className={`nav-button ${!canGoBack ? 'disabled' : ''}`}
                onClick={onGoBack}
                disabled={!canGoBack}
                title="Zurück"
                aria-label="Zurück"
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
                    <path d="M19 12H5M12 19l-7-7 7-7" />
                </svg>
            </button>

            {/* Vorwärts-Button */}
            <button
                className={`nav-button ${!canGoForward ? 'disabled' : ''}`}
                onClick={onGoForward}
                disabled={!canGoForward}
                title="Vorwärts"
                aria-label="Vorwärts"
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
                    <path d="M5 12h14M12 5l7 7-7 7" />
                </svg>
            </button>

            {/* Nach oben-Button */}
            <button
                className="nav-button"
                onClick={() => {
                    // [Backend Integration] - Zum übergeordneten Verzeichnis navigieren
                    // /* BACKEND_INTEGRATION: Zum übergeordneten Verzeichnis navigieren */
                    console.log('Navigiere zum übergeordneten Verzeichnis');
                }}
                title="Nach oben"
                aria-label="Nach oben zum übergeordneten Verzeichnis"
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
                    <path d="M17 11l-5-5-5 5M17 18l-5-5-5 5" />
                </svg>
            </button>

            {/* Aktualisieren-Button */}
            <button
                className="nav-button"
                onClick={() => {
                    // [Backend Integration] - Aktuelles Verzeichnis neu laden
                    // /* BACKEND_INTEGRATION: Aktuelles Verzeichnis neu laden */
                    console.log('Aktualisiere Verzeichnis');
                }}
                title="Aktualisieren"
                aria-label="Aktualisieren"
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
                    <path d="M1 4v6h6M23 20v-6h-6" />
                    <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" />
                </svg>
            </button>
        </div>
    );
};

export default NavButtons;