import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';

// Create history context
const HistoryContext = createContext({
    history: [],
    currentIndex: -1,
    currentPath: null,
    canGoBack: false,
    canGoForward: false,
    navigateTo: () => {},
    goBack: () => {},
    goForward: () => {},
});

export default function HistoryProvider({ children }) {
    // State for navigation history
    const [history, setHistory] = useState([]);
    const [currentIndex, setCurrentIndex] = useState(-1);

    // Derived state
    const currentPath = history[currentIndex] || null;
    const canGoBack = currentIndex > 0;
    const canGoForward = currentIndex < history.length - 1;

    // Load history from session storage on mount
    useEffect(() => {
        try {
            const savedHistory = JSON.parse(sessionStorage.getItem('fileExplorerHistory') || '[]');
            const savedIndex = parseInt(sessionStorage.getItem('fileExplorerHistoryIndex') || '-1');

            if (savedHistory.length) {
                setHistory(savedHistory);
                setCurrentIndex(savedIndex >= 0 ? savedIndex : 0);
            }
        } catch (error) {
            console.error('Failed to load navigation history:', error);
        }
    }, []);

    // Save history to session storage when it changes
    useEffect(() => {
        if (history.length) {
            sessionStorage.setItem('fileExplorerHistory', JSON.stringify(history));
            sessionStorage.setItem('fileExplorerHistoryIndex', currentIndex.toString());
        }
    }, [history, currentIndex]);

    // Navigate to a specific path
    const navigateTo = useCallback((path) => {
        if (path === currentPath) return;

        setHistory(prevHistory => {
            // If we're not at the end of the history, trim the forward history
            const newHistory = prevHistory.slice(0, currentIndex + 1);

            // Add the new path to history
            newHistory.push(path);

            // Set current index to the new end of history
            setCurrentIndex(newHistory.length - 1);

            return newHistory;
        });
    }, [currentIndex, currentPath]);

    // Navigate back in history
    const goBack = useCallback(() => {
        if (!canGoBack) return;
        setCurrentIndex(prevIndex => prevIndex - 1);
    }, [canGoBack]);

    // Navigate forward in history
    const goForward = useCallback(() => {
        if (!canGoForward) return;
        setCurrentIndex(prevIndex => prevIndex + 1);
    }, [canGoForward]);

    const contextValue = {
        history,
        currentIndex,
        currentPath,
        canGoBack,
        canGoForward,
        navigateTo,
        goBack,
        goForward,
    };

    return (
        <HistoryContext.Provider value={contextValue}>
            {children}
        </HistoryContext.Provider>
    );
}

// Custom hook for using the history context
export const useHistory = () => useContext(HistoryContext);