import React, { useState, useEffect, useRef } from 'react';
import './searchBar.css';

const SearchBar = ({ value = '', onChange, onSubmit, placeholder = 'Search files and folders' }) => {
    const [inputValue, setInputValue] = useState(value);
    const [isFocused, setIsFocused] = useState(false);
    const [isExpanded, setIsExpanded] = useState(false);
    const inputRef = useRef(null);
    const searchBarRef = useRef(null);

    // Update local state when props change
    useEffect(() => {
        setInputValue(value);
    }, [value]);

    // Handle input change
    const handleChange = (e) => {
        const newValue = e.target.value;
        setInputValue(newValue);

        if (onChange) {
            onChange(newValue);
        }
    };

    // Handle form submission
    const handleSubmit = (e) => {
        e.preventDefault();

        if (onSubmit) {
            onSubmit(inputValue);
        }
    };

    // Handle focus
    const handleFocus = () => {
        setIsFocused(true);
        setIsExpanded(true);
    };

    // Handle blur
    const handleBlur = () => {
        setIsFocused(false);

        // Only collapse if empty
        if (!inputValue) {
            setIsExpanded(false);
        }
    };

    // Handle clear button
    const handleClear = () => {
        setInputValue('');

        if (onChange) {
            onChange('');
        }

        if (inputRef.current) {
            inputRef.current.focus();
        }
    };

    // Handle click on search icon
    const handleSearchIconClick = () => {
        setIsExpanded(true);

        if (inputRef.current) {
            inputRef.current.focus();
        }
    };

    // Handle keyboard shortcut (Ctrl+F)
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

    // Handle click outside to collapse
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