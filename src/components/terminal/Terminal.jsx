import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useSettings } from '../../providers/SettingsProvider';
import './terminal.css';

const Terminal = ({ isOpen, onToggle }) => {
    const [commandHistory, setCommandHistory] = useState([]);
    const [currentCommand, setCurrentCommand] = useState('');
    const [historyIndex, setHistoryIndex] = useState(-1);
    const [isExecuting, setIsExecuting] = useState(false);
    const inputRef = useRef(null);
    const terminalRef = useRef(null);
    const { currentPath } = useHistory();
    const { settings } = useSettings();

    const terminalHeight = settings.terminalHeight || 240;

    // Initialize with welcome message
    useEffect(() => {
        if (commandHistory.length === 0) {
            const welcomeMessage = {
                type: 'system',
                content: `Fast File Explorer Terminal
Current directory: ${currentPath || '/'}
Type 'help' to see available commands.`,
                timestamp: new Date().toLocaleTimeString(),
            };
            setCommandHistory([welcomeMessage]);
        }
    }, []);

    // Update terminal when path changes
    useEffect(() => {
        if (currentPath) {
            const pathChangeMessage = {
                type: 'system',
                content: `Changed directory to: ${currentPath}`,
                timestamp: new Date().toLocaleTimeString(),
            };
            setCommandHistory(prev => [...prev, pathChangeMessage]);
        }
    }, [currentPath]);

    // Focus input when terminal opens
    useEffect(() => {
        if (isOpen && inputRef.current) {
            inputRef.current.focus();
        }
    }, [isOpen]);

    // Scroll to bottom when command history changes
    useEffect(() => {
        if (terminalRef.current) {
            terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
        }
    }, [commandHistory]);

    // Get terminal prompt
    const getPrompt = () => {
        const username = 'user';
        const hostname = 'localhost';
        const pathDisplay = currentPath || '/';
        return `${username}@${hostname}:${pathDisplay}$`;
    };

    // Execute command
    const executeCommand = async (command) => {
        setIsExecuting(true);

        try {
            const output = await invoke('execute_command', { command });
            return {
                type: 'output',
                content: output || 'Command completed successfully.',
            };
        } catch (error) {
            return {
                type: 'error',
                content: `Error: ${error.message || error}`,
            };
        } finally {
            setIsExecuting(false);
        }
    };

    // Handle built-in commands
    const handleBuiltinCommand = async (command, args) => {
        switch (command) {
            case 'help':
                return {
                    type: 'output',
                    content: `Available commands:
  help                    - Show this help message
  clear                   - Clear the terminal
  ls, dir                 - List directory contents
  pwd                     - Print working directory
  cd <path>               - Change directory
  echo <text>             - Print text
  mkdir <name>            - Create directory
  touch <name>            - Create file
  cat <file>              - Display file contents
  tree                    - Show directory tree
  whoami                  - Show current user
  date                    - Show current date and time
  exit                    - Close the terminal
  
  You can also run system commands directly.`,
                };

            case 'clear':
                setCommandHistory([]);
                return null;

            case 'pwd':
                return {
                    type: 'output',
                    content: currentPath || '/',
                };

            case 'whoami':
                return {
                    type: 'output',
                    content: 'user',
                };

            case 'date':
                return {
                    type: 'output',
                    content: new Date().toString(),
                };

            case 'ls':
            case 'dir':
                try {
                    const dirContent = await invoke('open_directory', { path: currentPath });
                    const data = JSON.parse(dirContent);
                    const items = [
                        ...data.directories.map(dir => `${dir.name}/`),
                        ...data.files.map(file => file.name)
                    ];
                    return {
                        type: 'output',
                        content: items.length > 0 ? items.join('  ') : 'Directory is empty',
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `Cannot list directory: ${error.message || error}`,
                    };
                }

            case 'tree':
                // Simple tree implementation
                try {
                    const dirContent = await invoke('open_directory', { path: currentPath });
                    const data = JSON.parse(dirContent);
                    let tree = `${currentPath}\n`;

                    data.directories.forEach((dir, index) => {
                        const isLast = index === data.directories.length - 1 && data.files.length === 0;
                        tree += `${isLast ? '└── ' : '├── '}${dir.name}/\n`;
                    });

                    data.files.forEach((file, index) => {
                        const isLast = index === data.files.length - 1;
                        tree += `${isLast ? '└── ' : '├── '}${file.name}\n`;
                    });

                    return {
                        type: 'output',
                        content: tree,
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `Cannot generate tree: ${error.message || error}`,
                    };
                }

            case 'echo':
                return {
                    type: 'output',
                    content: args.join(' '),
                };

            case 'mkdir':
                if (args.length === 0) {
                    return {
                        type: 'error',
                        content: 'mkdir: missing operand',
                    };
                }
                try {
                    await invoke('create_directory', {
                        folder_path_abs: currentPath,
                        directory_name: args[0]
                    });
                    return {
                        type: 'output',
                        content: `Directory '${args[0]}' created successfully.`,
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `mkdir: ${error.message || error}`,
                    };
                }

            case 'touch':
                if (args.length === 0) {
                    return {
                        type: 'error',
                        content: 'touch: missing operand',
                    };
                }
                try {
                    await invoke('create_file', {
                        folder_path_abs: currentPath,
                        file_name: args[0]
                    });
                    return {
                        type: 'output',
                        content: `File '${args[0]}' created successfully.`,
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `touch: ${error.message || error}`,
                    };
                }

            case 'cat':
                if (args.length === 0) {
                    return {
                        type: 'error',
                        content: 'cat: missing operand',
                    };
                }
                try {
                    const filePath = `${currentPath}/${args[0]}`;
                    const content = await invoke('open_file', { file_path: filePath });
                    return {
                        type: 'output',
                        content: content,
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `cat: ${error.message || error}`,
                    };
                }

            case 'exit':
                onToggle();
                return {
                    type: 'system',
                    content: 'Terminal closed.',
                };

            default:
                return null; // Not a built-in command
        }
    };

    // Handle command submission
    const handleSubmit = async (e) => {
        e.preventDefault();

        if (!currentCommand.trim() || isExecuting) return;

        const command = currentCommand.trim();
        const [cmd, ...args] = command.split(' ');

        // Add command to history
        const commandEntry = {
            type: 'command',
            prompt: getPrompt(),
            content: command,
            timestamp: new Date().toLocaleTimeString(),
        };

        setCommandHistory(prev => [...prev, commandEntry]);

        // Process command
        let response;

        // Check for built-in commands first
        response = await handleBuiltinCommand(cmd.toLowerCase(), args);

        // If not a built-in command, execute as system command
        if (response === null && cmd.toLowerCase() !== 'clear') {
            response = await executeCommand(command);
        }

        // Add response to history
        if (response) {
            setCommandHistory(prev => [...prev, {
                ...response,
                timestamp: new Date().toLocaleTimeString(),
            }]);
        }

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
        if (isExecuting) return;

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
        } else if (e.key === 'Tab') {
            e.preventDefault();
            // Simple auto-completion for common commands
            const commonCommands = ['help', 'clear', 'ls', 'dir', 'pwd', 'cd', 'mkdir', 'touch', 'cat', 'tree', 'echo'];
            const matches = commonCommands.filter(cmd => cmd.startsWith(currentCommand.toLowerCase()));

            if (matches.length === 1) {
                setCurrentCommand(matches[0] + ' ');
            }
        }
    };

    if (!isOpen) return null;

    return (
        <div className="enhanced-terminal" style={{ height: `${terminalHeight}px` }}>
            <div className="terminal-header">
                <div className="terminal-title">
                    <span className="icon icon-terminal"></span>
                    <span>Terminal - {currentPath || '/'}</span>
                </div>
                <div className="terminal-controls">
                    <button
                        className="terminal-control"
                        onClick={() => setCommandHistory([])}
                        title="Clear terminal"
                    >
                        <span className="icon icon-clear"></span>
                    </button>
                    <button
                        className="terminal-control"
                        onClick={onToggle}
                        title="Close terminal"
                    >
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
                        <div className="terminal-entry">
                            {entry.type === 'command' && (
                                <span className="terminal-prompt">{entry.prompt} </span>
                            )}
                            <pre className="terminal-text">{entry.content}</pre>
                            {entry.timestamp && (
                                <span className="terminal-timestamp">{entry.timestamp}</span>
                            )}
                        </div>
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
                        disabled={isExecuting}
                        autoFocus
                        spellCheck="false"
                        autoComplete="off"
                        autoCapitalize="off"
                    />
                    {isExecuting && (
                        <span className="terminal-executing">
                            <span className="spinner-small"></span>
                        </span>
                    )}
                </form>
            </div>
        </div>
    );
};

export default Terminal;