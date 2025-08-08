import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useSettings } from '../../providers/SettingsProvider';
import './terminal.css';

/**
 * Terminal component - Provides a command-line interface within the application
 * Supports built-in commands and passes through system commands
 *
 * @param {Object} props - Component props
 * @param {boolean} props.isOpen - Whether the terminal is currently open
 * @param {Function} props.onToggle - Callback function to toggle terminal visibility
 * @returns {React.ReactElement} Terminal component
 */
const Terminal = ({ isOpen, onToggle }) => {
    const [commandHistory, setCommandHistory] = useState([]);
    const [currentCommand, setCurrentCommand] = useState('');
    const [historyIndex, setHistoryIndex] = useState(-1);
    const [isExecuting, setIsExecuting] = useState(false);
    const [abortController, setAbortController] = useState(null);
    const inputRef = useRef(null);
    const terminalRef = useRef(null);
    const { currentPath } = useHistory();
    const { settings } = useSettings();

    /**
     * Get terminal height from settings with fallback to default value
     * @type {number}
     */
    const terminalHeight = settings.terminal_height || 240;

    /**
     * Initialize terminal with welcome message on first render
     */
    useEffect(() => {
        if (commandHistory.length === 0) {
            const welcomeMessage = {
                type: 'system',
                content: `File Explorer Terminal
Current directory: ${currentPath || '/'}
Type 'help' to see available commands.`,
                timestamp: new Date().toLocaleTimeString(),
            };
            setCommandHistory([welcomeMessage]);
        }
    }, []);

    /**
     * Update terminal output when current path changes
     * Adds a notification in the terminal about the directory change
     */
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

    /**
     * Focus input field when terminal opens
     */
    useEffect(() => {
        if (isOpen && inputRef.current) {
            inputRef.current.focus();
        }
    }, [isOpen]);

    /**
     * Automatically scroll to the bottom when command history updates
     */
    useEffect(() => {
        if (terminalRef.current) {
            // Ensure proper scrolling without pushing content off screen
            terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
        }
    }, [commandHistory]);

    /**
     * Cleanup running commands when component unmounts
     */
    useEffect(() => {
        return () => {
            if (abortController) {
                abortController.abort();
            }
        };
    }, [abortController]);

    /**
     * Generates the terminal prompt string with username, hostname and current path
     * @returns {string} Formatted prompt string
     */
    const getPrompt = () => {
        const username = 'user';
        const hostname = 'localhost';
        const pathDisplay = currentPath || '/';
        return `${username}@${hostname}:${pathDisplay}$`;
    };

    /**
     * Executes a command using the Tauri backend
     * Handles response parsing and error formatting
     * Supports command cancellation via AbortController
     *
     * @param {string} command - The command to execute
     * @returns {Object} Result object with type and content
     * @async
     */
    const executeCommand = async (command) => {
        const controller = new AbortController();
        setAbortController(controller);
        setIsExecuting(true);

        try {
            // Create a race between the command execution and cancellation
            const commandPromise = invoke('execute_command', { command });
            const cancelPromise = new Promise((_, reject) => {
                controller.signal.addEventListener('abort', () => {
                    reject(new Error('Command cancelled'));
                });
            });

            const output = await Promise.race([commandPromise, cancelPromise]);

            // Format the response properly
            if (typeof output === 'string') {
                try {
                    // Check if it's JSON
                    const parsedOutput = JSON.parse(output);

                    // Handle empty results
                    if (parsedOutput.stdout === "" && parsedOutput.stderr === "" && parsedOutput.status === 0) {
                        return { type: 'output', content: '' };
                    }

                    // Handle status 1 with empty stdout/stderr (command not found)
                    if (parsedOutput.status === 1) {
                        if (parsedOutput.stdout === "" && parsedOutput.stderr === "") {
                            return { type: 'error', content: `Command not found: ${command.split(' ')[0]}` };
                        } else if (parsedOutput.stderr) {
                            return { type: 'error', content: parsedOutput.stderr.trim() };
                        }
                    }

                    // Handle actual output
                    if (parsedOutput.stdout && parsedOutput.stdout.trim()) {
                        return { type: 'output', content: parsedOutput.stdout.trim() };
                    }

                    // Handle error output
                    if (parsedOutput.stderr && parsedOutput.stderr.trim()) {
                        return { type: 'error', content: parsedOutput.stderr.trim() };
                    }

                    // Handle other output based on status
                    return {
                        type: parsedOutput.status === 0 ? 'output' : 'error',
                        content: parsedOutput.status === 0 ?
                            (output || '') :
                            `Command failed with status ${parsedOutput.status}`
                    };
                } catch (e) {
                    // Not JSON, return as is
                    return { type: 'output', content: output || '' };
                }
            } else if (typeof output === 'object') {
                // Handle object response
                if (output.stdout === "" && output.stderr === "" && output.status === 0) {
                    return { type: 'output', content: '' };
                }

                // Handle command not found
                if (output.status === 1) {
                    if (output.stdout === "" && output.stderr === "") {
                        return { type: 'error', content: `Command not found: ${command.split(' ')[0]}` };
                    } else if (output.stderr) {
                        return { type: 'error', content: output.stderr };
                    }
                }

                if (output.stdout) {
                    return { type: 'output', content: output.stdout };
                }

                // Empty response
                return {
                    type: output.status === 0 ? 'output' : 'error',
                    content: output.status === 0 ? '' : `Command failed with status ${output.status}`
                };
            }

            return {
                type: 'output',
                content: output || '',
            };
        } catch (error) {
            // Handle command cancellation
            if (error.message === 'Command cancelled') {
                return {
                    type: 'system',
                    content: 'Command cancelled by user',
                };
            }

            // Format error messages properly
            let errorMessage = '';

            try {
                if (typeof error === 'string') {
                    try {
                        // Try to parse JSON error
                        const parsedError = JSON.parse(error);
                        errorMessage = parsedError.custom_message ||
                            parsedError.message_from_code ||
                            parsedError.message ||
                            error;
                    } catch {
                        errorMessage = error;
                    }
                } else if (typeof error === 'object') {
                    errorMessage = error.custom_message ||
                        error.message_from_code ||
                        error.message ||
                        JSON.stringify(error);
                } else {
                    errorMessage = String(error);
                }
            } catch {
                errorMessage = String(error);
            }

            return {
                type: 'error',
                content: `Error: ${errorMessage}`,
            };
        } finally {
            setIsExecuting(false);
            setAbortController(null);
        }
    };

    /**
     * Handles built-in terminal commands
     * Provides functionality for commands like help, clear, ls, etc.
     *
     * @param {string} command - The command to handle
     * @param {Array<string>} args - Command arguments
     * @returns {Object|null} Command result object or null if not a built-in command
     * @async
     */
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
  
  Keyboard shortcuts:
  Ctrl+C                  - Interrupt running command or clear input
  ↑/↓                     - Navigate command history
  Tab                     - Auto-complete commands
  
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

    /**
     * Handles form submission for the terminal input
     * Processes the command and displays the result
     *
     * @param {React.FormEvent} e - Form submit event
     * @async
     */
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

        // Add response to history if there's content or it's a system message
        if (response && (response.content || response.type === 'system')) {
            setCommandHistory(prev => [...prev, {
                ...response,
                timestamp: new Date().toLocaleTimeString(),
            }]);
        }

        // Reset command input
        setCurrentCommand('');
        setHistoryIndex(-1);
    };

    /**
     * Handles input changes for the terminal command
     * @param {React.ChangeEvent<HTMLInputElement>} e - Input change event
     */
    const handleChange = (e) => {
        setCurrentCommand(e.target.value);
    };

    /**
     * Handles keyboard navigation through command history and tab completion
     * Supports arrow up/down for history navigation and tab for command completion
     * Also handles Ctrl+C for command interruption
     *
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleKeyDown = (e) => {
        // Handle Ctrl+C to interrupt command execution or clear current input
        if (e.ctrlKey && e.key === 'c') {
            e.preventDefault();
            
            if (isExecuting && abortController) {
                // Interrupt running command
                abortController.abort();
                setIsExecuting(false);
                setAbortController(null);
                setCommandHistory(prev => [...prev, {
                    type: 'system',
                    content: '^C - Command interrupted',
                    timestamp: new Date().toLocaleTimeString(),
                }]);
                return;
            } else if (currentCommand.trim()) {
                // Clear current input and show ^C
                const commandEntry = {
                    type: 'command',
                    prompt: getPrompt(),
                    content: currentCommand + '^C',
                    timestamp: new Date().toLocaleTimeString(),
                };
                setCommandHistory(prev => [...prev, commandEntry]);
                setCurrentCommand('');
                setHistoryIndex(-1);
                return;
            }
        }

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
                        <span className="icon icon-delete"></span>
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
                            <span className="terminal-interrupt-hint">Press Ctrl+C to interrupt</span>
                        </span>
                    )}
                </form>
            </div>
        </div>
    );
};

export default Terminal;

