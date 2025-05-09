import React from 'react';
import ThemeProvider from './providers/ThemeProvider';
import AppStateProvider from './providers/AppStateProvider';
import HistoryProvider from './providers/HistoryProvider';
import FileSystemProvider from './providers/FileSystemProvider';
import SettingsProvider from './providers/SettingsProvider';
import ContextMenuProvider from './providers/ContextMenuProvider';
import MainLayout from './layouts/MainLayout';

// Simple fallback for error cases
function ErrorFallback() {
    return (
        <div style={{
            padding: '20px',
            color: '#333',
            backgroundColor: '#f8f8f8',
            fontFamily: 'system-ui, sans-serif',
            maxWidth: '800px',
            margin: '40px auto',
            border: '1px solid #ddd',
            borderRadius: '8px',
            boxShadow: '0 2px 8px rgba(0,0,0,0.1)'
        }}>
            <h1 style={{ color: '#d32f2f' }}>Fast File Explorer</h1>
            <p>The application could not be loaded properly. Try refreshing the page.</p>
            <p>If the problem persists, check the console (F12) for error messages.</p>
            <button
                onClick={() => window.location.reload()}
                style={{
                    padding: '8px 16px',
                    backgroundColor: '#0078d4',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    marginTop: '15px'
                }}
            >
                Reload Page
            </button>
        </div>
    );
}

// App component with error boundary
class App extends React.Component {
    constructor(props) {
        super(props);
        this.state = { hasError: false };
    }

    // Error Boundary
    static getDerivedStateFromError(error) {
        return { hasError: true };
    }

    componentDidCatch(error, errorInfo) {
        console.error("Application error:", error, errorInfo);
    }

    render() {
        // Show fallback in case of error
        if (this.state.hasError) {
            return <ErrorFallback />;
        }

        // Render normal application with all providers
        // The order of providers is important:
        // - ThemeProvider should be outermost as other components may depend on theme variables
        // - HistoryProvider should come before FileSystemProvider since navigation depends on history
        // - ContextMenuProvider should come after FileSystemProvider to access selected items
        return (
            <div className="app-container">
                <ThemeProvider>
                    <AppStateProvider>
                        <HistoryProvider>
                            <FileSystemProvider>
                                <SettingsProvider>
                                    <ContextMenuProvider>
                                        <MainLayout />
                                    </ContextMenuProvider>
                                </SettingsProvider>
                            </FileSystemProvider>
                        </HistoryProvider>
                    </AppStateProvider>
                </ThemeProvider>
            </div>
        );
    }
}

export default App;