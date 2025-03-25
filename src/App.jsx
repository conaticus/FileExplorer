import React from 'react';
import ThemeProvider from './providers/ThemeProvider';
import AppStateProvider from './providers/AppStateProvider';
import FileSystemProvider from './providers/FileSystemProvider';
import MainLayout from './layouts/MainLayout';

// Definiere ein sehr einfaches Fallback für Fehlerfälle
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
            <p>Die Anwendung konnte nicht richtig geladen werden. Versuchen Sie, die Seite neu zu laden.</p>
            <p>Falls das Problem weiterhin besteht, überprüfen Sie die Konsole (F12) auf Fehlermeldungen.</p>
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
                Seite neu laden
            </button>
        </div>
    );
}

// Minimal App-Komponente
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
        console.error("Fehler in der Anwendung:", error, errorInfo);
    }

    render() {
        // Im Fehlerfall das Fallback anzeigen
        if (this.state.hasError) {
            return <ErrorFallback />;
        }

        // Normale Anwendung rendern
        return (
            <div className="app-container" style={{ width: '100%', height: '100vh' }}>
                <ThemeProvider>
                    <AppStateProvider>
                        <FileSystemProvider>
                            <MainLayout />
                        </FileSystemProvider>
                    </AppStateProvider>
                </ThemeProvider>
            </div>
        );
    }
}

export default App;