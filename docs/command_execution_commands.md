# Tauri Command Execution Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content
- [Execute Command](#execute_command-endpoint)
- [Execute Command Improved](#execute_command_improved-endpoint)
- [Execute Command With Timeout](#execute_command_with_timeout-endpoint)

# `execute_command` endpoint

---
## Parameters
- `command`: A string representing the shell command to execute. This will be split into a program name and arguments.
- `working_directory`: Optional string specifying the directory to run the command in.

## Returns
- Ok(String) - The combined stdout and stderr output from the command as a string.
- Err(String) - An error message if the command cannot be executed or other errors occur. 

## JSON Example Response
```json
{
    "stdout":"hello world\n",
    "stderr":"",
    "status":0,
    "exec_time_in_ms":3
}
```

## Description
Executes a shell command and returns its output. The command string is split into a program name and arguments. Both stdout and stderr are captured and combined in the response. If the command fails (non-zero exit status), the exit status is appended to the output.

## Example call
```typescript jsx
useEffect(() => {
    const runCommand = async () => {
        try {
            // Run a simple command
            const result = await invoke("execute_command", { 
                command: "ls -la" 
            });
            console.log("Command output:", result);
            
            // Run another command
            const gitStatus = await invoke("execute_command", { 
                command: "git status" 
            });
            console.log("Git status:", gitStatus);
        } catch (error) {
            console.error("Error executing command:", error);
        }
    };

    runCommand();
}, []);
```

# `execute_command_improved` endpoint

---
## Parameters
- `command`: A string representing the shell command to execute.
- `working_directory`: Optional string specifying the directory to run the command in.

## Returns
- Ok(String) - JSON string containing CommandResponse with stdout, stderr, status, and execution time.
- Err(String) - An error message if the command cannot be executed.

## JSON Example Response
```json
{
    "stdout":"hello world\n",
    "stderr":"",
    "status":0,
    "exec_time_in_ms":5
}
```

## Description
An improved version of `execute_command` with better error handling and environment setup. Uses PowerShell on Windows and bash/sh on Unix systems. Includes better PATH handling, locale settings, and more comprehensive error reporting.

## Example call
```typescript jsx
const result = await invoke("execute_command_improved", { 
    command: "git status",
    working_directory: "/path/to/repo"
});
```

# `execute_command_with_timeout` endpoint

---
## Parameters
- `command`: A string representing the shell command to execute.
- `working_directory`: Optional string specifying the directory to run the command in.
- `timeout_seconds`: Optional timeout in seconds (default: 30 seconds).

## Returns
- Ok(String) - JSON string containing CommandResponse with stdout, stderr, status, and execution time.
- Err(String) - An error message if the command cannot be executed or times out.

## JSON Example Response
```json
{
    "stdout":"PING example.com (93.184.216.34): 56 data bytes\n64 bytes from 93.184.216.34: icmp_seq=0 ttl=56 time=12.345 ms\n",
    "stderr":"",
    "status":0,
    "exec_time_in_ms":1234
}
```

## Description
Executes shell commands with timeout support for potentially long-running commands. Automatically modifies certain commands (like `ping`) to prevent infinite execution by adding appropriate flags. Uses async execution with configurable timeout.

## Example call
```typescript jsx
// Run ping with 10 second timeout
const result = await invoke("execute_command_with_timeout", { 
    command: "ping google.com",
    working_directory: null,
    timeout_seconds: 10
});
```

## Security Considerations
All `execute_command` functions execute arbitrary shell commands with the same permissions as the application. Exercise caution when accepting command input from untrusted sources, as this could lead to security vulnerabilities. Consider implementing restrictions on allowed commands or validating input before execution in production environments.
