import React, { useState } from 'react';

const SortControls = ({ sortBy, sortDirection, onChange }) => {
    const [isOpen, setIsOpen] = useState(false);

    // Sortiermöglichkeiten
    const sortOptions = [
        { id: 'name', label: 'Name' },
        { id: 'date', label: 'Datum geändert' },
        { id: 'size', label: 'Größe' },
        { id: 'type', label: 'Typ' },
    ];

    // Sortierrichtung-Icons
    const directionIcons = {
        asc: 'M7 14l5-5 5 5',
        desc: 'M7 10l5 5 5-5',
    };

    // Dropdown-Menü öffnen/schließen
    const toggleDropdown = () => {
        setIsOpen(!isOpen);
    };

    // Sortierung ändern
    const handleSortChange = (option) => {
        onChange(option);
        setIsOpen(false);
    };

    // Aktuelle Sortierungsoption
    const currentOption = sortOptions.find(option => option.id === sortBy) || sortOptions[0];

    return (
        <div className="sort-controls">
            <button
                className="sort-button"
                onClick={toggleDropdown}
                aria-expanded={isOpen}
                aria-haspopup="true"
            >
                <span>Sortieren: {currentOption.label}</span>
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
                    className="sort-direction-icon"
                >
                    <path d={directionIcons[sortDirection]} />
                </svg>
            </button>

            {isOpen && (
                <div className="sort-dropdown">
                    {sortOptions.map((option) => (
                        <button
                            key={option.id}
                            className={`sort-option ${sortBy === option.id ? 'active' : ''}`}
                            onClick={() => handleSortChange(option.id)}
                        >
                            {option.label}
                            {sortBy === option.id && (
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
                                    className="sort-check-icon"
                                >
                                    <path d="M20 6L9 17l-5-5" />
                                </svg>
                            )}
                        </button>
                    ))}

                    <div className="sort-direction-controls">
                        <button
                            className={`sort-direction-option ${sortDirection === 'asc' ? 'active' : ''}`}
                            onClick={() => onChange(sortBy)} // Toggle wird in der übergeordneten Komponente behandelt
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
                                <path d={directionIcons.asc} />
                            </svg>
                            <span>Aufsteigend</span>
                        </button>
                        <button
                            className={`sort-direction-option ${sortDirection === 'desc' ? 'active' : ''}`}
                            onClick={() => onChange(sortBy)} // Toggle wird in der übergeordneten Komponente behandelt
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
                                <path d={directionIcons.desc} />
                            </svg>
                            <span>Absteigend</span>
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
};

export default SortControls;