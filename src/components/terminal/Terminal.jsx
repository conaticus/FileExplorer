import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistory } from '../../providers/HistoryProvider';
import { useFileSystem } from '../../providers/FileSystemProvider';
import { useSettings } from '../../providers/SettingsProvider';
import Icon from '../common/Icon';
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
    const [persistentHistory, setPersistentHistory] = useState([]);
    const [isExecuting, setIsExecuting] = useState(false);
    const [abortController, setAbortController] = useState(null);
    const [isSearchMode, setIsSearchMode] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [searchResults, setSearchResults] = useState([]);
    const [selectedSearchIndex, setSelectedSearchIndex] = useState(-1);
    const inputRef = useRef(null);
    const terminalRef = useRef(null);
    const { currentPath, navigateTo } = useHistory();
    const { setCurrentPath } = useFileSystem();
    const { settings } = useSettings();

    /**
     * Get terminal height from settings with fallback to default value
     * @type {number}
     */
    const terminalHeight = settings.terminal_height || 240;

    /**
     * Load persistent command history from localStorage on component mount
     */
    useEffect(() => {
        try {
            const savedHistory = localStorage.getItem('terminal-command-history');
            if (savedHistory) {
                const parsed = JSON.parse(savedHistory);
                setPersistentHistory(parsed.slice(-50)); // Keep last 50 commands
            }
        } catch (error) {
            console.error('Failed to load command history:', error);
        }
    }, []);

    /**
     * Initialize terminal with welcome message on first render
     */
    useEffect(() => {
        if (commandHistory.length === 0) {
            const welcomeMessage = {
                type: 'system',
                content: `File Explorer Terminal v2.0
Current directory: ${currentPath || '/'}

Type 'help' to see available commands.
Command history is persistent across sessions.
Use Ctrl+R for reverse search, Ctrl+C to interrupt commands.`,
                timestamp: new Date().toLocaleTimeString(),
            };
            setCommandHistory([welcomeMessage]);
        }
    }, []);

    /**
     * Save command history to localStorage whenever it changes
     */
    useEffect(() => {
        try {
            if (persistentHistory.length > 0) {
                localStorage.setItem('terminal-command-history', JSON.stringify(persistentHistory));
            }
        } catch (error) {
            console.error('Failed to save command history:', error);
        }
    }, [persistentHistory]);

    // Removed the useEffect that automatically adds directory change messages
    // to prevent double messages when using the cd command

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
        // Truncate long paths for better display
        const displayPath = pathDisplay.length > 50 
            ? '...' + pathDisplay.slice(-47)
            : pathDisplay;
        return `${username}@${hostname}:${displayPath}$`;
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
            // Determine if this is a potentially long-running command
            const longRunningCommands = ['ping', 'traceroute', 'curl', 'wget', 'tail -f', 'watch'];
            const isLongRunning = longRunningCommands.some(cmd => command.toLowerCase().startsWith(cmd));
            
            // Use appropriate command execution method
            const commandPromise = isLongRunning 
                ? invoke('execute_command_with_timeout', { 
                    command, 
                    working_directory: currentPath,
                    timeout_seconds: 30
                })
                : invoke('execute_command_improved', { 
                    command, 
                    working_directory: currentPath 
                });
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
  clear [history]         - Clear the terminal (or history with 'clear history')
  history                 - Show recent commands
  ls, dir                 - List directory contents
  pwd                     - Print working directory
  cd [path]               - Change directory (supports .., ~, relative/absolute paths)
  echo <text>             - Print text
  mkdir <name>            - Create directory
  touch <name>            - Create file
  cat <file>              - Display file contents
  tree                    - Show enhanced directory tree with icons
  find <pattern>          - Search for files and directories by name
  which <command>         - Show command type and location
  whoami                  - Show current user
  date                    - Show current date and time
  ping <host>             - Test network connectivity (auto-limited to 4 packets)
  exit                    - Close the terminal
  
  Enhanced features:
  • Persistent command history across sessions (up to 50 commands)
  • Smart tab completion for commands and file paths
  • Real-time directory synchronization with file explorer
  • Syntax highlighting for file listings and paths
  • File type icons and visual formatting
  
  Keyboard shortcuts:
  Ctrl+C                  - Interrupt running command or clear input
  Ctrl+R                  - Reverse search through command history
  ↑/↓                     - Navigate persistent command history
  Tab                     - Smart auto-complete (commands/paths)
  Esc                     - Exit search mode
  
  Note: All system commands run in the current directory.`,
                };

            case 'clear':
                if (args.length > 0 && args[0] === 'history') {
                    // Clear persistent command history
                    setPersistentHistory([]);
                    localStorage.removeItem('terminal-command-history');
                    return {
                        type: 'system',
                        content: 'Command history cleared.',
                    };
                } else {
                    // Clear terminal display
                    setCommandHistory([]);
                    return null;
                }

            case 'history':
                if (persistentHistory.length === 0) {
                    return {
                        type: 'output',
                        content: 'No command history available.',
                    };
                }

                const recentCommands = persistentHistory.slice(-20).map((cmd, index) => {
                    const lineNumber = persistentHistory.length - 20 + index + 1;
                    return `${lineNumber.toString().padStart(3, ' ')}  ${cmd}`;
                }).join('\n');

                return {
                    type: 'output',
                    content: `Recent commands:\n${recentCommands}\n\nTip: Use 'clear history' to clear command history.`,
                };


            case 'cd': {
                console.log('[Terminal cd] Starting cd command with args:', args);
                try {
                    if (args.length === 0) {
                        // No argument provided - go to home directory
                        try {
                            const homeResult = await invoke('execute_command_improved', {
                                command: process.platform === 'win32' ? 'echo %USERPROFILE%' : 'echo $HOME',
                                working_directory: null
                            });
                            
                            let homeDir;
                            if (typeof homeResult === 'string') {
                                const parsed = JSON.parse(homeResult);
                                homeDir = parsed.stdout.trim();
                            }
                            
                            if (homeDir) {
                                await navigateTo(homeDir);
                                console.log('[Terminal cd] Successfully navigated to home');
                                return { type: 'output', content: '' }; // Silent success - empty content
                            }
                        } catch (error) {
                            // Fallback to current path
                            console.log('[Terminal cd] Home navigation failed, showing current path');
                            return {
                                type: 'output',
                                content: currentPath || '/',
                            };
                        }
                    }

                    let targetPath = args[0];
                    let resolvedPath;
                    
                    console.log('[Terminal cd] Input:', targetPath, 'Current path:', currentPath);
                    
                    // Handle special cases
                    if (targetPath === '~') {
                        // Handle home directory
                        const homeResult = await invoke('execute_command_improved', {
                            command: process.platform === 'win32' ? 'echo %USERPROFILE%' : 'echo $HOME',
                            working_directory: null
                        });
                        if (typeof homeResult === 'string') {
                            const parsed = JSON.parse(homeResult);
                            resolvedPath = parsed.stdout.trim();
                        }
                    } else if (targetPath.startsWith('/') || targetPath.match(/^[A-Za-z]:/)) {
                        // Absolute path - use as is
                        resolvedPath = targetPath;
                    } else {
                        // Relative path - build from current path
                        const basePath = currentPath || '/';
                        if (basePath === '/') {
                            resolvedPath = '/' + targetPath;
                        } else {
                            resolvedPath = basePath + '/' + targetPath;
                        }
                    }

                    // Clean up path - remove trailing slashes except for root
                    if (resolvedPath.endsWith('/') && resolvedPath.length > 1) {
                        resolvedPath = resolvedPath.slice(0, -1);
                    }

                    // Resolve . and .. segments
                    const parts = resolvedPath.split('/');
                    const stack = [];
                    for (let part of parts) {
                        if (part === '' || part === '.') continue;
                        if (part === '..') {
                            if (stack.length > 0) stack.pop();
                        } else {
                            stack.push(part);
                        }
                    }
                    
                    // Final path construction
                    const finalPath = stack.length > 0 ? '/' + stack.join('/') : '/';

                    console.log('[Terminal cd] Resolved to:', finalPath);

                    // Try to navigate directly - let the file explorer handle validation
                    await navigateTo(finalPath);
                    console.log('[Terminal cd] Navigation succeeded');
                    return { type: 'output', content: '' }; // Silent success - empty content

                } catch (error) {
                    console.error('[Terminal cd] Error in cd command:', error);
                    return {
                        type: 'error',
                        content: `cd: ${args[0] || ''}: No such file or directory`,
                    };
                }
            }

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
                    
                    if (data.directories.length === 0 && data.files.length === 0) {
                        return {
                            type: 'output',
                            content: 'Directory is empty',
                        };
                    }
                    
                    // Format directories and files separately for better display
                    const directories = data.directories.map(dir => `${dir.name}/`);
                    const files = data.files.map(file => file.name);
                    
                    // Create formatted output with proper spacing
                    const allItems = [...directories, ...files];
                    const content = allItems.join('  ');
                    
                    return {
                        type: 'output',
                        content: content,
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `Cannot list directory: ${error.message || error}`,
                    };
                }

            case 'tree':
                try {
                    const dirContent = await invoke('open_directory', { path: currentPath });
                    const data = JSON.parse(dirContent);
                    
                    const pathParts = currentPath.split('/').filter(part => part);
                    const folderName = pathParts[pathParts.length - 1] || 'root';
                    
                    let tree = `📁 ${folderName}/\n`;
                    
                    const totalItems = data.directories.length + data.files.length;
                    if (totalItems === 0) {
                        tree += '   (empty directory)\n';
                        return {
                            type: 'output',
                            content: tree,
                        };
                    }

                    // Sort directories first, then files
                    const sortedDirectories = data.directories.sort((a, b) => a.name.localeCompare(b.name));
                    const sortedFiles = data.files.sort((a, b) => a.name.localeCompare(b.name));

                    sortedDirectories.forEach((dir, index) => {
                        const isLastDir = index === sortedDirectories.length - 1 && sortedFiles.length === 0;
                        const connector = isLastDir ? '└── ' : '├── ';
                        tree += `${connector}📁 ${dir.name}/\n`;
                    });

                    sortedFiles.forEach((file, index) => {
                        const isLast = index === sortedFiles.length - 1;
                        const connector = isLast ? '└── ' : '├── ';
                        
                        // Get file extension for icon
                        const ext = file.name.split('.').pop()?.toLowerCase() || '';
                        let icon = '📄';
                        
                        if (['js', 'jsx'].includes(ext)) icon = '🟨';
                        else if (['ts', 'tsx'].includes(ext)) icon = '🔷';
                        else if (ext === 'py') icon = '🐍';
                        else if (ext === 'java') icon = '☕';
                        else if (ext === 'rs') icon = '🦀';
                        else if (['png', 'jpg', 'jpeg', 'gif', 'svg'].includes(ext)) icon = '🖼️';
                        else if (ext === 'pdf') icon = '📕';
                        else if (['zip', 'tar', 'gz'].includes(ext)) icon = '📦';
                        else if (ext === 'md') icon = '📝';
                        
                        tree += `${connector}${icon} ${file.name}\n`;
                    });

                    tree += `\n📊 ${sortedDirectories.length} directories, ${sortedFiles.length} files`;

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

            case 'find':
                if (args.length === 0) {
                    return {
                        type: 'error',
                        content: 'find: missing search pattern\nUsage: find <pattern>',
                    };
                }
                
                try {
                    const pattern = args[0].toLowerCase();
                    const dirContent = await invoke('open_directory', { path: currentPath });
                    const data = JSON.parse(dirContent);
                    
                    const matches = [
                        ...data.directories.filter(dir => 
                            dir.name.toLowerCase().includes(pattern)
                        ).map(dir => `📁 ${dir.name}/`),
                        ...data.files.filter(file => 
                            file.name.toLowerCase().includes(pattern)
                        ).map(file => {
                            const ext = file.name.split('.').pop()?.toLowerCase() || '';
                            let icon = '📄';
                            if (['js', 'jsx'].includes(ext)) icon = '🟨';
                            else if (['ts', 'tsx'].includes(ext)) icon = '🔷';
                            else if (ext === 'py') icon = '🐍';
                            else if (['png', 'jpg', 'jpeg', 'gif', 'svg'].includes(ext)) icon = '🖼️';
                            return `${icon} ${file.name}`;
                        })
                    ];
                    
                    if (matches.length === 0) {
                        return {
                            type: 'output',
                            content: `No files or directories found matching '${args[0]}'`,
                        };
                    }
                    
                    return {
                        type: 'output',
                        content: `Found ${matches.length} matches for '${args[0]}':\n\n${matches.join('\n')}`,
                    };
                } catch (error) {
                    return {
                        type: 'error',
                        content: `find: ${error.message || error}`,
                    };
                }

            case 'which':
                if (args.length === 0) {
                    return {
                        type: 'error',
                        content: 'which: missing command name',
                    };
                }
                
                const builtinCommands = ['help', 'clear', 'history', 'ls', 'dir', 'pwd', 'cd', 'mkdir', 'touch', 'cat', 'tree', 'echo', 'whoami', 'date', 'find', 'which', 'ping', 'exit'];
                const searchCommand = args[0].toLowerCase();
                
                if (builtinCommands.includes(searchCommand)) {
                    return {
                        type: 'output',
                        content: `${searchCommand}: built-in terminal command`,
                    };
                } else {
                    // Check if it's a system command by trying to execute it with --version or --help
                    try {
                        const testResult = await invoke('execute_command_improved', { 
                            command: `${searchCommand} --version`, 
                            working_directory: currentPath 
                        });
                        return {
                            type: 'output',
                            content: `${searchCommand}: system command (use system PATH)`,
                        };
                    } catch {
                        return {
                            type: 'output',
                            content: `${searchCommand}: command not found`,
                        };
                    }
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

        // Save command to persistent history (avoid duplicates)
        setPersistentHistory(prev => {
            const filtered = prev.filter(cmd => cmd !== command);
            return [...filtered, command].slice(-50); // Keep last 50 commands
        });

        // Process command
        let response;

        console.log('[Terminal] Processing command:', cmd.toLowerCase(), 'with args:', args);

        // Check for built-in commands first
        response = await handleBuiltinCommand(cmd.toLowerCase(), args);
        console.log('[Terminal] Built-in command response:', response);

        // If not a built-in command, execute as system command
        if (response === null && cmd.toLowerCase() !== 'clear') {
            console.log('[Terminal] Falling back to system command execution');
            response = await executeCommand(command);
        }

        // Add response to history if there's content or it's a system message
        if (response && (response.content.trim() || response.type === 'system')) {
            setCommandHistory(prev => [...prev, {
                ...response,
                originalCommand: response.type === 'output' ? command : undefined,
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
        const value = e.target.value;
        
        if (isSearchMode) {
            setSearchQuery(value);
            handleHistorySearch(value);
        } else {
            setCurrentCommand(value);
        }
    };

    /**
     * Formats terminal output with basic syntax highlighting and structure
     * @param {string} content - Raw output content
     * @param {string} command - The original command that generated the output
     * @returns {string} Formatted content with HTML-like structure
     */
    const formatOutput = (content, command = '') => {
        if (!content || typeof content !== 'string') return content;

        // Format ls/dir output to highlight directories and files differently
        if (command.startsWith('ls') || command.startsWith('dir')) {
            return content.split(/\s+/).map(item => {
                if (item.endsWith('/')) {
                    return `<span class="terminal-directory">${item}</span>`;
                } else if (item.includes('.')) {
                    const ext = item.split('.').pop().toLowerCase();
                    return `<span class="terminal-file terminal-file-${ext}">${item}</span>`;
                } else {
                    return `<span class="terminal-file">${item}</span>`;
                }
            }).join('  ');
        }

        // For simple outputs like version numbers and paths, don't apply complex formatting
        // This prevents issues with version numbers like "11.5.0" and path formatting in pwd
        const isSimpleOutput = content.trim().match(/^[\d\.]+$/) || 
                              content.trim().match(/^v?[\d\.]+(-[\w\.]+)?$/) ||
                              content.trim().match(/^\/[\w\-\.\/]+$/) || // Unix absolute paths
                              content.trim().match(/^[A-Za-z]:[\\\w\-\.\\]+$/); // Windows paths
        
        if (isSimpleOutput) {
            return content;
        }

        // Only apply formatting to more complex outputs
        let formattedContent = content;
        
        // Format file paths (only if they look like actual paths in complex text)
        formattedContent = formattedContent.replace(/\b([\/\\][\w\-\.\/\\]{3,})\b/g, '<span class="terminal-path">$1</span>');
        
        // Format URLs
        formattedContent = formattedContent.replace(/(https?:\/\/[^\s]+)/g, '<span class="terminal-url">$1</span>');
        
        // Format quoted strings
        formattedContent = formattedContent.replace(/["']([^"']+)["']/g, '<span class="terminal-string">"$1"</span>');
        
        return formattedContent;
    };

    /**
     * Handles tab completion for commands and file paths
     * Provides intelligent completion based on context
     * @async
     */
    const handleTabCompletion = async () => {
        if (!currentCommand.trim()) return;

        const parts = currentCommand.split(' ');
        const lastPart = parts[parts.length - 1];
        const isFirstWord = parts.length === 1;

        if (isFirstWord) {
            // Complete command names
            const commonCommands = ['help', 'clear', 'history', 'ls', 'dir', 'pwd', 'cd', 'mkdir', 'touch', 'cat', 'tree', 'find', 'which', 'echo', 'whoami', 'date', 'exit'];
            const matches = commonCommands.filter(cmd => cmd.startsWith(lastPart.toLowerCase()));

            if (matches.length === 1) {
                setCurrentCommand(matches[0] + ' ');
            } else if (matches.length > 1) {
                // Show available options in terminal
                const optionsMessage = {
                    type: 'system',
                    content: `Available commands: ${matches.join(', ')}`,
                    timestamp: new Date().toLocaleTimeString(),
                };
                setCommandHistory(prev => [...prev, optionsMessage]);
            }
        } else {
            // Complete file/directory paths
            try {
                const dirContent = await invoke('open_directory', { path: currentPath });
                const data = JSON.parse(dirContent);
                
                const allItems = [
                    ...data.directories.map(dir => dir.name + '/'),
                    ...data.files.map(file => file.name)
                ];

                const matches = allItems.filter(item => item.toLowerCase().startsWith(lastPart.toLowerCase()));

                if (matches.length === 1) {
                    const completedParts = parts.slice(0, -1);
                    completedParts.push(matches[0]);
                    setCurrentCommand(completedParts.join(' '));
                } else if (matches.length > 1) {
                    // Show available files/directories
                    const optionsMessage = {
                        type: 'system',
                        content: `Available items: ${matches.join(', ')}`,
                        timestamp: new Date().toLocaleTimeString(),
                    };
                    setCommandHistory(prev => [...prev, optionsMessage]);
                }
            } catch (error) {
                // Silently ignore errors in tab completion
            }
        }
    };

    /**
     * Searches through command history based on query
     * @param {string} query - Search query
     * @returns {string[]} Filtered commands
     */
    const searchHistory = (query) => {
        if (!query.trim()) return [];
        return persistentHistory.filter(cmd => 
            cmd.toLowerCase().includes(query.toLowerCase())
        ).slice(-10); // Show last 10 matches
    };

    /**
     * Handles search mode toggle and functionality
     * @param {string} query - Current search query
     */
    const handleHistorySearch = (query) => {
        const results = searchHistory(query);
        setSearchResults(results);
        setSelectedSearchIndex(results.length > 0 ? 0 : -1);
    };

    /**
     * Exits search mode and resets search state
     */
    const exitSearchMode = () => {
        setIsSearchMode(false);
        setSearchQuery('');
        setSearchResults([]);
        setSelectedSearchIndex(-1);
    };

    /**
     * Handles keyboard navigation through command history and tab completion
     * Supports arrow up/down for history navigation and tab for command completion
     * Also handles Ctrl+C for command interruption and Ctrl+R for search
     *
     * @param {React.KeyboardEvent} e - Keyboard event
     */
    const handleKeyDown = async (e) => {
        // Handle Ctrl+R for reverse search
        if (e.ctrlKey && e.key === 'r') {
            e.preventDefault();
            if (!isSearchMode) {
                setIsSearchMode(true);
                setSearchQuery('');
                setCurrentCommand('');
            }
            return;
        }

        // Handle search mode
        if (isSearchMode) {
            if (e.key === 'Escape') {
                e.preventDefault();
                exitSearchMode();
                return;
            }

            if (e.key === 'Enter') {
                e.preventDefault();
                if (selectedSearchIndex >= 0 && searchResults[selectedSearchIndex]) {
                    setCurrentCommand(searchResults[selectedSearchIndex]);
                    exitSearchMode();
                }
                return;
            }

            if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
                e.preventDefault();
                if (searchResults.length > 0) {
                    let newIndex;
                    if (e.key === 'ArrowUp') {
                        newIndex = selectedSearchIndex > 0 ? selectedSearchIndex - 1 : searchResults.length - 1;
                    } else {
                        newIndex = selectedSearchIndex < searchResults.length - 1 ? selectedSearchIndex + 1 : 0;
                    }
                    setSelectedSearchIndex(newIndex);
                }
                return;
            }

            return; // Let other keys be handled normally in search mode
        }

        // Handle Ctrl+C to interrupt command execution or clear current input
        if (e.ctrlKey && e.key === 'c') {
            e.preventDefault();
            
            if (isExecuting && abortController) {
                // Interrupt running command
                abortController.abort();
                setIsExecuting(false);
                setAbortController(null);
                
                // Add visual feedback that the command was interrupted
                setCommandHistory(prev => [...prev, {
                    type: 'error',
                    content: '❌ Command interrupted by user (Ctrl+C)',
                    timestamp: new Date().toLocaleTimeString(),
                }]);
                
                // Clear the current command line
                setCurrentCommand('');
                setHistoryIndex(-1);
                return;
            } else if (currentCommand.trim()) {
                // Clear current input and show ^C
                const commandEntry = {
                    type: 'command',
                    prompt: getPrompt(),
                    content: currentCommand + ' ^C',
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
            
            if (persistentHistory.length > 0) {
                const newIndex = historyIndex < persistentHistory.length - 1
                    ? historyIndex + 1
                    : historyIndex;

                if (newIndex >= 0 && newIndex < persistentHistory.length) {
                    setCurrentCommand(persistentHistory[persistentHistory.length - 1 - newIndex]);
                    setHistoryIndex(newIndex);
                }
            }
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();

            if (historyIndex > 0) {
                const newIndex = historyIndex - 1;
                setCurrentCommand(persistentHistory[persistentHistory.length - 1 - newIndex]);
                setHistoryIndex(newIndex);
            } else if (historyIndex === 0) {
                setCurrentCommand('');
                setHistoryIndex(-1);
            }
        } else if (e.key === 'Tab') {
            e.preventDefault();
            await handleTabCompletion();
        }
    };

    if (!isOpen) return null;

    /**
     * Handles clearing the terminal display
     */
    const handleClearTerminal = () => {
        setCommandHistory([]);
    };

    return (
        <div className="enhanced-terminal" style={{ height: `${terminalHeight}px` }}>
            <div className="terminal-header">
                <div className="terminal-title">
                    <span>Terminal</span>
                </div>
                <div className="terminal-controls">
                    <button 
                        className="terminal-control terminal-clear"
                        onClick={handleClearTerminal}
                        title="Clear terminal"
                    >
                        <Icon name="trash" size="small" className="terminal-icon" />
                    </button>
                    <button 
                        className="terminal-control terminal-close"
                        onClick={onToggle}
                        title="Close terminal"
                    >
                        <Icon name="x" size="small" className="terminal-icon" />
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
                                <div className="terminal-command-line">
                                    <span className="terminal-prompt-user">{entry.prompt.split('@')[0]}</span>
                                    <span className="terminal-prompt-at">@</span>
                                    <span className="terminal-prompt-host">{entry.prompt.split('@')[1].split(':')[0]}</span>
                                    <span className="terminal-prompt-colon">:</span>
                                    <span className="terminal-prompt-path">{entry.prompt.split(':')[1].replace('$', '')}</span>
                                    <span className="terminal-prompt-dollar">$ </span>
                                    <span className="terminal-command-text">{entry.content}</span>
                                </div>
                            )}
                            {entry.type !== 'command' && (
                                <pre 
                                    className="terminal-text"
                                    dangerouslySetInnerHTML={{ 
                                        __html: entry.type === 'output' && entry.originalCommand 
                                            ? formatOutput(entry.content, entry.originalCommand) 
                                            : entry.content 
                                    }}
                                ></pre>
                            )}
                        </div>
                    </div>
                ))}
                <form onSubmit={handleSubmit} className="terminal-input-line">
                    {isSearchMode ? (
                        <span className="terminal-prompt-search">(reverse-i-search) </span>
                    ) : (
                        <div className="terminal-current-prompt">
                            <span className="terminal-prompt-user">{getPrompt().split('@')[0]}</span>
                            <span className="terminal-prompt-at">@</span>
                            <span className="terminal-prompt-host">{getPrompt().split('@')[1].split(':')[0]}</span>
                            <span className="terminal-prompt-colon">:</span>
                            <span className="terminal-prompt-path">{getPrompt().split(':')[1].replace('$', '')}</span>
                            <span className="terminal-prompt-dollar">$ </span>
                        </div>
                    )}
                    <input
                        ref={inputRef}
                        type="text"
                        className="terminal-input"
                        value={isSearchMode ? searchQuery : currentCommand}
                        onChange={handleChange}
                        onKeyDown={handleKeyDown}
                        disabled={isExecuting}
                        autoFocus
                        spellCheck="false"
                        autoComplete="off"
                        autoCapitalize="off"
                        placeholder={isSearchMode ? "Start typing to search history..." : ""}
                    />
                    {!isSearchMode && isExecuting && (
                        <span className="terminal-executing">
                            <span className="spinner-small"></span>
                            <span className="terminal-interrupt-hint">⏱️ Running... Press Ctrl+C to interrupt</span>
                        </span>
                    )}
                </form>
            </div>
        </div>
    );
};

export default Terminal;

