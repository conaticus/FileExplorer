# Tauri Search Engine Commands Documentation

## Content
- [Search for Files](#search-endpoint)
- [Search with Extension](#search_with_extension-endpoint)
- [Add Paths Recursively](#add_paths_recursive-endpoint)
- [Add a Single Path](#add_path-endpoint)
- [Remove Paths Recursively](#remove_paths_recursive-endpoint)
- [Remove a Single Path](#remove_path-endpoint)
- [Clear Search Engine](#clear_search_engine-endpoint)
- [Get Search Engine Info](#get_search_engine_info-endpoint)

# `search` endpoint

---
## Parameters
- `query`: The search query string. This should be a string representing the text to search for.

## Returns
- `Ok(SearchResult)`: A vector of paths and their relevance scores that match the query. Each result is a tuple containing the file path as a string and a relevance score as a floating-point number.
- `Err(String)`: An error message if there was an error during the search operation.

## Example call
```typescript jsx
useEffect(() => {
    const performSearch = async () => {
        try {
            const result = await invoke("search", { query: "document" });
            console.log("Search results:", result);
            // result is an array of [path, score] tuples
            // e.g. [["/path/to/document.txt", 0.95], ["/path/to/other.doc", 0.82]]
        } catch (error) {
            console.error("Search error:", error);
        }
    };

    performSearch();
}, []);
```

# `search_with_extension` endpoint

---
## Parameters
- `query`: The search query string. This should be a string representing the text to search for.
- `extensions`: A vector of file extensions to filter by (e.g., ["txt", "md"]). Only files with these extensions will be included in search results.

## Returns
- `Ok(SearchResult)`: A vector of paths and their relevance scores that match the query and extension filters. Each result is a tuple containing the file path as a string and a relevance score as a floating-point number.
- `Err(String)`: An error message if there was an error during the search operation.

## Example call
```typescript jsx
useEffect(() => {
    const performSearch = async () => {
        try {
            const result = await invoke("search_with_extension", { 
                query: "document", 
                extensions: ["txt", "md"] 
            });
            console.log("Search results:", result);
            // result is an array of [path, score] tuples with the specified extensions
        } catch (error) {
            console.error("Search error:", error);
        }
    };

    performSearch();
}, []);
```

# `add_paths_recursive` endpoint

---
## Parameters
- `folder`: The path to the directory to index. This should be a string representing the absolute path to the directory.

## Returns
- `Ok(())`: No content is returned. The function will start the indexing process for the specified directory and all its subdirectories.
- `Err(String)`: An error message if there was an error starting the indexing process.

## Example call
```typescript jsx
const startIndexing = async () => {
    try {
        await invoke("add_paths_recursive", { 
            folder: "/path/to/documents" 
        });
        console.log("Started indexing the directory");
    } catch (error) {
        console.error("Failed to start indexing:", error);
    }
};
```

# `add_path` endpoint

---
## Parameters
- `path`: The path to the file to add to the index. This should be a string representing the absolute path to the file.

## Returns
- `Ok(())`: No content is returned. The function will add the specified file to the search index.
- `Err(String)`: An error message if there was an error adding the file.

## Example call
```typescript jsx
const addFileToIndex = async () => {
    try {
        await invoke("add_path", { 
            path: "/path/to/document.txt" 
        });
        console.log("File added to index");
    } catch (error) {
        console.error("Failed to add file:", error);
    }
};
```

# `remove_paths_recursive` endpoint

---
## Parameters
- `folder`: The path to the directory to remove from the index. This should be a string representing the absolute path to the directory.

## Returns
- `Ok(())`: No content is returned. The function will remove the specified directory and all its contents from the search index.
- `Err(String)`: An error message if there was an error removing the directory.

## Example call
```typescript jsx
const removeDirectory = async () => {
    try {
        await invoke("remove_paths_recursive", { 
            folder: "/path/to/old_documents" 
        });
        console.log("Directory removed from index");
    } catch (error) {
        console.error("Failed to remove directory:", error);
    }
};
```

# `remove_path` endpoint

---
## Parameters
- `path`: The path to the file to remove from the index. This should be a string representing the absolute path to the file.

## Returns
- `Ok(())`: No content is returned. The function will remove the specified file from the search index.
- `Err(String)`: An error message if there was an error removing the file.

## Example call
```typescript jsx
const removeFile = async () => {
    try {
        await invoke("remove_path", { 
            path: "/path/to/old_document.txt" 
        });
        console.log("File removed from index");
    } catch (error) {
        console.error("Failed to remove file:", error);
    }
};
```

# `clear_search_engine` endpoint

---
## Parameters
None. This command does not take any parameters.

## Returns
- `Ok(())`: No content is returned. The function will clear all indexed data from the search engine.
- `Err(String)`: An error message if there was an error clearing the search engine.

## Example call
```typescript jsx
const clearSearchEngine = async () => {
    try {
        await invoke("clear_search_engine");
        console.log("Search engine index cleared");
    } catch (error) {
        console.error("Failed to clear search engine:", error);
    }
};
```

# `get_search_engine_info` endpoint

---
## Parameters
None. This command does not take any parameters.

## Returns
- `Ok(SearchEngineInfo)`: A struct containing all relevant search engine information including:
  - `status`: The current status of the search engine
  - `progress`: Information about indexing progress
  - `metrics`: Performance metrics of the search engine
  - `stats`: Statistics about the engine's data structures
  - `last_updated`: Timestamp of when the engine was last updated

- `Err(String)`: An error message if there was an error retrieving the information.

## Description
Retrieves comprehensive information about the search engine's current state including status, indexing progress, metrics, recent activity, and engine statistics.

## Example call
```typescript jsx
const getEngineInfo = async () => {
    try {
        const info = await invoke("get_search_engine_info");
        console.log("Search engine status:", info.status);
        console.log("Indexing progress:", info.progress.percentage_complete + "%");
        console.log("Files indexed:", `${info.progress.files_indexed}/${info.progress.files_discovered}`);
        console.log("Currently indexing:", info.progress.current_path);
        
        console.log("Total searches:", info.metrics.total_searches);
        console.log("Average search time:", info.metrics.average_search_time_ms + "ms");
        
        console.log("Index size:", info.stats.trie_size + " entries");
        
        // Convert timestamp to readable date
        const lastUpdated = new Date(info.last_updated);
        console.log("Last updated:", lastUpdated.toLocaleString());
    } catch (error) {
        console.error("Failed to get search engine info:", error);
    }
};
```