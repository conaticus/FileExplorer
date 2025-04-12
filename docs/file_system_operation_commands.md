# Tauri Filesystem Commands Documentation

## Overview

This document provides a reference for filesystem-related commands available in a Tauri application with a React frontend.

## Command Reference

### File Operations

### `open_file`

**Description**: Opens a text file and returns its contents as a string.

**Parameters**:
- `path` (string): Path to the file to be opened.

**Returns**:
- Success: `string` - The contents of the file.
- Error: Error message explaining why the file couldn't be opened.

### `create_file`

**Description**: Creates a new empty file in the specified directory.

**Parameters**:
- `folder_path_abs` (string): Absolute path to the directory where the file will be created.
- `filename` (string): Name of the file to create.

**Returns**:
- Success: Empty result.
- Error: Error message explaining why the file couldn't be created.

### `move_file_to_trash`

**Description**: Moves a file to the system trash instead of permanently deleting it.

**Parameters**:
- `path` (string): Path to the file to be moved to trash.

**Returns**:
- Success: Empty result.
- Error: Error message explaining why the file couldn't be moved to trash.

### `rename_file`

**Description**: Renames or moves a file from one path to another.

**Parameters**:
- `old_path` (string): Current path of the file.
- `new_path` (string): New path for the file.

**Returns**:
- Success: Empty result.
- Error: Error message explaining why the file couldn't be renamed.

### Directory Operations

### `open_directory`

**Description**: Lists contents of a directory, including files and subdirectories with metadata.

**Parameters**:
- `path` (string): Path to the directory to open.

**Returns**:
- Success: JSON string containing: