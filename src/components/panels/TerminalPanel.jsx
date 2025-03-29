import React, { useState, useEffect, useRef } from 'react';

const TerminalPanel = ({ isOpen, currentPath, onClose }) => {
    const [commandHistory, setCommandHistory] = useState([]);
    const [currentCommand, setCurrentCommand] = useState('');
    const [historyIndex, setHistoryIndex] = useState(-1);
    const inputRef = useRef(null);
    const terminalRef = useRef(null);

    // Beim Öffnen des Terminals wird der Fokus auf die Eingabe gesetzt
    useEffect(() => {
        if (isOpen && inputRef.current) {
            inputRef.current.focus();
        }
    }, [isOpen]);

    // Beim Ändern des Pfades wird ein neuer Pfad im Terminal angezeigt
    useEffect(() => {
        if (isOpen && currentPath) {
            addOutput(`Verzeichnis gewechselt zu: ${currentPath}`, 'system');
        }
    }, [currentPath, isOpen]);

    // Scroll zum Ende des Terminals, wenn neue Ausgabe hinzugefügt wird
    useEffect(() => {
        if (terminalRef.current) {
            terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
        }
    }, [commandHistory]);

    // Füge Ausgabe zum Terminal hinzu
    const addOutput = (text, type = 'output') => {
        setCommandHistory(prev => [...prev, { text, type }]);
    };

    // Behandelt die Eingabe eines Befehls
    const handleCommandSubmit = (e) => {
        e.preventDefault();

        if (!currentCommand.trim()) return;

        // Füge den Befehl zur Historie hinzu
        addOutput(`${currentPath}> ${currentCommand}`, 'input');

        // [Backend Integration] - Befehl im Backend ausführen
        // /* BACKEND_INTEGRATION: Befehl ausführen */

        // Simuliere eine Antwort
        executeCommand(currentCommand);

        // Setze den Befehl zurück und aktualisiere den Historien-Index
        setCommandHistory(prev => [...prev, { text: currentCommand, type: 'command' }]);
        setCurrentCommand('');
        setHistoryIndex(-1);
    };

    // Navigiere durch die Befehlshistorie mit den Pfeiltasten
    const handleKeyDown = (e) => {
        const commands = commandHistory
            .filter(item => item.type === 'command')
            .map(item => item.text);

        if (e.key === 'ArrowUp' && historyIndex < commands.length - 1) {
            e.preventDefault();
            const newIndex = historyIndex + 1;
            setHistoryIndex(newIndex);
            setCurrentCommand(commands[commands.length - 1 - newIndex]);
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();
            if (historyIndex > 0) {
                const newIndex = historyIndex - 1;
                setHistoryIndex(newIndex);
                setCurrentCommand(commands[commands.length - 1 - newIndex]);
            } else {
                setHistoryIndex(-1);
                setCurrentCommand('');
            }
        } else if (e.key === 'Tab') {
            e.preventDefault();
            // [Backend Integration] - Auto-Vervollständigung im Backend
            // /* BACKEND_INTEGRATION: Befehl vervollständigen */

            // Simuliere eine Auto-Vervollständigung
            if (currentCommand.startsWith('cd ')) {
                setCurrentCommand('cd /Beispielpfad');
            } else if (currentCommand.startsWith('mkdir ')) {
                setCurrentCommand('mkdir neuer_ordner');
            }
        }
    };

    // Simuliere die Ausführung eines Befehls (für Demonstrationszwecke)
    const executeCommand = (command) => {
        const cmd = command.trim().toLowerCase();
        const args = cmd.split(' ').slice(1).join(' ');
        const baseCmd = cmd.split(' ')[0];

        switch (baseCmd) {
            case 'help':
                addOutput('Verfügbare Befehle: help, cd, ls, mkdir, touch, echo, clear, exit');
                break;
            case 'cd':
                addOutput(`Wechsle zu Verzeichnis: ${args || '/'}`);
                break;
            case 'ls':
                addOutput('Verzeichnisinhalt:');
                addOutput('Ordner1/');
                addOutput('Ordner2/');
                addOutput('datei1.txt');
                addOutput('datei2.docx');
                break;
            case 'mkdir':
                if (!args) {
                    addOutput('Fehler: Ordnername fehlt', 'error');
                } else {
                    addOutput(`Ordner erstellt: ${args}`);
                }
                break;
            case 'touch':
                if (!args) {
                    addOutput('Fehler: Dateiname fehlt', 'error');
                } else {
                    addOutput(`Datei erstellt: ${args}`);
                }
                break;
            case 'echo':
                addOutput(args || '');
                break;
            case 'clear':
                setCommandHistory([]);
                break;
            case 'exit':
                onClose();
                break;
            default:
                addOutput(`Befehl nicht gefunden: ${baseCmd}`, 'error');
        }
    };

    // Terminal-Ausgabe rendern
    const renderOutput = () => {
        return commandHistory.map((item, index) => (
            <div key={index} className={`terminal-line terminal-${item.type}`}>
                {item.text}
            </div>
        ));
    };

    if (!isOpen) return null;

    return (
        <div className="terminal-panel">
            <div className="terminal-header">
                <div className="terminal-title">Terminal</div>
                <div className="terminal-controls">
                    <button
                        className="terminal-clear-button"
                        onClick={() => setCommandHistory([])}
                        title="Terminal leeren"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            width="14"
                            height="14"
                        >
                            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                        </svg>
                    </button>
                    <button
                        className="terminal-close-button"
                        onClick={onClose}
                        title="Terminal schließen"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            width="14"
                            height="14"
                        >
                            <path d="M18 6L6 18M6 6l12 12" />
                        </svg>
                    </button>
                </div>
            </div>

            <div className="terminal-content" ref={terminalRef}>
                <div className="terminal-welcome">
                    <div className="terminal-line terminal-system">Fast File Explorer Terminal</div>
                    <div className="terminal-line terminal-system">Für Hilfe, geben Sie 'help' ein.</div>
                    <div className="terminal-line terminal-system">Aktuelles Verzeichnis: {currentPath}</div>
                </div>

                {renderOutput()}

                <form className="terminal-input-line" onSubmit={handleCommandSubmit}>
                    <span className="terminal-prompt">{currentPath}&gt; </span>
                    <input
                        type="text"
                        className="terminal-input"
                        value={currentCommand}
                        onChange={(e) => setCurrentCommand(e.target.value)}
                        onKeyDown={handleKeyDown}
                        ref={inputRef}
                        autoFocus
                    />
                </form>
            </div>
        </div>
    );
};

export default TerminalPanel;