# Tauri Command Execution Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content
- [Execute Command](#execute_command-endpoint)

# `execute_command` endpoint

---
## Parameters
- `command`: A string representing the shell command to execute. This will be split into a program name and arguments.

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

## Security Considerations
The `execute_command` function executes arbitrary shell commands with the same permissions as the application. Exercise caution when accepting command input from untrusted sources, as this could lead to security vulnerabilities. Consider implementing restrictions on allowed commands or validating input before execution in production environments.
