import React, { useState, useEffect, useRef } from 'react';
import Breadcrumb from './Breadcrumb';

const LocationBar = ({ currentPath, onPathChange }) => {
    const [isEditing, setIsEditing] = useState(false);
    const [inputPath, setInputPath] = useState(currentPath);
    const inputRef = useRef(null);

    // Aktualisiere den Eingabepfad, wenn sich der aktuelle Pfad ändert
    useEffect(() => {
        setInputPath(currentPath);
    }, [currentPath]);

    // Fokussiere das Eingabefeld, wenn der Bearbeitungsmodus aktiviert wird
    useEffect(() => {
        if (isEditing && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.setSelectionRange(0, inputRef.current.value.length);
        }
    }, [isEditing]);

    // Wechsle in den Bearbeitungsmodus
    const handleEditClick = () => {
        setIsEditing(true);
    };

    // Behandle die Änderung des Eingabepfads
    const handleInputChange = (e) => {
        setInputPath(e.target.value);
    };

    // Navigiere zu einem neuen Pfad, wenn Enter gedrückt wird
    const handleKeyDown = (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            handleSubmit();
        } else if (e.key === 'Escape') {
            setIsEditing(false);
            setInputPath(currentPath);
        }
    };

    // Behandle das Absenden des Formulars
    const handleSubmit = () => {
        // [Backend Integration] - Prüfen, ob der Pfad existiert
        // /* BACKEND_INTEGRATION: Pfad-Existenz prüfen */

        // Navigiere zum neuen Pfad
        if (inputPath && inputPath !== currentPath) {
            onPathChange(inputPath);
        }

        setIsEditing(false);
    };

    // Behandle Klicks außerhalb des Eingabefelds
    const handleBlur = () => {
        setIsEditing(false);
        setInputPath(currentPath);
    };

    return (
        <div className="location-bar">
            {isEditing ? (
                <div className="location-input-container">
                    <input
                        ref={inputRef}
                        type="text"
                        className="location-input"
                        value={inputPath}
                        onChange={handleInputChange}
                        onKeyDown={handleKeyDown}
                        onBlur={handleBlur}
                    />
                    <button
                        className="location-submit-button"
                        onClick={handleSubmit}
                        title="Navigieren"
                        aria-label="Zum Pfad navigieren"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            width="16"
                            height="16"
                        >
                            <path d="M9 18l6-6-6-6" />
                        </svg>
                    </button>
                </div>
            ) : (
                <div className="location-breadcrumb-container" onClick={handleEditClick}>
                    <Breadcrumb
                        path={currentPath}
                        onNavigate={onPathChange}
                    />
                </div>
            )}

            {/* Toggle-Button zum Umschalten zwischen Breadcrumb und Texteingabe */}
            <button
                className="location-toggle-button"
                onClick={() => setIsEditing(!isEditing)}
                title={isEditing ? "Breadcrumb anzeigen" : "Als Text bearbeiten"}
                aria-label={isEditing ? "Breadcrumb anzeigen" : "Als Text bearbeiten"}
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    width="16"
                    height="16"
                >
                    {isEditing ? (
                        <path d="M17 10l-5 5-5-5" /> // Nach unten (zur Breadcrumb)
                    ) : (
                        <path d="M5 12h14M12 5l7 7-7 7" /> // Pfeil (zur Texteingabe)
                    )}
                </svg>
            </button>
        </div>
    );
};

export default LocationBar;