import React, { useState, useEffect, useRef } from 'react';
import './searchBar.css';

/**
 * SearchBar component - Provides a search input with expand/collapse functionality
 *
 * @param {Object} props - Component props
 * @param {string} [props.value=''] - Initial value for the search input
 * @param {Function} [props.onChange] - Callback function when input value changes
 * @param {Function} [props.onSubmit] - Callback function when form is submitted
 * @param {string} [props.placeholder='Search files and folders'] - Placeholder text for the input
 * @returns {React.ReactElement} SearchBar component
 */
const SearchBar = ({ value = '', onChange, onSubmit, placeholder = 'Search files and folders' }) => {
    const [inputValue, setInputValue] = useState(value);
    const [isFocused, setIsFocused] = useState(false);
    const [isExpanded, setIsExpanded] = useState(true); // Always expanded
    const inputRef = useRef(null);
    const searchBarRef = useRef(null);

    /**
     * Update local state when props change
     */
    useEffect(() => {
        setInputValue(value);
    }, [value]);

    /**
     * Handle input change event
     * @param {React.ChangeEvent<HTMLInputElement>} e - Input change event
     */
    const handleChange = (e) => {
        const newValue = e.target.value;
        setInputValue(newValue);

        if (onChange) {
            onChange(newValue);
        }
    };

    /**
     * Handle form submission event
     * @param {React.FormEvent} e - Form submit event
     */
    const handleSubmit = (e) => {
        e.preventDefault();

        if (onSubmit) {
            onSubmit(inputValue);
        }
    };

    /**
     * Handle input focus event
     */
    const handleFocus = () => {
        setIsFocused(true);
        setIsExpanded(true);
    };

    /**
     * Handle input blur event
     * Only collapses search bar if input is empty
     */
    const handleBlur = () => {
        setIsFocused(false);

        // Only collapse if empty
        if (!inputValue) {
            setIsExpanded(false);
        }
    };

    /**
     * Handle clear button click
     * Clears input and focuses back on the search input
     */
    const handleClear = () => {
        setInputValue('');

        if (onChange) {
            onChange('');
        }

        if (inputRef.current) {
            inputRef.current.focus();
        }
    };

    /**
     * Handle search icon click
     * Expands the search bar and focuses on the input
     */
    const handleSearchIconClick = () => {
        setIsExpanded(true);

        if (inputRef.current) {
            inputRef.current.focus();
        }
    };

    /**
     * Setup keyboard shortcut (Ctrl+F) to focus on search
     */
    useEffect(() => {
        const handleKeyDown = (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
                e.preventDefault();
                setIsExpanded(true);

                if (inputRef.current) {
                    inputRef.current.focus();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, []);

    /**
     * Handle click outside to collapse search bar
     */
    useEffect(() => {
        const handleClickOutside = (e) => {
            if (searchBarRef.current && !searchBarRef.current.contains(e.target) && !inputValue) {
                setIsExpanded(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [inputValue]);

    return (
        <div
            ref={searchBarRef}
            className={`search-bar ${isExpanded ? 'expanded' : ''} ${isFocused ? 'focused' : ''}`}
        >
            <button
                className="search-icon"
                onClick={handleSearchIconClick}
                aria-label="Search"
            >
                <span className="icon icon-search"></span>
            </button>

            <form onSubmit={handleSubmit} className="search-form">
                <input
                    ref={inputRef}
                    type="text"
                    className="search-input"
                    placeholder={placeholder}
                    value={inputValue}
                    onChange={handleChange}
                    onFocus={handleFocus}
                    onBlur={handleBlur}
                    aria-label="Search input"
                />

                {inputValue && (
                    <button
                        type="button"
                        className="clear-button"
                        onClick={handleClear}
                        aria-label="Clear search"
                    >
                        <span className="icon icon-x"></span>
                    </button>
                )}
            </form>
        </div>
    );
};

export default SearchBar;