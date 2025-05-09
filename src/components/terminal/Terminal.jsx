import React, { useState, useRef, useEffect } from 'react';
import { useHistory } from '../../providers/HistoryProvider';
import './terminal.css';

const Terminal = () => {
    const [commandHistory, setCommandHistory] = useState([]);
    const [currentCommand, setCurrentCommand] = useState('');
    const [historyIndex, setHistoryIndex] = useState(-1);
    const inputRef = useRef(null);
    const terminalRef = useRef(null);
    const { currentPath } = useHistory();

    // Initial welcome message
    useEffect(() => {
        const welcomeMessage = {
            type: 'system',
            content: `Welcome to File Explorer Terminal
Type 'help' to see available commands.`,
        };

        setCommandHistory([welcomeMessage]);
    }, []);

    // Focus input on mount
    useEffect(() => {
        if (inputRef.current) {
            inputRef.current.focus();
        }
    }, []);

    // Scroll to bottom when command history changes
    useEffect(() => {
        if (terminalRef.current) {
            terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
        }
    }, [commandHistory]);

    // Get terminal prompt
    const getPrompt = () => {
        // Extract username and hostname for prompt (in a real implementation, these would come from system info)
        const username = 'user';
        const hostname = 'localhost';

        // Format current path for display
        const pathDisplay = currentPath || '/';

        return `${username}@${hostname}:${pathDisplay}$`;
    };

    // Handle command submission
    const handleSubmit = (e) => {
        e.preventDefault();

        if (!currentCommand.trim()) return;

        // Add command to history
        const commandEntry = {
            type: 'command',
            prompt: getPrompt(),
            content: currentCommand,
        };

        // Process command (in a real implementation, this would interface with the backend)
        let response;

        const command = currentCommand.trim().toLowerCase();
        const args = command.split(' ').slice(1);

        switch (command.split(' ')[0]) {
            case 'help':
                response = {
                    type: 'output',
                    content: `Available commands:
  help - Display this help message
  clear - Clear the terminal
  ls - List files and directories
  pwd - Print working directory
  cd [directory] - Change directory
  echo [text] - Print text
  exit - Close the terminal`,
                };
                break;

            case 'clear':
                // Clear all but the welcome message
                setCommandHistory([commandHistory[0]]);
                setCurrentCommand('');
                setHistoryIndex(-1);
                return;

            case 'pwd':
                response = {
                    type: 'output',
                    content: currentPath || '/',
                };
                break;

            case 'ls':
                // In a real implementation, this would show actual directory contents
                response = {
                    type: 'output',
                    content: 'directory1/ directory2/ file1.txt file2.txt',
                };
                break;

            case 'cd':
                // In a real implementation, this would change the current directory
                if (args.length === 0) {
                    response = {
                        type: 'error',
                        content: 'cd: missing argument',
                    };
                } else {
                    response = {
                        type: 'output',
                        content: `Changed directory to ${args[0]}`,
                    };
                }
                break;

            case 'echo':
                response = {
                    type: 'output',
                    content: args.join(' '),
                };
                break;

            case 'exit':
                // In a real implementation, this would close the terminal
                response = {
                    type: 'system',
                    content: 'Closing terminal...',
                };
                break;

            default:
                response = {
                    type: 'error',
                    content: `Command not found: ${command.split(' ')[0]}`,
                };
        }

        // Update command history
        setCommandHistory(prev => [...prev, commandEntry, response]);

        // Reset command input
        setCurrentCommand('');
        setHistoryIndex(-1);
    };

    // Handle input changes
    const handleChange = (e) => {
        setCurrentCommand(e.target.value);
    };

    // Handle keyboard navigation through command history
    const handleKeyDown = (e) => {
        if (e.key === 'ArrowUp') {
            e.preventDefault();

            const commandEntries = commandHistory.filter(entry => entry.type === 'command');

            if (commandEntries.length > 0) {
                const newIndex = historyIndex < commandEntries.length - 1
                    ? historyIndex + 1
                    : historyIndex;

                if (newIndex >= 0) {
                    setCurrentCommand(commandEntries[commandEntries.length - 1 - newIndex].content);
                    setHistoryIndex(newIndex);
                }
            }
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();

            if (historyIndex > 0) {
                const commandEntries = commandHistory.filter(entry => entry.type === 'command');
                const newIndex = historyIndex - 1;

                setCurrentCommand(commandEntries[commandEntries.length - 1 - newIndex].content);
                setHistoryIndex(newIndex);
            } else if (historyIndex === 0) {
                setCurrentCommand('');
                setHistoryIndex(-1);
            }
        }
    };

    return (
        <div className="terminal-container">
            <div className="terminal-header">
                <span className="terminal-title">Terminal</span>
                <div className="terminal-controls">
                    <button className="terminal-control" aria-label="Minimize">
                        <span className="icon icon-minus"></span>
                    </button>
                    <button className="terminal-control" aria-label="Close">
                        <span className="icon icon-x"></span>
                    </button>
                </div>
            </div>

            <div className="terminal-content" ref={terminalRef}>
                {commandHistory.map((entry, index) => (
                    <div
                        key={index}
                        className={`terminal-line terminal-${entry.type}`}
                    >
                        {entry.type === 'command' && (
                            <span className="terminal-prompt">{entry.prompt} </span>
                        )}
                        <span className="terminal-text">{entry.content}</span>
                    </div>
                ))}

                <form onSubmit={handleSubmit} className="terminal-input-line">
                    <span className="terminal-prompt">{getPrompt()} </span>
                    <input
                        ref={inputRef}
                        type="text"
                        className="terminal-input"
                        value={currentCommand}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        autoFocus
                        spellCheck="false"
                        autoComplete="off"
                        autoCapitalize="off"
                    />
                </form>
            </div>
        </div>
    );
};

export default Terminal;